mod common;

use common::*;
use esquery_json::query;

#[test]
fn descendant_conditional() {
    let ast = load("conditional");
    let matches = query(&ast, "Program IfStatement");
    assert_includes(&matches, &[
        nav(&ast, "body.0"),
        nav(&ast, "body.1"),
        nav(&ast, "body.1.alternate"),
    ]);
}

#[test]
fn descendant_includes_ancestor_in_search() {
    let ast = load("conditional");

    // Compound selector (no space) - matches Identifier nodes with name=x
    let matches = query(&ast, "Identifier[name=x]");
    assert_eq!(4, matches.len());

    // Descendant selector (with space) - matches [name=x] nodes that are descendants of Identifier
    let matches = query(&ast, "Identifier [name=x]");
    assert_eq!(0, matches.len());

    let matches = query(&ast, "BinaryExpression [name=x]");
    assert_eq!(2, matches.len());

    let matches = query(&ast, "AssignmentExpression [name=x]");
    assert_eq!(1, matches.len());
}
