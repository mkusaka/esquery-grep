mod matcher;

use esquery_selector::{self as sel, AttrValue, Selector, SelectorKind};
use serde_json::Value;

/// Query a JSON ESTree AST with an ESQuery selector string.
/// Returns references to all matching nodes.
/// Returns empty if the selector is invalid (including invalid regex patterns).
pub fn query<'a>(ast: &'a Value, selector_str: &str) -> Vec<&'a Value> {
    let Some(selector) = sel::parse(selector_str) else {
        return vec![];
    };
    // Validate all regex patterns in the selector (JS throws on invalid regex at parse time)
    if !validate_regexes(&selector) {
        return vec![];
    }
    query_selector(ast, &selector)
}

/// Query using a pre-parsed selector.
/// Returns empty if the selector contains invalid regex patterns.
pub fn query_selector<'a>(ast: &'a Value, selector: &Selector) -> Vec<&'a Value> {
    if !validate_regexes(selector) {
        return vec![];
    }
    let alt_subjects = subjects(selector);
    let mut results = Vec::new();
    let mut ancestry = Vec::new();

    traverse_and_match(ast, selector, &alt_subjects, &mut ancestry, &mut results);
    results
}

/// Validate that all regex patterns in the selector can be compiled.
/// JS esquery constructs RegExp objects at parse time and throws on invalid patterns.
fn validate_regexes(selector: &Selector) -> bool {
    match &selector.kind {
        SelectorKind::Attribute(attr) => {
            if let Some(AttrValue::Regex(re)) = &attr.value
                && matcher::build_regex(re).is_none()
            {
                return false;
            }
            true
        }
        SelectorKind::Compound(sels)
        | SelectorKind::Matches(sels)
        | SelectorKind::Not(sels)
        | SelectorKind::Has(sels) => sels.iter().all(validate_regexes),
        SelectorKind::Child { left, right }
        | SelectorKind::Descendant { left, right }
        | SelectorKind::Sibling { left, right }
        | SelectorKind::Adjacent { left, right } => {
            validate_regexes(left) && validate_regexes(right)
        }
        _ => true,
    }
}

/// Check if a node matches a selector given its ancestry.
/// Returns false for non-node values (like JS's `if (!node) return false`).
/// Returns false if the selector contains invalid regex patterns.
pub fn matches(node: &Value, selector: &Selector, ancestry: &[&Value]) -> bool {
    if !is_node(node) {
        return false;
    }
    if !validate_regexes(selector) {
        return false;
    }
    matcher::matches_node(node, selector, ancestry)
}

// -- Traversal --

fn traverse_and_match<'a>(
    node: &'a Value,
    selector: &Selector,
    alt_subjects: &[Selector],
    ancestry: &mut Vec<&'a Value>,
    results: &mut Vec<&'a Value>,
) {
    if !is_node(node) {
        return;
    }

    if matcher::matches_node(node, selector, ancestry) {
        if alt_subjects.is_empty() {
            results.push(node);
        } else {
            for alt in alt_subjects {
                if matcher::matches_node(node, alt, ancestry) {
                    results.push(node);
                }
                for k in 0..ancestry.len() {
                    let succeeding: Vec<&Value> = ancestry[k + 1..].to_vec();
                    if matcher::matches_node(ancestry[k], alt, &succeeding) {
                        results.push(ancestry[k]);
                    }
                }
            }
        }
    }

    // Recurse into children
    ancestry.insert(0, node);
    for key in visitor_keys(node) {
        match node.get(key) {
            Some(val) if val.is_array() => {
                for elem in val.as_array().unwrap() {
                    if is_node(elem) {
                        traverse_and_match(elem, selector, alt_subjects, ancestry, results);
                    }
                }
            }
            Some(val) if is_node(val) => {
                traverse_and_match(val, selector, alt_subjects, ancestry, results);
            }
            _ => {}
        }
    }
    ancestry.remove(0);
}

// -- Node utilities --

/// Check if a JSON value is an AST node (object with "type" string property).
pub fn is_node(val: &Value) -> bool {
    val.as_object()
        .and_then(|obj| obj.get("type"))
        .is_some_and(|t| t.is_string())
}

/// Get visitor keys for a node.
/// Uses estraverse.VisitorKeys for known ESTree node types.
/// Falls back to all keys except "type" for unknown node types (iteration fallback).
fn visitor_keys(node: &Value) -> Vec<&str> {
    let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if let Some(keys) = estraverse_visitor_keys(node_type) {
        return keys.to_vec();
    }
    // Iteration fallback for unknown node types
    match node.as_object() {
        Some(obj) => obj
            .keys()
            .filter(|k| k.as_str() != "type")
            .map(|k| k.as_str())
            .collect(),
        None => vec![],
    }
}

