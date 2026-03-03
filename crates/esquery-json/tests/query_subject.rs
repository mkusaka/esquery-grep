mod common;

use common::*;
use esquery_json::query;

#[test]
fn subject_type() {
    let ast = load("conditional");
    let matches = query(&ast, "!IfStatement Identifier");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0"),
            nav(&ast, "body.1"),
            nav(&ast, "body.1.alternate"),
        ],
    );
}

#[test]
fn subject_wildcard() {
    let ast = load("forLoop");
    let matches = query(&ast, r#"!* > [name="foo"]"#);
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.test.right"),
            nav(&ast, "body.0.body.body.0.expression.callee"),
        ],
    );
}

#[test]
fn subject_nth_child() {
    let ast = load("simpleFunction");
    let matches = query(&ast, r#"!:nth-child(1) [name="y"]"#);
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0"),
            nav(&ast, "body.0.body.body.0"),
            nav(&ast, "body.0.body.body.0.declarations.0"),
        ],
    );
}

#[test]
fn subject_nth_last_child() {
    let ast = load("simpleProgram");
    let matches = query(&ast, r#"!:nth-last-child(1) [name="y"]"#);
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.3"),
            nav(&ast, "body.1.declarations.0"),
            nav(&ast, "body.3.consequent.body.0"),
        ],
    );
}

#[test]
fn subject_attribute_literal() {
    let ast = load("simpleProgram");
    let matches = query(&ast, r#"![test] [name="y"]"#);
    assert_includes(&matches, &[nav(&ast, "body.3")]);
}

#[test]
fn subject_attribute_type() {
    let ast = load("nestedFunctions");
    let matches = query(&ast, "![generator=type(boolean)] > BlockStatement");
    assert_includes(
        &matches,
        &[nav(&ast, "body.0"), nav(&ast, "body.0.body.body.1")],
    );
}

#[test]
fn subject_attribute_regexp() {
    let ast = load("conditional");
    let matches = query(&ast, r#"![operator=/=+/] > [name="x"]"#);
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.test"),
            nav(&ast, "body.0.alternate.body.0.expression"),
            nav(&ast, "body.1.test.left.left"),
        ],
    );
}

#[test]
fn subject_field() {
    let ast = load("forLoop");
    let matches = query(&ast, "!.test");
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}

#[test]
fn subject_matches() {
    let ast = load("forLoop");
    let matches = query(&ast, r#"!:matches(*) > [name="foo"]"#);
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.test.right"),
            nav(&ast, "body.0.body.body.0.expression.callee"),
        ],
    );
}

#[test]
fn subject_not() {
    let ast = load("nestedFunctions");
    let matches = query(&ast, r#"!:not(BlockStatement) > [name="foo"]"#);
    assert_includes(&matches, &[nav(&ast, "body.0")]);
}

#[test]
fn subject_compound_attributes() {
    let ast = load("conditional");
    let matches = query(&ast, r#"![left.name="x"][right.value=1]"#);
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}

#[test]
fn subject_descendant_right() {
    let ast = load("forLoop");
    let matches = query(&ast, "* !AssignmentExpression");
    assert_includes(&matches, &[nav(&ast, "body.0.init")]);
}

#[test]
fn subject_child_right() {
    let ast = load("forLoop");
    let matches = query(&ast, "* > !AssignmentExpression");
    assert_includes(&matches, &[nav(&ast, "body.0.init")]);
}

#[test]
fn subject_sibling_left() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "!VariableDeclaration ~ IfStatement");
    assert_includes(&matches, &[nav(&ast, "body.0"), nav(&ast, "body.1")]);
}

#[test]
fn subject_sibling_right() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "!VariableDeclaration ~ !IfStatement");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0"),
            nav(&ast, "body.1"),
            nav(&ast, "body.3"),
        ],
    );
}

#[test]
fn subject_adjacent_right() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "!VariableDeclaration + !ExpressionStatement");
    assert_includes(&matches, &[nav(&ast, "body.1"), nav(&ast, "body.2")]);
}

#[test]
fn multiple_adjacent_siblings() {
    let ast = load("bigArray");
    let matches = query(&ast, "Identifier + Identifier");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.expression.elements.4"),
            nav(&ast, "body.0.expression.elements.8"),
        ],
    );
    assert_eq!(2, matches.len());
}

#[test]
fn multiple_siblings() {
    let ast = load("bigArray");
    let matches = query(&ast, "Identifier ~ Identifier");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.expression.elements.4"),
            nav(&ast, "body.0.expression.elements.7"),
            nav(&ast, "body.0.expression.elements.8"),
        ],
    );
    assert_eq!(3, matches.len());
}
