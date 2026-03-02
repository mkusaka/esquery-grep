/// Regression tests for issues found by Codex CLI review.
/// Each test reproduces a specific JS/Rust behavior mismatch.
use esquery_json::query;
use serde_json::json;

// -- Fix 1: `=` regex should not match non-string values --

#[test]
fn eq_regex_should_not_match_number() {
    // JS: typeof p === 'string' && regex.test(p)
    // A numeric value should NOT match [value=/.*/]
    let ast = json!({
        "type": "Literal",
        "value": 1,
        "raw": "1"
    });
    let matches = query(&ast, "[value=/.*/]");
    assert_eq!(matches.len(), 0, "regex should not match non-string value");
}

#[test]
fn eq_regex_should_match_string() {
    let ast = json!({
        "type": "Literal",
        "value": "hello",
        "raw": "\"hello\""
    });
    let matches = query(&ast, "[value=/.*/]");
    assert_eq!(matches.len(), 1, "regex should match string value");
}

// -- Fix 2: regex flags m/s --

#[test]
fn regex_dotall_flag() {
    // JS: /foo.bar/s should match "foo\nbar" (dotall: . matches \n)
    let ast = json!({
        "type": "Literal",
        "value": "foo\nbar",
        "raw": "\"foo\\nbar\""
    });
    let matches = query(&ast, "[value=/foo.bar/s]");
    assert_eq!(matches.len(), 1, "dotall flag should make . match newline");
}

#[test]
fn regex_multiline_flag() {
    // JS: /^bar/m should match "foo\nbar" (multiline: ^ matches after \n)
    let ast = json!({
        "type": "Literal",
        "value": "foo\nbar",
        "raw": "\"foo\\nbar\""
    });
    let matches = query(&ast, "[value=/^bar/m]");
    assert_eq!(
        matches.len(),
        1,
        "multiline flag should make ^ match after newline"
    );
}

#[test]
fn regex_without_dotall_should_not_match_newline() {
    // Without /s, . should NOT match \n
    let ast = json!({
        "type": "Literal",
        "value": "foo\nbar",
        "raw": "\"foo\\nbar\""
    });
    let matches = query(&ast, "[value=/foo.bar/]");
    assert_eq!(
        matches.len(),
        0,
        "without dotall, . should not match newline"
    );
}

// -- Fix 3: get_path null vs undefined (null mid-path) --

#[test]
fn null_mid_path_neq_type_object() {
    // JS: getPath({id: null}, ['id', 'name']) returns null
    //     typeof null !== 'object' → false → FunctionExpression should NOT match
    let ast = json!({
        "type": "FunctionExpression",
        "id": null,
        "params": [],
        "body": {"type": "BlockStatement", "body": []}
    });
    let matches = query(&ast, "[id.name!=type(object)]");
    // FunctionExpression: id is null → get_path(['id','name']) → null → typeof null = "object"
    //   "object" != "object" → false → should NOT match
    // BlockStatement: no "id" field → get_path returns None (undefined)
    //   typeof undefined = "undefined" → "object" != "undefined" → true → should match
    let types: Vec<&str> = matches
        .iter()
        .map(|v| v.get("type").unwrap().as_str().unwrap())
        .collect();
    assert!(
        !types.contains(&"FunctionExpression"),
        "FunctionExpression should NOT match (id is null, null mid-path → typeof null = 'object')"
    );
    assert!(
        types.contains(&"BlockStatement"),
        "BlockStatement should match (no id field → undefined → typeof undefined != 'object')"
    );
}

// -- Fix 4: != regex — JS uses RegExp.test() which stringifies its argument --

#[test]
fn neq_regex_missing_path_non_matching_pattern() {
    // JS: !/(x|y)/.test(undefined) → !/(x|y)/.test("undefined") → !"undefined" contains x or y → !false → true
    let ast = json!({
        "type": "Literal",
        "value": 1,
        "raw": "1"
    });
    let matches = query(&ast, "[name!=/x|y/]");
    assert_eq!(
        matches.len(),
        1,
        "!= regex: 'undefined' does not match /x|y/ → true"
    );
}

#[test]
fn neq_regex_missing_path_matching_pattern() {
    // JS: !/undefined/.test(undefined) → !/undefined/.test("undefined") → !true → false
    let ast = json!({
        "type": "Literal",
        "value": 1,
        "raw": "1"
    });
    let matches = query(&ast, "[name!=/undefined/]");
    assert_eq!(
        matches.len(),
        0,
        "!= regex: 'undefined' matches /undefined/ → false"
    );
}

