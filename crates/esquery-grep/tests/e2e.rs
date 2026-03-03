use std::process::Command;

fn eg() -> Command {
    Command::new(env!("CARGO_BIN_EXE_eg"))
}

fn fixtures() -> String {
    format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"))
}

fn run(args: &[&str]) -> (String, String, i32) {
    let output = eg().args(args).output().expect("failed to run eg");
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let code = output.status.code().unwrap_or(-1);
    (stdout, stderr, code)
}

#[test]
fn identifier_in_js() {
    let pattern = format!("{}/app.js", fixtures());
    let (stdout, _, code) = run(&[&pattern, "Identifier"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("app.js:1:10: greet"));
    assert!(stdout.contains("app.js:1:16: name"));
    assert!(stdout.contains("app.js:7:5: x"));
}

#[test]
fn binary_expression() {
    let pattern = format!("{}/app.js", fixtures());
    let (stdout, _, code) = run(&[&pattern, "BinaryExpression"]);
    assert_eq!(code, 0);
    assert!(stdout.contains(r#"app.js:2:19: "Hello, " + name"#));
    assert!(stdout.contains("app.js:8:5: x > 10"));
}

#[test]
fn has_selector() {
    let pattern = format!("{}/app.js", fixtures());
    let (stdout, _, code) = run(&[&pattern, "FunctionDeclaration:has(ReturnStatement)"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("app.js:1:1: function greet(name) {"));
}

#[test]
fn attribute_selector() {
    let pattern = format!("{}/app.js", fixtures());
    let (stdout, _, code) = run(&[&pattern, r#"Identifier[name="greet"]"#]);
    assert_eq!(code, 0);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 2, "should find 2 occurrences of greet");
    assert!(stdout.contains("app.js:1:10: greet"));
    assert!(stdout.contains("app.js:9:3: greet"));
}

#[test]
fn typescript_file() {
    let pattern = format!("{}/utils.ts", fixtures());
    let (stdout, _, code) = run(&[&pattern, "TSInterfaceDeclaration"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("utils.ts:1:1: interface Config {"));
}

#[test]
fn tsx_jsx_element() {
    let pattern = format!("{}/component.tsx", fixtures());
    let (stdout, _, code) = run(&[&pattern, "JSXElement"]);
    assert_eq!(code, 0);
    assert!(stdout.contains(r#"component.tsx:2:10: <div className="app">Hello</div>"#));
}

#[test]
fn glob_expansion() {
    let pattern = format!("{}/*", fixtures());
    let (stdout, _, code) = run(&[&pattern, "FunctionDeclaration"]);
    assert_eq!(code, 0);
    // Should find functions in all 3 files
    assert!(stdout.contains("app.js:"));
    assert!(stdout.contains("utils.ts:"));
    assert!(stdout.contains("component.tsx:"));
}

#[test]
fn no_match_returns_exit_1() {
    let pattern = format!("{}/app.js", fixtures());
    let (_, _, code) = run(&[&pattern, "WhileStatement"]);
    assert_eq!(code, 1);
}

#[test]
fn invalid_selector_returns_exit_1() {
    let pattern = format!("{}/app.js", fixtures());
    let (_, _, code) = run(&[&pattern, "[[[invalid"]);
    assert_eq!(code, 1);
}

#[test]
fn type_flag_override() {
    let pattern = format!("{}/utils.ts", fixtures());
    let (stdout, _, code) = run(&[&pattern, "Identifier", "--type", "ts"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("utils.ts:"));
}

#[test]
fn multiline_match_shows_first_line() {
    let pattern = format!("{}/app.js", fixtures());
    let (stdout, _, code) = run(&[&pattern, "FunctionDeclaration"]);
    assert_eq!(code, 0);
    // Multiline function should only show first line
    let line = stdout
        .lines()
        .find(|l| l.contains("function greet"))
        .unwrap();
    assert_eq!(line.matches('\n').count(), 0);
    assert!(line.contains("function greet(name) {"));
}

#[test]
fn skips_unknown_extensions() {
    let pattern = format!("{}/../Cargo.toml", fixtures());
    let (_, _, code) = run(&[&pattern, "*"]);
    // .toml is not a recognized extension, should be skipped → no match
    assert_eq!(code, 1);
}
