use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use serde_json::Value;

/// Result of an ESQuery match against a parsed AST.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchResult {
    /// ESTree node type name (e.g., "Identifier", "BinaryExpression")
    pub node_type: String,
    /// UTF-8 byte offset of the start of the matched node
    pub start: u32,
    /// UTF-8 byte offset of the end of the matched node
    pub end: u32,
    /// Source text of the matched node
    pub text: String,
}

/// Source type for parsing.
///
/// Determines how the parser treats the input and which ESTree serialization
/// format is used (JS vs TS).
#[derive(Debug, Clone, Copy, Default)]
pub enum JsSourceType {
    /// JavaScript (parsed as ES module).
    #[default]
    Js,
    /// JavaScript with JSX support.
    Jsx,
    /// TypeScript.
    Ts,
    /// TypeScript with JSX support.
    Tsx,
}

impl JsSourceType {
    fn to_oxc(self) -> SourceType {
        match self {
            JsSourceType::Js => SourceType::mjs(),
            JsSourceType::Jsx => SourceType::jsx(),
            JsSourceType::Ts => SourceType::ts(),
            JsSourceType::Tsx => SourceType::tsx(),
        }
    }
}

/// Query JavaScript/TypeScript source code with an ESQuery selector.
///
/// Parses the source, serializes to ESTree JSON, then matches using
/// esquery-json. Returns match results with source locations.
///
/// Returns an empty `Vec` when:
/// - The source contains syntax errors (partial ASTs are not queried).
/// - The selector is invalid or contains invalid regex patterns.
/// - Internal JSON serialization/deserialization fails.
///
/// # Known limitations
///
/// - TypeScript-specific fields (e.g., `typeAnnotation`) are not traversed
///   because the underlying esquery-json matcher uses estraverse visitor keys,
///   which only cover standard ESTree node types. TS-specific top-level
///   declarations (e.g., `TSInterfaceDeclaration`) are still found.
/// - Regular expression literals are not deeply parsed (`parse_regular_expression`
///   is disabled); the regex body is treated as opaque text. This does not affect
///   AST node matching but means regex pattern validation is skipped.
pub fn query(source: &str, selector: &str, source_type: JsSourceType) -> Vec<MatchResult> {
    let allocator = Allocator::default();
    let oxc_source_type = source_type.to_oxc();
    let ret = Parser::new(&allocator, source, oxc_source_type)
        .with_options(ParseOptions {
            parse_regular_expression: false,
            ..ParseOptions::default()
        })
        .parse();

    // Return empty on parse errors (partial ASTs from error recovery are unreliable)
    if !ret.errors.is_empty() {
        return vec![];
    }

    // Serialize to ESTree JSON string (with ranges for start/end offsets)
    let json_str = if matches!(source_type, JsSourceType::Ts | JsSourceType::Tsx) {
        ret.program.to_estree_ts_json(true)
    } else {
        ret.program.to_estree_js_json(true)
    };

    // Parse JSON string into serde_json::Value
    let json: Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => return vec![],
    };

    // Run ESQuery
    let matches = esquery_json::query(&json, selector);

    // Convert matched nodes to MatchResults
    matches
        .into_iter()
        .filter_map(|node| node_to_match_result(node, source))
        .collect()
}

