use assert_cmd::cargo::cargo_bin_cmd;
use assert_cmd::Command;
use predicates::prelude::*;

fn eg() -> Command {
    cargo_bin_cmd!("eg")
}

fn fixtures() -> String {
    format!("{}/tests/fixtures", env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn identifier_in_js() {
    eg().args([&format!("{}/app.js", fixtures()), "Identifier"])
        .assert()
        .success()
        .stdout(predicate::str::contains("app.js:1:10: greet"))
        .stdout(predicate::str::contains("app.js:1:16: name"))
        .stdout(predicate::str::contains("app.js:7:5: x"));
}

#[test]
fn binary_expression() {
    eg().args([&format!("{}/app.js", fixtures()), "BinaryExpression"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#"app.js:2:19: "Hello, " + name"#))
        .stdout(predicate::str::contains("app.js:8:5: x > 10"));
}

#[test]
fn has_selector() {
    eg().args([
        &format!("{}/app.js", fixtures()),
        "FunctionDeclaration:has(ReturnStatement)",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "app.js:1:1: function greet(name) {",
    ));
}

#[test]
fn attribute_selector() {
    eg().args([
        &format!("{}/app.js", fixtures()),
        r#"Identifier[name="greet"]"#,
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("app.js:1:10: greet"))
    .stdout(predicate::str::contains("app.js:9:3: greet"));
}

#[test]
fn typescript_file() {
    eg().args([
        &format!("{}/utils.ts", fixtures()),
        "TSInterfaceDeclaration",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("utils.ts:1:1: interface Config {"));
}

#[test]
fn tsx_jsx_element() {
    eg().args([&format!("{}/component.tsx", fixtures()), "JSXElement"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            r#"component.tsx:2:10: <div className="app">Hello</div>"#,
        ));
}

#[test]
fn glob_expansion() {
    eg().args([&format!("{}/*", fixtures()), "FunctionDeclaration"])
        .assert()
        .success()
        .stdout(predicate::str::contains("app.js:"))
        .stdout(predicate::str::contains("utils.ts:"))
        .stdout(predicate::str::contains("component.tsx:"));
}

#[test]
fn no_match_returns_exit_1() {
    eg().args([&format!("{}/app.js", fixtures()), "WhileStatement"])
        .assert()
        .code(1);
}

#[test]
fn invalid_selector_returns_exit_1() {
    eg().args([&format!("{}/app.js", fixtures()), "[[[invalid"])
        .assert()
        .code(1);
}

#[test]
fn type_flag_override() {
    eg().args([
        &format!("{}/utils.ts", fixtures()),
        "Identifier",
        "--type",
        "ts",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("utils.ts:"));
}

#[test]
fn multiline_match_shows_first_line() {
    eg().args([&format!("{}/app.js", fixtures()), "FunctionDeclaration"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "app.js:1:1: function greet(name) {",
        ));
}

#[test]
fn skips_unknown_extensions() {
    eg().args([&format!("{}/../Cargo.toml", fixtures()), "*"])
        .assert()
        .code(1);
}
