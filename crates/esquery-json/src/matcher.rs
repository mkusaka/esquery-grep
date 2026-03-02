use esquery_selector::*;
use fancy_regex::Regex;
use serde_json::Value;

use crate::{is_node, visitor_keys};

/// Check if a JSON AST node matches a selector given its ancestry.
/// `ancestry[0]` is the immediate parent, `ancestry[1]` is grandparent, etc.
pub fn matches_node(node: &Value, selector: &Selector, ancestry: &[&Value]) -> bool {
    match &selector.kind {
        SelectorKind::Wildcard => true,

        SelectorKind::Identifier(name) => {
            let name_lower = name.to_lowercase();
            node.get("type")
                .and_then(|t| t.as_str())
                .map_or(false, |t| t.to_lowercase() == name_lower)
        }

        SelectorKind::ExactNode => ancestry.is_empty(),

        SelectorKind::Field { name } => {
            let path: Vec<&str> = name.split('.').collect();
            if path.len() > ancestry.len() {
                return false;
            }
            let ancestor = ancestry.get(path.len() - 1).copied();
            match ancestor {
                Some(anc) => in_path(node, anc, &path, 0),
                None => false,
            }
        }

        SelectorKind::Attribute(attr) => match_attribute(node, attr),

        SelectorKind::Class(name) => match_class(node, name, ancestry),

        SelectorKind::Compound(sels) => {
            sels.iter().all(|s| matches_node(node, s, ancestry))
        }

        SelectorKind::Matches(sels) => {
            sels.iter().any(|s| matches_node(node, s, ancestry))
        }

        SelectorKind::Not(sels) => {
            !sels.iter().any(|s| matches_node(node, s, ancestry))
        }

        SelectorKind::Has(sels) => has_match(node, sels),

        SelectorKind::NthChild { index } => nth_child(node, ancestry, *index),

        SelectorKind::NthLastChild { index } => nth_child(node, ancestry, -(*index)),

        SelectorKind::Child { left, right } => {
            if !ancestry.is_empty() && matches_node(node, right, ancestry) {
                let parent = ancestry[0];
                let parent_ancestry = &ancestry[1..];
                matches_node(parent, left, parent_ancestry)
            } else {
                false
            }
        }

        SelectorKind::Descendant { left, right } => {
            if matches_node(node, right, ancestry) {
                for i in 0..ancestry.len() {
                    let anc_ancestry = &ancestry[i + 1..];
                    if matches_node(ancestry[i], left, anc_ancestry) {
                        return true;
                    }
                }
            }
            false
        }

        SelectorKind::Sibling { left, right } => {
            (matches_node(node, right, ancestry)
                && sibling_match(node, left, ancestry, Side::Left))
                || (left.subject
                    && matches_node(node, left, ancestry)
                    && sibling_match(node, right, ancestry, Side::Right))
        }

        SelectorKind::Adjacent { left, right } => {
            (matches_node(node, right, ancestry)
                && adjacent_match(node, left, ancestry, Side::Left))
                || (right.subject
                    && matches_node(node, left, ancestry)
                    && adjacent_match(node, right, ancestry, Side::Right))
        }
    }
}

// -- Attribute matching --

