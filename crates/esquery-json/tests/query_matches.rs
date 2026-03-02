mod common;

use common::*;
use esquery_json::query;

#[test]
fn matches_conditional() {
    let ast = load("conditional");
    let matches = query(&ast, ":matches(IfStatement)");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.1.alternate"),
    ]);
}

#[test]
fn matches_for_loop() {
    let ast = load("forLoop");
    let matches = query(&ast, ":matches(BinaryExpression, MemberExpression)");
    assert_includes(&matches, &[
        nav(&ast, "body.0.test"),
        nav(&ast, "body.0.body.body.0.expression.callee"),
    ]);
}

#[test]
fn matches_simple_function() {
    let ast = load("simpleFunction");
    let matches = query(&ast, r#":matches([name="foo"], ReturnStatement)"#);
    assert_includes(&matches, &[
        nav(&ast, "body.0.id"),
        nav(&ast, "body.0.body.body.2"),
    ]);
}

#[test]
fn matches_simple_program() {
    let ast = load("simpleProgram");
    let matches = query(&ast, ":matches(AssignmentExpression, BinaryExpression)");
    assert_includes(&matches, &[
        nav(&ast, "body.2.expression"),
        nav(&ast, "body.3.consequent.body.0.expression"),
        nav(&ast, "body.2.expression.right"),
    ]);
}

#[test]
fn matches_implicit() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "AssignmentExpression, BinaryExpression, NonExistant");
    assert_includes(&matches, &[
        nav(&ast, "body.2.expression"),
        nav(&ast, "body.3.consequent.body.0.expression"),
        nav(&ast, "body.2.expression.right"),
    ]);
}

#[test]
fn matches_is_alias() {
    let ast = load("simpleFunction");
    let matches = query(&ast, r#":is([name="foo"], ReturnStatement)"#);
    assert_includes(&matches, &[
        nav(&ast, "body.0.id"),
        nav(&ast, "body.0.body.body.2"),
    ]);
}
