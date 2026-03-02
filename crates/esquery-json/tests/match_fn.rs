/// Tests for `query()` / `query_selector()` error handling.
/// Ported from esquery tests/match.js.
///
/// In JS, match.js tests that esquery.match() throws on invalid selector AST objects.
/// In Rust, Selector is a typed enum so invalid variants can't be constructed.
/// Instead, we verify that invalid selector *strings* produce empty results (no panic).
mod common;

use common::*;
use esquery_json::{query, query_selector};
use esquery_selector::parse;

#[test]
fn invalid_selector_string_returns_empty() {
    let ast = load("forLoop");
    // Completely invalid selector strings should return empty results
    let result = query(&ast, ":::badSelector");
    assert!(result.is_empty());
}

#[test]
fn empty_selector_returns_empty() {
    let ast = load("forLoop");
    let result = query(&ast, "");
    assert!(result.is_empty());
}

#[test]
fn whitespace_only_selector_returns_empty() {
    let ast = load("forLoop");
    let result = query(&ast, "   ");
    assert!(result.is_empty());
}

#[test]
fn query_selector_with_parsed_selector() {
    // Verify query_selector (the Rust equivalent of JS esquery.match) works
    let ast = load("forLoop");
    let selector = parse("ForStatement").unwrap();
    let result = query_selector(&ast, &selector);
    assert_eq!(result.len(), 1);
    assert_eq!(
        result[0].get("type").unwrap().as_str().unwrap(),
        "ForStatement"
    );
}