#[test]
fn neq_regex_number_value_matching_pattern() {
    // JS: !/\d+/.test(123) → !/\d+/.test("123") → !true → false
    let ast = json!({
        "type": "Literal",
        "value": 123,
        "raw": "123"
    });
    let matches = query(&ast, r"[value!=/\d+/]");
    assert_eq!(matches.len(), 0, "!= regex: '123' matches /\\d+/ → false");
}

#[test]
fn neq_regex_existing_string_that_matches_pattern() {
    // If name matches the regex, != should return false
    let ast = json!({
        "type": "Identifier",
        "name": "x"
    });
    let matches = query(&ast, "[name!=/x|y/]");
    assert_eq!(
        matches.len(),
        0,
        "!= regex should not match when name matches the pattern"
    );
}

#[test]
fn neq_regex_existing_string_that_does_not_match_pattern() {
    // If name does NOT match the regex, != should return true
    let ast = json!({
        "type": "Identifier",
        "name": "z"
    });
    let matches = query(&ast, "[name!=/x|y/]");
    assert_eq!(
        matches.len(),
        1,
        "!= regex should match when name doesn't match the pattern"
    );
}

// -- Fix 5: comparison operators should support string comparison --

#[test]
fn compare_gt_string() {
    // JS: "z" > "a" → true (lexicographic)
    let ast = json!({
        "type": "Identifier",
        "name": "z"
    });
    let matches = query(&ast, r#"[name>"a"]"#);
    assert_eq!(matches.len(), 1, "string 'z' > 'a' should be true");
}

#[test]
fn compare_lt_string() {
    // JS: "a" < "z" → true (lexicographic)
    let ast = json!({
        "type": "Identifier",
        "name": "a"
    });
    let matches = query(&ast, r#"[name<"z"]"#);
    assert_eq!(matches.len(), 1, "string 'a' < 'z' should be true");
}

#[test]
fn compare_gte_string_equal() {
    // JS: "abc" >= "abc" → true
    let ast = json!({
        "type": "Identifier",
        "name": "abc"
    });
    let matches = query(&ast, r#"[name>="abc"]"#);
    assert_eq!(matches.len(), 1, "string 'abc' >= 'abc' should be true");
}

#[test]
fn compare_gt_string_false() {
    // JS: "a" > "z" → false
    let ast = json!({
        "type": "Identifier",
        "name": "a"
    });
    let matches = query(&ast, r#"[name>"z"]"#);
    assert_eq!(matches.len(), 0, "string 'a' > 'z' should be false");
}

// -- Fix 6: comparison with JS type coercion (string/bool/null → number) --

#[test]
fn compare_string_value_gt_number_literal() {
    // JS: "2" > 1 → Number("2") > 1 → 2 > 1 → true
    let ast = json!({
        "type": "Literal",
        "value": "2",
        "raw": "\"2\""
    });
    let matches = query(&ast, "[value>1]");
    assert_eq!(
        matches.len(),
        1,
        "string '2' > 1 should be true (JS coercion)"
    );
}

#[test]
fn compare_bool_value_gt_zero() {
    // JS: true > 0 → Number(true) > 0 → 1 > 0 → true
    let ast = json!({
        "type": "Literal",
        "value": true,
        "raw": "true"
    });
    let matches = query(&ast, "[value>0]");
    assert_eq!(
        matches.len(),
        1,
        "true > 0 should be true (JS coercion: true=1)"
    );
}

#[test]
fn compare_null_value_gte_zero() {
    // JS: null >= 0 → Number(null) >= 0 → 0 >= 0 → true
    let ast = json!({
        "type": "Literal",
        "value": null,
        "raw": "null"
    });
    let matches = query(&ast, "[value>=0]");
    assert_eq!(
        matches.len(),
        1,
        "null >= 0 should be true (JS coercion: null=0)"
    );
}

#[test]
fn compare_false_value_gte_zero() {
    // JS: false >= 0 → Number(false) >= 0 → 0 >= 0 → true
    let ast = json!({
        "type": "Literal",
        "value": false,
        "raw": "false"
    });
    let matches = query(&ast, "[value>=0]");
    assert_eq!(
        matches.len(),
        1,
        "false >= 0 should be true (JS coercion: false=0)"
    );
}

// -- Fix 7: JS Number() conversion for hex/binary strings and arrays --

#[test]
fn compare_hex_string_gt_number() {
    // JS: Number("0x10") = 16, 16 > 15 → true
    let ast = json!({
        "type": "Literal",
        "value": "0x10"
    });
    let matches = query(&ast, "[value>15]");
    assert_eq!(
        matches.len(),
        1,
        "hex string '0x10' (=16) > 15 should be true"
    );
}

#[test]
fn compare_binary_string_gt_number() {
    // JS: Number("0b10") = 2, 2 > 1 → true
    let ast = json!({
        "type": "Literal",
        "value": "0b10"
    });
    let matches = query(&ast, "[value>1]");
    assert_eq!(
        matches.len(),
        1,
        "binary string '0b10' (=2) > 1 should be true"
    );
}

#[test]
fn compare_empty_array_gte_zero() {
    // JS: Number([]) = Number("") = 0, 0 >= 0 → true
    let ast = json!({
        "type": "Literal",
        "value": []
    });
    let matches = query(&ast, "[value>=0]");
    assert_eq!(matches.len(), 1, "[] >= 0 should be true (Number([])=0)");
}

#[test]
fn compare_single_elem_array_gte_one() {
    // JS: Number([1]) = Number("1") = 1, 1 >= 1 → true
    let ast = json!({
        "type": "Literal",
        "value": [1]
    });
    let matches = query(&ast, "[value>=1]");
    assert_eq!(matches.len(), 1, "[1] >= 1 should be true (Number([1])=1)");
}

// -- Fix 12: js_parse_number should reject "inf"/"infinity" (Rust-specific f64 parsing) --

#[test]
fn compare_inf_string_should_not_match() {
    // JS: Number("inf") = NaN, NaN > 1 → false
    // Rust f64::parse("inf") = Infinity, but JS doesn't accept "inf"
    let ast = json!({
        "type": "Literal",
        "value": "inf"
    });
    let matches = query(&ast, "[value>1]");
    assert_eq!(matches.len(), 0, "'inf' should be NaN in JS Number()");
}

#[test]
fn compare_infinity_string_should_match() {
    // JS: Number("Infinity") = Infinity, Infinity > 1 → true
    let ast = json!({
        "type": "Literal",
        "value": "Infinity"
    });
    let matches = query(&ast, "[value>1]");
    assert_eq!(
        matches.len(),
        1,
        "'Infinity' should be Infinity in JS Number()"
    );
}

// -- Fix 13: Array.toString() should treat null as empty string --

#[test]
fn compare_null_array_gte_zero() {
    // JS: [null].toString() = "" → Number("") = 0, 0 >= 0 → true
    let ast = json!({
        "type": "Literal",
        "value": [null]
    });
    let matches = query(&ast, "[value>=0]");
    assert_eq!(
        matches.len(),
        1,
        "[null] >= 0 should be true (Number([null])=0)"
    );
}

#[test]
fn compare_null_null_array_should_be_nan() {
    // JS: [null,null].toString() = "," → Number(",") = NaN, NaN >= 0 → false
    let ast = json!({
        "type": "Literal",
        "value": [null, null]
    });
    let matches = query(&ast, "[value>=0]");
    assert_eq!(
        matches.len(),
        0,
        "[null,null] >= 0 should be false (Number(',')=NaN)"
    );
}

// -- Fix 14: regex lookbehind support (fancy-regex) --

#[test]
fn regex_lookbehind() {
    // JS: /(?<=a)b/.test("ab") → true
    let ast = json!({
        "type": "Literal",
        "value": "ab"
    });
    let matches = query(&ast, "[value=/(?<=a)b/]");
    assert_eq!(matches.len(), 1, "lookbehind should be supported");
}

#[test]
fn regex_lookahead() {
    // JS: /a(?=b)/.test("ab") → true
    let ast = json!({
        "type": "Literal",
        "value": "ab"
    });
    let matches = query(&ast, "[value=/a(?=b)/]");
    assert_eq!(matches.len(), 1, "lookahead should be supported");
}

// -- Fix 15: visitor_keys should use estraverse.VisitorKeys --

#[test]
fn comments_should_not_be_traversed() {
    // JS: Program.comments is not in estraverse.VisitorKeys → not traversed
    let ast = json!({
        "type": "Program",
        "body": [{"type": "ExpressionStatement", "expression": {"type": "Literal", "value": 1, "raw": "1"}}],
        "sourceType": "script",
        "comments": [{"type": "Line", "value": "hello"}]
    });
    let matches = query(&ast, "*");
    let types: Vec<&str> = matches
        .iter()
        .map(|v| v.get("type").unwrap().as_str().unwrap())
        .collect();
    assert!(
        !types.contains(&"Line"),
        "comments should not be traversed (not in visitor keys)"
    );
    assert_eq!(
        types.len(),
        3,
        "should find Program, ExpressionStatement, Literal"
    );
}

// -- Fix 16: unknown class name should cause parse failure --

#[test]
fn unknown_class_returns_empty() {
    // JS: :foobar throws "Unknown class name: foobar"
    // Rust: parse failure → query returns empty
    let ast = json!({
        "type": "Program",
        "body": [],
        "sourceType": "script"
    });
    let matches = query(&ast, ":foobar");
    assert_eq!(matches.len(), 0, "unknown class should not match any node");
}

#[test]
fn known_class_still_works() {
    let ast = json!({
        "type": "Program",
        "body": [{"type": "ExpressionStatement", "expression": {"type": "Literal", "value": 1, "raw": "1"}}],
        "sourceType": "script"
    });
    let matches = query(&ast, ":statement");
    assert!(!matches.is_empty(), ":statement should match statements");
}

// -- Fix 17: PropertyDefinition should not traverse decorators --

#[test]
fn property_definition_decorators_not_traversed() {
    // estraverse.VisitorKeys.PropertyDefinition = ["key", "value"]
    // "decorators" is not in visitor keys → should not be traversed
    let ast = json!({
        "type": "Program",
        "body": [{
            "type": "ClassDeclaration",
            "id": {"type": "Identifier", "name": "C"},
            "superClass": null,
            "body": {"type": "ClassBody", "body": [{
                "type": "PropertyDefinition",
                "key": {"type": "Identifier", "name": "x"},
                "value": {"type": "Literal", "value": 1, "raw": "1"},
                "computed": false,
                "static": false,
                "decorators": [{"type": "Identifier", "name": "dec"}]
            }]}
        }],
        "sourceType": "script"
    });
    let matches = query(&ast, "*");
    let _types: Vec<&str> = matches
        .iter()
        .map(|v| v.get("type").unwrap().as_str().unwrap())
        .collect();
    // "dec" decorator should not appear since decorators is not in PropertyDefinition visitor keys
    let decorator_count = matches
        .iter()
        .filter(|v| v.get("name").and_then(|n| n.as_str()) == Some("dec"))
        .count();
    assert_eq!(
        decorator_count, 0,
        "decorators should not be traversed for PropertyDefinition"
    );
}

// -- Fix 18: invalid regex should cause parse/query failure --

#[test]
fn invalid_regex_pattern_returns_empty() {
    // JS: [name=/(/] throws "Invalid regular expression"
    let ast = json!({"type": "Identifier", "name": "x"});
    let matches = query(&ast, "[name=/(/]");
    assert_eq!(matches.len(), 0, "invalid regex should return empty");
}

#[test]
fn invalid_regex_in_matches_returns_empty() {
    // JS: :matches(Identifier, [name=/(/]) throws
    let ast = json!({
        "type": "Program",
        "body": [{"type": "Identifier", "name": "x"}],
        "sourceType": "script"
    });
    let matches = query(&ast, ":matches(Identifier, [name=/(/])");
    assert_eq!(
        matches.len(),
        0,
        "invalid regex in :matches should fail entire query"
    );
}

#[test]
fn duplicate_regex_flags_returns_empty() {
    // JS: [name=/x/ii] throws "Invalid flags supplied to RegExp constructor 'ii'"
    let ast = json!({"type": "Identifier", "name": "x"});
    let matches = query(&ast, "[name=/x/ii]");
    assert_eq!(matches.len(), 0, "duplicate flags should fail");
}

// -- Fix 19: validate_regexes should apply to query_selector() and matches() --

#[test]
fn query_selector_rejects_invalid_regex() {
    use esquery_selector::parse as parse_selector;

    let ast = json!({"type": "Identifier", "name": "x"});
    let selector = parse_selector(":matches(Identifier, [name=/(/])").unwrap();
    let matches = esquery_json::query_selector(&ast, &selector);
    assert_eq!(
        matches.len(),
        0,
        "query_selector should reject invalid regex"
    );
}

#[test]
fn matches_fn_rejects_invalid_regex() {
    use esquery_selector::parse as parse_selector;

    let node = json!({"type": "Identifier", "name": "x"});
    let selector = parse_selector(":matches(Identifier, [name=/(/])").unwrap();
    let result = esquery_json::matches(&node, &selector, &[]);
    assert!(!result, "matches() should return false for invalid regex");
}

// -- Fix 20: duplicate regex flags should not fall back to path_value --

#[test]
fn duplicate_regex_flags_not_parsed_as_path() {
    // JS: [name=/x/ii] throws at parse time. Must NOT match anything.
    let ast = json!({"type": "Identifier", "name": "/x/ii"});
    let matches = query(&ast, "[name=/x/ii]");
    assert_eq!(
        matches.len(),
        0,
        "duplicate flags should cause parse failure, not path match"
    );
}