fn match_attribute(node: &Value, attr: &AttributeSelector) -> bool {
    let path: Vec<&str> = attr.name.split('.').collect();
    let val = get_path(node, &path);

    match (&attr.operator, &attr.value) {
        (None, None) => val.is_some(),

        (Some(op), Some(attr_val)) => {
            let Some(val) = val else {
                // Path doesn't exist → JS val is `undefined`.
                // Match JS semantics for != operators:
                //   != type(T)    : typeof undefined !== T → "undefined" !== T → true (unless T is "undefined")
                //   != "literal"  : `${undefined}` !== `${literal}` → "undefined" !== lit_str
                //   != /regex/    : typeof undefined !== 'string' → true
                //   == anything   : false
                //   <,<=,>,>=     : false
                return match (op, attr_val) {
                    (AttrOperator::NotEq, AttrValue::Type(type_name)) => {
                        type_name.as_str() != "undefined"
                    }
                    (AttrOperator::NotEq, AttrValue::Literal(lit)) => {
                        let lit_str = match lit {
                            AttrLiteral::String(s) => s.clone(),
                            AttrLiteral::Number(n) => format_js_number(*n),
                            AttrLiteral::Path(s) => s.clone(),
                        };
                        lit_str != "undefined"
                    }
                    (AttrOperator::NotEq, AttrValue::Regex(re)) => {
                        // JS: !regex.test(undefined) → !regex.test("undefined")
                        match build_regex(re) {
                            Some(r) => !r.is_match("undefined").unwrap_or(false),
                            None => false,
                        }
                    }
                    _ => false,
                };
            };
            let val = &val;
            match (op, attr_val) {
                (AttrOperator::Eq, AttrValue::Regex(re)) => {
                    // JS: typeof p === 'string' && regex.test(p)
                    let Some(s) = val.as_str() else {
                        return false;
                    };
                    match build_regex(re) {
                        Some(r) => r.is_match(s).unwrap_or(false),
                        None => false,
                    }
                }
                (AttrOperator::NotEq, AttrValue::Regex(re)) => {
                    // JS: !regex.test(getPath(node, path))
                    // RegExp.test() calls ToString() on its argument
                    let s = value_to_string(val);
                    match build_regex(re) {
                        Some(r) => !r.is_match(&s).unwrap_or(false),
                        None => false,
                    }
                }
                (AttrOperator::Eq, AttrValue::Literal(lit)) => {
                    literal_eq(val, lit)
                }
                (AttrOperator::NotEq, AttrValue::Literal(lit)) => {
                    !literal_eq(val, lit)
                }
                (AttrOperator::Eq, AttrValue::Type(type_name)) => {
                    js_typeof(val) == type_name.as_str()
                }
                (AttrOperator::NotEq, AttrValue::Type(type_name)) => {
                    js_typeof(val) != type_name.as_str()
                }
                (AttrOperator::Lt, AttrValue::Literal(lit)) => {
                    compare_js(val, lit).map_or(false, |ord| ord < 0.0)
                }
                (AttrOperator::Lte, AttrValue::Literal(lit)) => {
                    compare_js(val, lit).map_or(false, |ord| ord <= 0.0)
                }
                (AttrOperator::Gt, AttrValue::Literal(lit)) => {
                    compare_js(val, lit).map_or(false, |ord| ord > 0.0)
                }
                (AttrOperator::Gte, AttrValue::Literal(lit)) => {
                    compare_js(val, lit).map_or(false, |ord| ord >= 0.0)
                }
                _ => false,
            }
        }
        _ => false,
    }
}

/// Build a Rust regex from an esquery regex value.
/// Supports flags: i (case-insensitive), m (multiline), s (dotall).
/// Note: JS /u flag is parsed but has no effect in Rust — fancy-regex always
/// operates in Unicode mode. This means:
/// - \u{61} is always a Unicode escape (JS only does this with /u)
/// - . always matches a Unicode codepoint (JS matches UTF-16 code unit without /u)
/// These are known limitations of using a Rust regex engine vs JS RegExp.
pub(crate) fn build_regex(re: &RegexValue) -> Option<Regex> {
    let mut flags = String::new();
    if re.flags.contains('i') {
        flags.push('i');
    }
    if re.flags.contains('m') {
        flags.push('m');
    }
    if re.flags.contains('s') {
        flags.push('s');
    }
    // Note: 'u' flag is intentionally not mapped — Rust regex is always Unicode-aware
    let pattern = if flags.is_empty() {
        re.pattern.clone()
    } else {
        format!("(?{}){}", flags, re.pattern)
    };
    Regex::new(&pattern).ok()
}

/// Compare a JSON value to a literal using JS `==` string coercion semantics.
/// In esquery: `${selector.value.value}` === `${getPath(node, path)}`
fn literal_eq(val: &Value, lit: &AttrLiteral) -> bool {
    let val_str = value_to_string(val);
    let lit_str = match lit {
        AttrLiteral::String(s) => s.clone(),
        AttrLiteral::Number(n) => format_js_number(*n),
        AttrLiteral::Path(s) => s.clone(),
    };
    val_str == lit_str
}

/// Convert a JSON value to string like JS template literal `${value}`.
fn value_to_string(val: &Value) -> String {
    match val {
        Value::String(s) => s.clone(),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.to_string()
            } else if let Some(f) = n.as_f64() {
                format_js_number(f)
            } else {
                n.to_string()
            }
        }
        Value::Bool(b) => b.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(_) => {
            // JS: [1,2].toString() = "1,2"
            // JS: null/undefined in arrays become empty strings: [null].toString() = ""
            if let Some(arr) = val.as_array() {
                arr.iter()
                    .map(|v| if v.is_null() { String::new() } else { value_to_string(v) })
                    .collect::<Vec<_>>()
                    .join(",")
            } else {
                String::new()
            }
        }
        Value::Object(_) => "[object Object]".to_string(),
    }
}

/// Format a f64 like JavaScript would.
fn format_js_number(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        // Integer-like: "1" not "1.0"
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Get the JS typeof equivalent for a JSON value.
fn js_typeof(val: &Value) -> &'static str {
    match val {
        Value::String(_) => "string",
        Value::Number(_) => "number",
        Value::Bool(_) => "boolean",
        Value::Null => "object", // typeof null === "object" in JS
        Value::Array(_) => "object",
        Value::Object(_) => "object",
    }
}

