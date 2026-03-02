mod common;

use common::*;
use esquery_json::query;

#[test]
fn type_conditional() {
    let ast = load("conditional");

    let matches = query(&ast, "Program");
    assert_includes(&matches, &[&ast]);

    let matches = query(&ast, "IfStatement");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.1"),
        nav(&ast, "body.1.alternate"),
    ]);

    let matches = query(&ast, "LogicalExpression");
    assert_includes(&matches, &[
        nav(&ast, "body.1.test"),
        nav(&ast, "body.1.test.left"),
    ]);

    let matches = query(&ast, "ExpressionStatement");
    assert_includes(&matches, &[
        nav(&ast, "body.0.consequent.body.0"),
        nav(&ast, "body.0.alternate.body.0"),
        nav(&ast, "body.1.consequent.body.0"),
        nav(&ast, "body.1.alternate.consequent.body.0"),
    ]);
}

#[test]
fn type_for_loop() {
    let ast = load("forLoop");

    let matches = query(&ast, "Program");
    assert_includes(&matches, &[&ast]);

    let matches = query(&ast, "ForStatement");
    assert_includes(&matches, &[nav(&ast, "body.0")]);

    let matches = query(&ast, "BinaryExpression");
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}

#[test]
fn type_simple_function() {
    let ast = load("simpleFunction");

    let matches = query(&ast, "Program");
    assert_includes(&matches, &[&ast]);

    let matches = query(&ast, "VariableDeclaration");
    assert_includes(&matches, &[nav(&ast, "body.0.body.body.0")]);

    let matches = query(&ast, "FunctionDeclaration");
    assert_includes(&matches, &[nav(&ast, "body.0")]);

    let matches = query(&ast, "ReturnStatement");
    assert_includes(&matches, &[nav(&ast, "body.0.body.body.2")]);
}

#[test]
fn type_simple_program() {
    let ast = load("simpleProgram");

    let matches = query(&ast, "Program");
    assert_includes(&matches, &[&ast]);

    let matches = query(&ast, "VariableDeclaration");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.1"),
    ]);

    let matches = query(&ast, "AssignmentExpression");
    assert_includes(&matches, &[
        nav(&ast, "body.2.expression"),
        nav(&ast, "body.3.consequent.body.0.expression"),
    ]);

    let matches = query(&ast, "Identifier");
    assert_includes(&matches, &[
        nav(&ast, "body.0.declarations.0.id"),
        nav(&ast, "body.1.declarations.0.id"),
        nav(&ast, "body.2.expression.left"),
        nav(&ast, "body.2.expression.right.left"),
        nav(&ast, "body.3.test"),
        nav(&ast, "body.3.consequent.body.0.expression.left"),
    ]);
}

#[test]
fn type_hash_prefix() {
    let ast = load("forLoop");

    let matches = query(&ast, "#Program");
    assert_includes(&matches, &[&ast]);

    let matches = query(&ast, "#ForStatement");
    assert_includes(&matches, &[nav(&ast, "body.0")]);

    let matches = query(&ast, "#BinaryExpression");
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}

#[test]
fn type_case_insensitive() {
    let ast = load("forLoop");

    let matches = query(&ast, "Program");
    assert_includes(&matches, &[&ast]);

    let matches = query(&ast, "forStatement");
    assert_includes(&matches, &[nav(&ast, "body.0")]);

    let matches = query(&ast, "binaryexpression");
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}
