mod common;

use common::*;
use esquery_json::query;

#[test]
fn field_single() {
    let ast = load("conditional");
    let matches = query(&ast, ".test");
    assert_includes(&matches, &[
        nav(&ast, "body.0.test"),
        nav(&ast, "body.1.test"),
        nav(&ast, "body.1.alternate.test"),
    ]);
}

#[test]
fn field_sequence() {
    let ast = load("simpleProgram");
    let matches = query(&ast, ".declarations.init");
    assert_includes(&matches, &[
        nav(&ast, "body.0.declarations.0.init"),
        nav(&ast, "body.1.declarations.0.init"),
    ]);
}

#[test]
fn field_sequence_long() {
    let ast = load("simpleProgram");
    let matches = query(&ast, ".body.declarations.init");
    assert_includes(&matches, &[
        nav(&ast, "body.0.declarations.0.init"),
        nav(&ast, "body.1.declarations.0.init"),
    ]);
}
