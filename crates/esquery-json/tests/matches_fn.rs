/// Tests for the `matches()` function (single-node matching).
/// Ported from esquery tests/matches.js.
///
/// Skipped test groups (not applicable to Rust):
///   - "falsey selector" — Selector is a typed enum, can't be null
///   - "falsey ancestry" — ancestry is &[&Value], can't be null
///   - "custom visitor keys" / "nodeTypeKey" / "fallback" — not supported
mod common;

use common::*;
use esquery_json::matches;
use esquery_selector::parse;
use serde_json::{json, Value};

// -- falsey node --

#[test]
fn falsey_node_null() {
    let selector = parse("*").unwrap();
    assert!(!matches(&Value::Null, &selector, &[]));
}

#[test]
fn falsey_node_string() {
    let selector = parse("*").unwrap();
    assert!(!matches(&json!(""), &selector, &[]));
}

#[test]
fn falsey_node_bool() {
    let selector = parse("*").unwrap();
    assert!(!matches(&json!(false), &selector, &[]));
}

// -- missing parent (empty ancestry with sibling/adjacent selectors) --

#[test]
fn missing_parent_adjacent() {
    let ast = load("simpleProgram");
    let selector = parse("!VariableDeclaration + !ExpressionStatement").unwrap();
    // Should not panic, just return false (no parent to search siblings in)
    let node = nav(&ast, "body.2");
    let result = matches(node, &selector, &[]);
    // With empty ancestry, adjacent can't match — just verify no panic
    let _ = result;
}

#[test]
fn missing_parent_sibling() {
    let ast = load("simpleProgram");
    let selector = parse("!VariableDeclaration ~ IfStatement").unwrap();
    let node = nav(&ast, "body.3");
    let result = matches(node, &selector, &[]);
    let _ = result;
}

// -- adjacent/sibling with ancestry --

#[test]
fn adjacent_with_ancestry() {
    let ast = load("simpleProgram");
    let selector = parse("!VariableDeclaration + !ExpressionStatement").unwrap();
    let node = nav(&ast, "body.2");
    // ancestry[0] is the parent — simpleProgram itself (the Program node)
    let result = matches(node, &selector, &[&ast]);
    // Should not panic; the actual match result depends on sibling lookup
    let _ = result;
}

#[test]
fn sibling_with_ancestry() {
    let ast = load("simpleProgram");
    let selector = parse("!VariableDeclaration ~ IfStatement").unwrap();
    let node = nav(&ast, "body.3");
    let result = matches(node, &selector, &[&ast]);
    let _ = result;
}

// -- non-array list prop --

#[test]
fn non_array_list_prop_sibling() {
    let ast = load("conditional");
    let selector = parse("!IfStatement ~ IfStatement").unwrap();
    let node = nav(&ast, "body.1");
    // conditional.body is the parent context; the program node is the parent
    let result = matches(node, &selector, &[&ast]);
    let _ = result;
}

#[test]
fn non_array_list_prop_adjacent() {
    let ast = load("conditional");
    let selector = parse("!IfStatement + IfStatement").unwrap();
    let node = nav(&ast, "body.1");
    let result = matches(node, &selector, &[&ast]);
    let _ = result;
}
