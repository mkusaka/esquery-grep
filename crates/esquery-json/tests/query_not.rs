mod common;

use common::*;
use esquery_json::query;

#[test]
fn not_conditional() {
    let ast = load("conditional");
    let matches = query(&ast, ":not(Literal)");
    assert_eq!(28, matches.len());
}

#[test]
fn not_for_loop() {
    let ast = load("forLoop");
    let matches = query(&ast, r#":not([name="x"])"#);
    assert_eq!(18, matches.len());
}

#[test]
fn not_simple_function() {
    let ast = load("simpleFunction");
    let matches = query(&ast, ":not(*)");
    assert_eq!(0, matches.len());
}

#[test]
fn not_simple_program() {
    let ast = load("simpleProgram");
    let matches = query(&ast, ":not(Identifier, IfStatement)");
    assert_eq!(15, matches.len());
}

#[test]
fn not_small_program() {
    let program = serde_json::json!({
        "type": "Program",
        "body": [{
            "type": "VariableDeclaration",
            "declarations": [{
                "type": "VariableDeclarator",
                "id": { "type": "Identifier", "name": "x" },
                "init": { "type": "Literal", "value": 1, "raw": "1" }
            }],
            "kind": "var"
        }]
    });
    let matches = query(&program, ":not([value=1])");
    assert_includes(
        &matches,
        &[
            &program,
            nav(&program, "body.0"),
            nav(&program, "body.0.declarations.0"),
            nav(&program, "body.0.declarations.0.id"),
        ],
    );
}
