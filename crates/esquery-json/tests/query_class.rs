mod common;

use common::*;
use esquery_json::query;

#[test]
fn class_statement() {
    let ast = load("allClasses");
    let matches = query(&ast, ":statement");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0"),
            nav(&ast, "body.0.body"),
            nav(&ast, "body.0.body.body.0"),
            nav(&ast, "body.0.body.body.1"),
            nav(&ast, "body.0.body.body.2"),
            nav(&ast, "body.0.body.body.3"),
        ],
    );
    assert_eq!(6, matches.len());
}

#[test]
fn class_expression() {
    let ast = load("allClasses");
    let matches = query(&ast, ":Expression");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.id"),
            nav(&ast, "body.0.body.body.0.expression"),
            nav(&ast, "body.0.body.body.0.expression.left.elements.0"),
            nav(&ast, "body.0.body.body.0.expression.right"),
            nav(&ast, "body.0.body.body.0.expression.right.body"),
            nav(&ast, "body.0.body.body.1.expression"),
            nav(&ast, "body.0.body.body.2.expression"),
            nav(&ast, "body.0.body.body.3.expression"),
            nav(&ast, "body.0.body.body.3.expression.expressions.0"),
        ],
    );
    assert_eq!(9, matches.len());
}

#[test]
fn class_function() {
    let ast = load("allClasses");
    let matches = query(&ast, ":FUNCTION");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0"),
            nav(&ast, "body.0.body.body.0.expression.right"),
        ],
    );
    assert_eq!(2, matches.len());
}

#[test]
fn class_declaration() {
    let ast = load("allClasses");
    let matches = query(&ast, ":declaratioN");
    assert_includes(&matches, &[nav(&ast, "body.0")]);
    assert_eq!(1, matches.len());
}

#[test]
fn class_pattern() {
    let ast = load("allClasses");
    let matches = query(&ast, ":paTTern");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.id"),
            nav(&ast, "body.0.body.body.0.expression"),
            nav(&ast, "body.0.body.body.0.expression.left"),
            nav(&ast, "body.0.body.body.0.expression.left.elements.0"),
            nav(&ast, "body.0.body.body.0.expression.right"),
            nav(&ast, "body.0.body.body.0.expression.right.body"),
            nav(&ast, "body.0.body.body.1.expression"),
            nav(&ast, "body.0.body.body.2.expression"),
            nav(&ast, "body.0.body.body.3.expression"),
            nav(&ast, "body.0.body.body.3.expression.expressions.0"),
        ],
    );
    assert_eq!(10, matches.len());
}