/// Convert a JSON value to a number using JS `Number()` semantics.
/// - Number → as-is
/// - String → parse (supports decimal, hex 0x, binary 0b, octal 0o, Infinity)
/// - Bool → true=1, false=0
/// - Null → 0
/// - Array → toString() then Number() ([] → 0, [1] → 1, [1,2] → NaN)
/// - Object → NaN
fn js_to_number(val: &Value) -> Option<f64> {
    match val {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => js_parse_number(s),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::Null => Some(0.0),
        Value::Array(_) => {
            // JS: Number([]) = Number("") = 0, Number([1]) = Number("1") = 1
            let s = value_to_string(val);
            js_parse_number(&s)
        }
        Value::Object(_) => None,
    }
}

/// Parse a string to f64 using JS `Number()` semantics.
fn js_parse_number(s: &str) -> Option<f64> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Some(0.0); // JS: Number("") === 0
    }
    // Hex: 0x or 0X
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        return u64::from_str_radix(&trimmed[2..], 16).ok().map(|n| n as f64);
    }
    // Binary: 0b or 0B
    if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
        return u64::from_str_radix(&trimmed[2..], 2).ok().map(|n| n as f64);
    }
    // Octal: 0o or 0O
    if trimmed.starts_with("0o") || trimmed.starts_with("0O") {
        return u64::from_str_radix(&trimmed[2..], 8).ok().map(|n| n as f64);
    }
    // Decimal / Infinity / -Infinity
    // Rust f64::parse accepts "inf"/"infinity"/"INF" etc. but JS Number() only
    // accepts "Infinity"/"+Infinity"/"-Infinity" (case-sensitive).
    let result = trimmed.parse::<f64>().ok()?;
    if result.is_infinite() {
        match trimmed {
            "Infinity" | "+Infinity" | "-Infinity" => Some(result),
            _ => None,
        }
    } else {
        Some(result)
    }
}

/// Compare a JSON value to a literal using JS comparison semantics.
/// JS: `getPath(node, path) > selector.value.value`
/// - Number vs Number: numeric comparison
/// - String vs String: lexicographic comparison
/// - Mixed: JS coerces both to Number via ToNumber()
/// Returns a value where < 0 means val < lit, 0 means equal, > 0 means val > lit.
fn compare_js(val: &Value, lit: &AttrLiteral) -> Option<f64> {
    match lit {
        AttrLiteral::Number(n) => {
            // Literal is a number → JS coerces val to number
            let val_num = js_to_number(val)?;
            Some(val_num - n)
        }
        AttrLiteral::String(s) | AttrLiteral::Path(s) => {
            if let Some(val_str) = val.as_str() {
                // Both strings → lexicographic comparison
                Some(match val_str.cmp(s.as_str()) {
                    std::cmp::Ordering::Less => -1.0,
                    std::cmp::Ordering::Equal => 0.0,
                    std::cmp::Ordering::Greater => 1.0,
                })
            } else {
                // Non-string val vs string literal → JS coerces both to number
                let val_num = js_to_number(val)?;
                let lit_num = js_parse_number(s)?;
                Some(val_num - lit_num)
            }
        }
    }
}

// -- Path utilities --

/// Navigate a JSON object by a path of keys.
/// Returns an owned clone of the final value.
/// Supports `.length` on arrays (like JS array.length).
fn get_path(obj: &Value, keys: &[&str]) -> Option<Value> {
    let mut current = obj;
    for (i, &key) in keys.iter().enumerate() {
        // JS: if (obj == null) return obj;
        // When current is null (or undefined, which can't happen in JSON),
        // return null immediately — don't try to navigate further.
        if current.is_null() {
            return Some(Value::Null);
        }
        match current {
            Value::Object(map) => {
                current = map.get(key)?;
            }
            Value::Array(arr) => {
                if key == "length" {
                    let len_val = Value::Number(serde_json::Number::from(arr.len()));
                    if i + 1 == keys.len() {
                        return Some(len_val);
                    }
                    return None;
                }
                let idx: usize = key.parse().ok()?;
                current = arr.get(idx)?;
            }
            _ => return None,
        }
    }
    Some(current.clone())
}

/// Check if `node` can be reached from `ancestor` by following `path`.
fn in_path(node: &Value, ancestor: &Value, path: &[&str], from_index: usize) -> bool {
    let mut current = ancestor;
    for i in from_index..path.len() {
        let field = match current.get(path[i]) {
            Some(f) => f,
            None => return false,
        };
        if let Some(arr) = field.as_array() {
            return arr
                .iter()
                .any(|elem| in_path(node, elem, path, i + 1));
        }
        current = field;
    }
    std::ptr::eq(node, current)
}

// -- Class matching --

