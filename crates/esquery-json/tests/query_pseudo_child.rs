mod common;

use common::*;
use esquery_json::query;

#[test]
fn first_child_conditional() {
    let ast = load("conditional");
    let matches = query(&ast, ":first-child");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.0.consequent.body.0"),
        nav(&ast, "body.0.alternate.body.0"),
        nav(&ast, "body.1.consequent.body.0"),
        nav(&ast, "body.1.alternate.consequent.body.0"),
    ]);
}

#[test]
fn last_child_conditional() {
    let ast = load("conditional");
    let matches = query(&ast, ":last-child");
    assert_includes(&matches, &[
        nav(&ast, "body.1"),
        nav(&ast, "body.0.consequent.body.0"),
        nav(&ast, "body.0.alternate.body.0"),
        nav(&ast, "body.1.consequent.body.0"),
        nav(&ast, "body.1.alternate.consequent.body.0"),
    ]);
}

#[test]
fn nth_child_conditional() {
    let ast = load("conditional");

    let matches = query(&ast, ":nth-child(2)");
    assert_includes(&matches, &[nav(&ast, "body.1")]);

    let matches = query(&ast, ":nth-last-child(2)");
    assert_includes(&matches, &[nav(&ast, "body.0")]);
}

#[test]
fn nth_child_multiple_digits() {
    let ast = load("conditionalLong");
    let matches = query(&ast, ":nth-child(10)");
    assert_includes(&matches, &[nav(&ast, "body.9")]);
}

#[test]
fn nth_last_child_multiple_digits() {
    let ast = load("conditionalLong");
    let matches = query(&ast, ":nth-last-child(10)");
    assert_includes(&matches, &[nav(&ast, "body.1")]);
}

#[test]
fn first_child_for_loop() {
    let ast = load("forLoop");
    let matches = query(&ast, ":first-child");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.0.body.body.0"),
    ]);
}

#[test]
fn last_child_for_loop() {
    let ast = load("forLoop");
    let matches = query(&ast, ":last-child");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.0.body.body.0"),
    ]);
}

#[test]
fn nth_child_for_loop() {
    let ast = load("forLoop");
    let matches = query(&ast, ":nth-last-child(1)");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.0.body.body.0"),
    ]);
}

#[test]
fn first_child_simple_function() {
    let ast = load("simpleFunction");
    let matches = query(&ast, ":first-child");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.0.params.0"),
        nav(&ast, "body.0.body.body.0"),
        nav(&ast, "body.0.body.body.0.declarations.0"),
    ]);
}

#[test]
fn last_child_simple_function() {
    let ast = load("simpleFunction");
    let matches = query(&ast, ":last-child");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.0.params.1"),
        nav(&ast, "body.0.body.body.2"),
        nav(&ast, "body.0.body.body.0.declarations.0"),
    ]);
}

#[test]
fn nth_child_simple_function() {
    let ast = load("simpleFunction");

    let matches = query(&ast, ":nth-child(2)");
    assert_includes(&matches, &[
        nav(&ast, "body.0.params.1"),
        nav(&ast, "body.0.body.body.1"),
    ]);

    let matches = query(&ast, ":nth-child(3)");
    assert_includes(&matches, &[nav(&ast, "body.0.body.body.2")]);

    let matches = query(&ast, ":nth-last-child(2)");
    assert_includes(&matches, &[
        nav(&ast, "body.0.params.0"),
        nav(&ast, "body.0.body.body.1"),
    ]);
}

#[test]
fn first_child_simple_program() {
    let ast = load("simpleProgram");
    let matches = query(&ast, ":first-child");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.0.declarations.0"),
        nav(&ast, "body.1.declarations.0"),
        nav(&ast, "body.3.consequent.body.0"),
    ]);
}

#[test]
fn last_child_simple_program() {
    let ast = load("simpleProgram");
    let matches = query(&ast, ":last-child");
    assert_includes(&matches, &[
        nav(&ast, "body.3"),
        nav(&ast, "body.0.declarations.0"),
        nav(&ast, "body.1.declarations.0"),
        nav(&ast, "body.3.consequent.body.0"),
    ]);
}

#[test]
fn nth_child_simple_program() {
    let ast = load("simpleProgram");

    let matches = query(&ast, ":nth-child(2)");
    assert_includes(&matches, &[nav(&ast, "body.1")]);

    let matches = query(&ast, ":nth-child(3)");
    assert_includes(&matches, &[nav(&ast, "body.2")]);

    let matches = query(&ast, ":nth-last-child(2)");
    assert_includes(&matches, &[nav(&ast, "body.2")]);
}
