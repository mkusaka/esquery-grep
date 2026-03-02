mod common;

use common::*;
use esquery_json::query;

#[test]
fn wildcard_empty() {
    let ast = load("conditional");
    let matches = query(&ast, "");
    assert_eq!(0, matches.len());
}

#[test]
fn wildcard_conditional() {
    let ast = load("conditional");
    let matches = query(&ast, "*");
    assert_eq!(35, matches.len());
}

#[test]
fn wildcard_for_loop() {
    let ast = load("forLoop");
    let matches = query(&ast, "*");
    assert_eq!(18, matches.len());
}

#[test]
fn wildcard_simple_function() {
    let ast = load("simpleFunction");
    let matches = query(&ast, "*");
    assert_eq!(17, matches.len());
}

#[test]
fn wildcard_simple_program() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "*");
    assert_eq!(22, matches.len());
}

#[test]
fn wildcard_small_program() {
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
    let matches = query(&program, "*");
    assert_includes(
        &matches,
        &[
            &program,
            nav(&program, "body.0"),
            nav(&program, "body.0.declarations.0"),
            nav(&program, "body.0.declarations.0.id"),
            nav(&program, "body.0.declarations.0.init"),
        ],
    );
}
