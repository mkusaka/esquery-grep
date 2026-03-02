mod common;

use common::*;
use esquery_json::query;

#[test]
fn compound_two_attributes() {
    let ast = load("conditional");
    let matches = query(&ast, r#"[left.name="x"][right.value=1]"#);
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}

#[test]
fn compound_type_and_pseudo() {
    let ast = load("conditional");
    let matches = query(&ast, r#"[left.name="x"]:matches(*)"#);
    assert_includes(&matches, &[nav(&ast, "body.0.test")]);
}