/// Extract MatchResult from a matched JSON AST node.
fn node_to_match_result(node: &Value, source: &str) -> Option<MatchResult> {
    let node_type = node.get("type")?.as_str()?.to_string();

    // oxc ESTree JSON with ranges=true includes "start" and "end" at top level
    let start = node.get("start")?.as_u64()? as u32;
    let end = node.get("end")?.as_u64()? as u32;

    let text = source
        .get((start as usize)..(end as usize))
        .unwrap_or("")
        .to_string();

    Some(MatchResult {
        node_type,
        start,
        end,
        text,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_identifier_query() {
        let source = "var x = 1;";
        let results = query(source, "Identifier", JsSourceType::Js);
        assert!(!results.is_empty(), "should find identifiers");
        assert!(
            results.iter().any(|r| r.text == "x"),
            "should find identifier 'x', got: {:?}",
            results
        );
    }

    #[test]
    fn literal_query() {
        let source = "var x = 42;";
        let results = query(source, "Literal", JsSourceType::Js);
        assert!(!results.is_empty(), "should find literal, got: {:?}", results);
        assert!(
            results.iter().any(|r| r.text == "42"),
            "should find literal '42', got: {:?}",
            results
        );
    }

    #[test]
    fn wildcard_query() {
        let source = "var x = 1;";
        let results = query(source, "*", JsSourceType::Js);
        assert!(results.len() > 1, "wildcard should match multiple nodes");
    }

    #[test]
    fn empty_results_for_no_match() {
        let source = "var x = 1;";
        let results = query(source, "WhileStatement", JsSourceType::Js);
        assert!(results.is_empty(), "should find no while statements");
    }

    #[test]
    fn typescript_support() {
        let source = "const x: number = 1;";
        let results = query(source, "*", JsSourceType::Ts);
        assert!(!results.is_empty(), "should parse and match TS code");
    }

    #[test]
    fn attribute_selector() {
        let source = "var x = 1 + 2;";
        let results = query(source, r#"BinaryExpression[operator="+"]"#, JsSourceType::Js);
        assert_eq!(results.len(), 1, "should find binary expression with +");
        assert_eq!(results[0].text, "1 + 2");
    }

    #[test]
    fn descendant_selector() {
        let source = "function foo() { return 1; }";
        let results = query(
            source,
            "FunctionDeclaration ReturnStatement",
            JsSourceType::Js,
        );
        assert_eq!(results.len(), 1, "should find return inside function");
    }

    #[test]
    fn compound_selector() {
        let source = "var x = 1; var y = 'hello';";
        let results = query(source, r#"Literal[value="hello"]"#, JsSourceType::Js);
        assert!(
            results.iter().any(|r| r.text.contains("hello")),
            "should find string literal, got: {:?}",
            results
        );
    }

    // -- Class selectors --

    #[test]
    fn class_statement() {
        let source = "if (true) {} else {}";
        let results = query(source, ":statement", JsSourceType::Js);
        assert!(!results.is_empty(), "should find statements");
        assert!(
            results.iter().any(|r| r.node_type == "IfStatement"),
            "should include IfStatement, got: {:?}",
            results
        );
    }

    #[test]
    fn class_expression() {
        let source = "var x = 1 + 2;";
        let results = query(source, ":expression", JsSourceType::Js);
        assert!(!results.is_empty(), "should find expressions");
        assert!(
            results.iter().any(|r| r.node_type == "BinaryExpression"),
            "should include BinaryExpression, got: {:?}",
            results
        );
    }

    // -- Child / sibling selectors --

    #[test]
    fn child_selector() {
        let source = "function foo() { if (true) { return 1; } }";
        let results = query(
            source,
            "FunctionDeclaration > BlockStatement",
            JsSourceType::Js,
        );
        assert_eq!(results.len(), 1, "should find exactly one direct child block");
        assert_eq!(results[0].node_type, "BlockStatement");
    }

    #[test]
    fn sibling_selector() {
        let source = "var x = 1; var y = 2;";
        let results = query(
            source,
            "VariableDeclaration ~ VariableDeclaration",
            JsSourceType::Js,
        );
        assert!(!results.is_empty(), "should find sibling declarations");
    }

    #[test]
    fn adjacent_selector() {
        let source = "var x = 1; var y = 2;";
        let results = query(
            source,
            "VariableDeclaration + VariableDeclaration",
            JsSourceType::Js,
        );
        assert!(!results.is_empty(), "should find adjacent declarations");
    }

    // -- :has / :not / :matches --

    #[test]
    fn has_selector() {
        let source = "function foo() { return 1; } function bar() {}";
        let results = query(
            source,
            "FunctionDeclaration:has(ReturnStatement)",
            JsSourceType::Js,
        );
        assert_eq!(
            results.len(),
            1,
            "only foo should have ReturnStatement, got: {:?}",
            results
        );
        assert!(results[0].text.contains("foo"));
    }

    #[test]
    fn not_selector() {
        let source = "var x = 1; var y = 'hello';";
        let results = query(source, "Literal:not([value=1])", JsSourceType::Js);
        assert!(
            results.iter().all(|r| r.text != "1"),
            "should exclude literal 1, got: {:?}",
            results
        );
    }

    #[test]
    fn matches_selector() {
        let source = "var x = 1; if (true) {}";
        let results = query(
            source,
            ":matches(VariableDeclaration, IfStatement)",
            JsSourceType::Js,
        );
        assert!(results.len() >= 2, "should match both, got: {:?}", results);
    }

    // -- :nth-child --

    #[test]
    fn nth_child_selector() {
        let source = "var x = 1; var y = 2; var z = 3;";
        let results = query(source, ":nth-child(2)", JsSourceType::Js);
        assert!(!results.is_empty(), "should find 2nd child nodes");
    }

    // -- Field selector --

    #[test]
    fn field_selector() {
        let source = "var x = { a: 1 };";
        let results = query(source, "Property > .value", JsSourceType::Js);
        assert!(!results.is_empty(), "should find property value field");
    }

    // -- TypeScript specific --

    #[test]
    fn ts_type_annotation() {
        // Note: estraverse visitor keys don't include TS-specific fields like
        // "typeAnnotation", so TSTypeAnnotation nodes nested inside standard
        // ESTree nodes (VariableDeclarator etc.) are not traversed.
        // This is a known limitation of using estraverse visitor keys.
        // TS-specific top-level declarations are still found.
        let source = "const x: string = 'hello';";
        let results = query(source, "TSTypeAnnotation", JsSourceType::Ts);
        // Currently not traversed — this documents the known limitation
        assert!(
            results.is_empty(),
            "TSTypeAnnotation not reachable via estraverse visitor keys (known limitation)"
        );
    }

    #[test]
    fn ts_interface() {
        let source = "interface Foo { bar: string; }";
        let results = query(source, "TSInterfaceDeclaration", JsSourceType::Ts);
        assert_eq!(results.len(), 1, "should find TS interface");
    }

    // -- JSX --

    #[test]
    fn jsx_element() {
        let source = "const el = <div className='test'>Hello</div>;";
        let results = query(source, "JSXElement", JsSourceType::Jsx);
        assert!(!results.is_empty(), "should find JSX element");
    }

    // -- Match result correctness --

    #[test]
    fn match_result_spans_are_correct() {
        let source = "var foo = 123;";
        let results = query(source, r#"Identifier[name="foo"]"#, JsSourceType::Js);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].node_type, "Identifier");
        assert_eq!(results[0].text, "foo");
        assert_eq!(results[0].start, 4);
        assert_eq!(results[0].end, 7);
    }

    #[test]
    fn invalid_selector_returns_empty() {
        let source = "var x = 1;";
        let results = query(source, "[[[invalid", JsSourceType::Js);
        assert!(results.is_empty(), "invalid selector should return empty");
    }

    #[test]
    fn tsx_element() {
        let source = "const el: JSX.Element = <div>Hello</div>;";
        let results = query(source, "JSXElement", JsSourceType::Tsx);
        assert_eq!(results.len(), 1, "should find JSX element in TSX");
        assert_eq!(results[0].node_type, "JSXElement");
    }

    #[test]
    fn regex_literal_in_source() {
        // parse_regular_expression is false, meaning the regex pattern body is
        // not deeply parsed. However, the regex literal itself is still
        // recognized as a RegExpLiteral node in the AST (oxc always tokenizes
        // regex literals; the flag only controls sub-pattern parsing).
        let source = "var re = /foo/g;";
        let results = query(source, "Literal", JsSourceType::Js);
        assert!(
            results.iter().any(|r| r.text == "/foo/g"),
            "should find regex literal, got: {:?}",
            results
        );
    }

    #[test]
    fn parse_error_returns_empty() {
        // Syntax errors should return empty, not partial AST results
        let source = "var x = ;; }{";
        let results = query(source, "*", JsSourceType::Js);
        assert!(
            results.is_empty(),
            "parse errors should return empty, got: {:?}",
            results
        );
    }
}
