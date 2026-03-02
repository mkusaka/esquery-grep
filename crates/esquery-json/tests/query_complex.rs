mod common;

use common::*;
use esquery_json::query;

#[test]
fn complex_child() {
    let ast = load("conditional");
    let matches = query(&ast, "IfStatement > BinaryExpression");
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}

#[test]
fn complex_three_types_child() {
    let ast = load("conditional");
    let matches = query(&ast, "IfStatement > BinaryExpression > Identifier");
    assert_includes(&matches, &[nav(&ast, "body.0.test.left")]);
}

#[test]
fn complex_descendant() {
    let ast = load("conditional");
    let matches = query(&ast, "IfStatement BinaryExpression");
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}

#[test]
fn complex_sibling() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "VariableDeclaration ~ IfStatement");
    assert_includes(&matches, &[nav(&ast, "body.3")]);
}

#[test]
fn complex_adjacent() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "VariableDeclaration + ExpressionStatement");
    assert_includes(&matches, &[nav(&ast, "body.2")]);
}

#[test]
fn complex_no_match_top_level() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "NonExistingNodeType > *");
    assert!(matches.is_empty());
}