/// ESTree visitor keys matching estraverse.VisitorKeys.
/// Returns None for unknown node types (triggers iteration fallback).
fn estraverse_visitor_keys(node_type: &str) -> Option<&'static [&'static str]> {
    match node_type {
        "AssignmentExpression" => Some(&["left", "right"]),
        "AssignmentPattern" => Some(&["left", "right"]),
        "ArrayExpression" => Some(&["elements"]),
        "ArrayPattern" => Some(&["elements"]),
        "ArrowFunctionExpression" => Some(&["params", "body"]),
        "AwaitExpression" => Some(&["argument"]),
        "BlockStatement" => Some(&["body"]),
        "BinaryExpression" => Some(&["left", "right"]),
        "BreakStatement" => Some(&["label"]),
        "CallExpression" => Some(&["callee", "arguments"]),
        "CatchClause" => Some(&["param", "body"]),
        "ChainExpression" => Some(&["expression"]),
        "ClassBody" => Some(&["body"]),
        "ClassDeclaration" => Some(&["id", "superClass", "body"]),
        "ClassExpression" => Some(&["id", "superClass", "body"]),
        "ComprehensionBlock" => Some(&["left", "right"]),
        "ComprehensionExpression" => Some(&["blocks", "filter", "body"]),
        "ConditionalExpression" => Some(&["test", "consequent", "alternate"]),
        "ContinueStatement" => Some(&["label"]),
        "DebuggerStatement" => Some(&[]),
        "DirectiveStatement" => Some(&[]),
        "DoWhileStatement" => Some(&["body", "test"]),
        "EmptyStatement" => Some(&[]),
        "ExportAllDeclaration" => Some(&["source"]),
        "ExportDefaultDeclaration" => Some(&["declaration"]),
        "ExportNamedDeclaration" => Some(&["declaration", "specifiers", "source"]),
        "ExportSpecifier" => Some(&["exported", "local"]),
        "ExpressionStatement" => Some(&["expression"]),
        "ForStatement" => Some(&["init", "test", "update", "body"]),
        "ForInStatement" => Some(&["left", "right", "body"]),
        "ForOfStatement" => Some(&["left", "right", "body"]),
        "FunctionDeclaration" => Some(&["id", "params", "body"]),
        "FunctionExpression" => Some(&["id", "params", "body"]),
        "GeneratorExpression" => Some(&["blocks", "filter", "body"]),
        "Identifier" => Some(&[]),
        "IfStatement" => Some(&["test", "consequent", "alternate"]),
        "ImportExpression" => Some(&["source"]),
        "ImportDeclaration" => Some(&["specifiers", "source"]),
        "ImportDefaultSpecifier" => Some(&["local"]),
        "ImportNamespaceSpecifier" => Some(&["local"]),
        "ImportSpecifier" => Some(&["imported", "local"]),
        "Literal" => Some(&[]),
        "LabeledStatement" => Some(&["label", "body"]),
        "LogicalExpression" => Some(&["left", "right"]),
        "MemberExpression" => Some(&["object", "property"]),
        "MetaProperty" => Some(&["meta", "property"]),
        "MethodDefinition" => Some(&["key", "value"]),
        "ModuleSpecifier" => Some(&[]),
        "NewExpression" => Some(&["callee", "arguments"]),
        "ObjectExpression" => Some(&["properties"]),
        "ObjectPattern" => Some(&["properties"]),
        "PrivateIdentifier" => Some(&[]),
        "Program" => Some(&["body"]),
        "Property" => Some(&["key", "value"]),
        "PropertyDefinition" => Some(&["key", "value"]),
        "RestElement" => Some(&["argument"]),
        "ReturnStatement" => Some(&["argument"]),
        "SequenceExpression" => Some(&["expressions"]),
        "SpreadElement" => Some(&["argument"]),
        "Super" => Some(&[]),
        "SwitchStatement" => Some(&["discriminant", "cases"]),
        "SwitchCase" => Some(&["test", "consequent"]),
        "TaggedTemplateExpression" => Some(&["tag", "quasi"]),
        "TemplateElement" => Some(&[]),
        "TemplateLiteral" => Some(&["quasis", "expressions"]),
        "ThisExpression" => Some(&[]),
        "ThrowStatement" => Some(&["argument"]),
        "TryStatement" => Some(&["block", "handler", "finalizer"]),
        "UnaryExpression" => Some(&["argument"]),
        "UpdateExpression" => Some(&["argument"]),
        "VariableDeclaration" => Some(&["declarations"]),
        "VariableDeclarator" => Some(&["id", "init"]),
        "WhileStatement" => Some(&["test", "body"]),
        "WithStatement" => Some(&["object", "body"]),
        "YieldExpression" => Some(&["argument"]),
        _ => None,
    }
}

// -- Subject extraction --
// Mirrors esquery.js `subjects()` function.

fn subjects(selector: &Selector) -> Vec<Selector> {
    subjects_inner(selector, selector)
}

fn subjects_inner(selector: &Selector, ancestor: &Selector) -> Vec<Selector> {
    let mut results = Vec::new();
    if selector.subject {
        results.push(ancestor.clone());
    }
    match &selector.kind {
        SelectorKind::Child { left, right }
        | SelectorKind::Descendant { left, right }
        | SelectorKind::Sibling { left, right }
        | SelectorKind::Adjacent { left, right } => {
            // For 'left' branches, the ancestor becomes the left selector itself
            results.extend(subjects_inner(left, left));
            results.extend(subjects_inner(right, ancestor));
        }
        SelectorKind::Compound(sels)
        | SelectorKind::Matches(sels)
        | SelectorKind::Not(sels)
        | SelectorKind::Has(sels) => {
            for s in sels {
                results.extend(subjects_inner(s, ancestor));
            }
        }
        _ => {}
    }
    results
}