fn match_class(node: &Value, class_name: &str, ancestry: &[&Value]) -> bool {
    let node_type = match node.get("type").and_then(|t| t.as_str()) {
        Some(t) => t,
        None => return false,
    };

    match class_name.to_lowercase().as_str() {
        "statement" => {
            if node_type.ends_with("Statement") {
                return true;
            }
            // fallthrough: all declarations are statements
            node_type.ends_with("Declaration")
        }
        "declaration" => node_type.ends_with("Declaration"),
        "pattern" => {
            if node_type.ends_with("Pattern") {
                return true;
            }
            // fallthrough: all expressions are patterns (and identifiers)
            is_expression(node_type, ancestry)
        }
        "expression" => is_expression(node_type, ancestry),
        "function" => matches!(
            node_type,
            "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression"
        ),
        _ => false,
    }
}

fn is_expression(node_type: &str, ancestry: &[&Value]) -> bool {
    node_type.ends_with("Expression")
        || node_type.ends_with("Literal")
        || (node_type == "Identifier"
            && (ancestry.is_empty()
                || ancestry[0]
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map_or(true, |t| t != "MetaProperty")))
        || node_type == "MetaProperty"
}

// -- Sibling / Adjacent --

#[derive(Clone, Copy)]
enum Side {
    Left,
    Right,
}

fn sibling_match(node: &Value, matcher: &Selector, ancestry: &[&Value], side: Side) -> bool {
    let parent = match ancestry.first() {
        Some(p) => *p,
        None => return false,
    };

    for key in visitor_keys(parent) {
        let Some(list_prop) = parent.get(key) else {
            continue;
        };
        let Some(arr) = list_prop.as_array() else {
            continue;
        };
        let start_index = arr.iter().position(|elem| std::ptr::eq(elem, node));
        let Some(start_index) = start_index else {
            continue;
        };

        let (lower, upper) = match side {
            Side::Left => (0, start_index),
            Side::Right => (start_index + 1, arr.len()),
        };

        for k in lower..upper {
            if is_node(&arr[k]) && matches_node(&arr[k], matcher, ancestry) {
                return true;
            }
        }
    }
    false
}

fn adjacent_match(node: &Value, matcher: &Selector, ancestry: &[&Value], side: Side) -> bool {
    let parent = match ancestry.first() {
        Some(p) => *p,
        None => return false,
    };

    for key in visitor_keys(parent) {
        let Some(list_prop) = parent.get(key) else {
            continue;
        };
        let Some(arr) = list_prop.as_array() else {
            continue;
        };
        let idx = arr.iter().position(|elem| std::ptr::eq(elem, node));
        let Some(idx) = idx else { continue };

        match side {
            Side::Left => {
                if idx > 0 && is_node(&arr[idx - 1]) {
                    if matches_node(&arr[idx - 1], matcher, ancestry) {
                        return true;
                    }
                }
            }
            Side::Right => {
                if idx + 1 < arr.len() && is_node(&arr[idx + 1]) {
                    if matches_node(&arr[idx + 1], matcher, ancestry) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

// -- Nth-child --

/// Check if `node` is the `nth` child. Positive nth is 1-indexed from start,
/// negative nth is indexed from end (-1 = last).
fn nth_child(node: &Value, ancestry: &[&Value], nth: i32) -> bool {
    if nth == 0 {
        return false;
    }
    let parent = match ancestry.first() {
        Some(p) => *p,
        None => return false,
    };

    for key in visitor_keys(parent) {
        let Some(list_prop) = parent.get(key) else {
            continue;
        };
        let Some(arr) = list_prop.as_array() else {
            continue;
        };
        let idx = if nth < 0 {
            (arr.len() as i32 + nth) as usize
        } else {
            (nth - 1) as usize
        };
        if idx < arr.len() && std::ptr::eq(&arr[idx], node) {
            return true;
        }
    }
    false
}

// -- :has() sub-traversal --

fn has_match(node: &Value, selectors: &[Selector]) -> bool {
    let mut local_ancestry: Vec<&Value> = Vec::new();
    has_traverse(node, selectors, &mut local_ancestry)
}

fn has_traverse<'a>(
    node: &'a Value,
    selectors: &[Selector],
    ancestry: &mut Vec<&'a Value>,
) -> bool {
    if !is_node(node) {
        return false;
    }

    for sel in selectors {
        if matches_node(node, sel, ancestry) {
            return true;
        }
    }

    ancestry.insert(0, node);
    for key in visitor_keys(node) {
        match node.get(key) {
            Some(val) if val.is_array() => {
                for elem in val.as_array().unwrap() {
                    if is_node(elem) && has_traverse(elem, selectors, ancestry) {
                        ancestry.remove(0);
                        return true;
                    }
                }
            }
            Some(val) if is_node(val) => {
                if has_traverse(val, selectors, ancestry) {
                    ancestry.remove(0);
                    return true;
                }
            }
            _ => {}
        }
    }
    ancestry.remove(0);
    false
}
