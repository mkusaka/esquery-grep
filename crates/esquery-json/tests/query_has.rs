mod common;

use common::*;
use esquery_json::query;

#[test]
fn has_basic() {
    let ast = load("conditional");
    let matches = query(
        &ast,
        r#"ExpressionStatement:has([name="foo"][type="Identifier"])"#,
    );
    assert_eq!(1, matches.len());
}

#[test]
fn has_one_of() {
    let ast = load("conditional");
    let matches = query(
        &ast,
        r#"IfStatement:has(LogicalExpression [name="foo"], LogicalExpression [name="x"])"#,
    );
    assert_eq!(1, matches.len());
}

#[test]
fn has_chaining() {
    let ast = load("conditional");
    let matches = query(
        &ast,
        r#"BinaryExpression:has(Identifier[name="x"]):has(Literal[value="test"])"#,
    );
    assert_eq!(1, matches.len());
}

#[test]
fn has_nesting() {
    let ast = load("conditional");
    let matches = query(
        &ast,
        "Program:has(IfStatement:has(Literal[value=true], Literal[value=false]))",
    );
    assert_eq!(1, matches.len());
}

#[test]
fn has_non_matching() {
    let ast = load("conditional");
    let matches = query(&ast, r#":has([value="impossible"])"#);
    assert_eq!(0, matches.len());
}

#[test]
fn has_binary_op() {
    let ast = load("conditional");

    // Deep child - should not match since Identifier[name="x"] is not a direct child of IfStatement
    let matches = query(&ast, r#"IfStatement:has(> Identifier[name="x"])"#);
    assert_eq!(0, matches.len());

    // Shallow child with field selector
    let matches = query(
        &ast,
        r#"IfStatement:has(> LogicalExpression.test, > Identifier[name="x"])"#,
    );
    assert_eq!(1, matches.len());
}
