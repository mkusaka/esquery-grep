mod common;

use common::*;
use esquery_json::query;

// -- conditional fixture --

#[test]
fn attr_name_eq_string() {
    let ast = load("conditional");
    let matches = query(&ast, r#"[name="x"]"#);
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.test.left"),
            nav(&ast, "body.0.alternate.body.0.expression.left"),
            nav(&ast, "body.1.test.left.left.left"),
            nav(&ast, "body.1.test.right"),
        ],
    );
}

#[test]
fn attr_nested_path() {
    let ast = load("conditional");
    let matches = query(&ast, r#"[callee.name="foo"]"#);
    assert_includes(
        &matches,
        &[nav(&ast, "body.0.consequent.body.0.expression")],
    );
}

#[test]
fn attr_existence() {
    let ast = load("conditional");
    let matches = query(&ast, "[operator]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.test"),
            nav(&ast, "body.0.alternate.body.0.expression"),
            nav(&ast, "body.1.test"),
            nav(&ast, "body.1.test.left"),
            nav(&ast, "body.1.test.left.left"),
        ],
    );
}

#[test]
fn attr_boolean_value() {
    let ast = load("conditional");
    let matches = query(&ast, "[prefix=true]");
    assert_includes(
        &matches,
        &[nav(&ast, "body.1.consequent.body.0.expression.right")],
    );
}

// -- literal fixture --

#[test]
fn attr_literal_special_escapes() {
    let ast = load("literal");
    let matches = query(
        &ast,
        r"Literal[value='\b\f\n\r\t\v and just a \ back\slash']",
    );
    assert_includes(&matches, &[nav(&ast, "body.0.declarations.0.init")]);
}

#[test]
fn attr_literal_decimal() {
    let ast = load("literal");
    let matches = query(&ast, "Literal[value=21.35]");
    assert_includes(&matches, &[nav(&ast, "body.1.declarations.0.init")]);
}

#[test]
fn attr_literal_extra_whitespace() {
    let ast = load("literal");
    let matches = query(&ast, "Literal[value  =  21.35]");
    assert_includes(&matches, &[nav(&ast, "body.1.declarations.0.init")]);
}

#[test]
fn attr_literal_backslash() {
    let ast = load("literal");
    let matches = query(&ast, r#"Literal[value="\z"]"#);
    assert_includes(&matches, &[nav(&ast, "body.2.declarations.0.init")]);
}

#[test]
fn attr_literal_backslash_after_beginning() {
    let ast = load("literal");
    let matches = query(&ast, r#"Literal[value="abc\z"]"#);
    assert_includes(&matches, &[nav(&ast, "body.3.declarations.0.init")]);
}

// -- forLoop fixture --

#[test]
fn attr_for_loop_operator() {
    let ast = load("forLoop");

    let matches = query(&ast, r#"[operator="="]"#);
    assert_includes(&matches, &[nav(&ast, "body.0.init")]);

    let matches = query(&ast, r#"[object.name="foo"]"#);
    assert_includes(&matches, &[nav(&ast, "body.0.test.right")]);

    let matches = query(&ast, "[operator]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.init"),
            nav(&ast, "body.0.test"),
            nav(&ast, "body.0.update"),
        ],
    );
}

// -- simpleFunction fixture --

#[test]
fn attr_simple_function() {
    let ast = load("simpleFunction");

    let matches = query(&ast, r#"[kind="var"]"#);
    assert_includes(&matches, &[nav(&ast, "body.0.body.body.0")]);

    let matches = query(&ast, r#"[id.name="foo"]"#);
    assert_includes(&matches, &[nav(&ast, "body.0")]);

    let matches = query(&ast, "[left]");
    assert_includes(
        &matches,
        &[nav(&ast, "body.0.body.body.0.declarations.0.init")],
    );
}

// -- simpleProgram fixture --

#[test]
fn attr_simple_program() {
    let ast = load("simpleProgram");

    let matches = query(&ast, r#"[kind="var"]"#);
    assert_includes(&matches, &[nav(&ast, "body.0"), nav(&ast, "body.1")]);

    let matches = query(&ast, r#"[id.name="y"]"#);
    assert_includes(&matches, &[nav(&ast, "body.1.declarations.0")]);

    let matches = query(&ast, "[body]");
    assert_includes(&matches, &[&ast, nav(&ast, "body.3.consequent")]);
}

// -- regexp --

#[test]
fn attr_conditional_regexp() {
    let ast = load("conditional");
    let matches = query(&ast, "[name=/x|foo/]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.test.left"),
            nav(&ast, "body.0.consequent.body.0.expression.callee"),
            nav(&ast, "body.0.alternate.body.0.expression.left"),
        ],
    );
}

#[test]
fn attr_simple_function_regexp() {
    let ast = load("simpleFunction");
    let matches = query(&ast, "[name=/x|foo/]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.id"),
            nav(&ast, "body.0.params.0"),
            nav(&ast, "body.0.body.body.0.declarations.0.init.left"),
        ],
    );
}

#[test]
fn attr_simple_function_numeric_index() {
    let ast = load("simpleFunction");
    let matches = query(&ast, "FunctionDeclaration[params.0.name=x]");
    assert_includes(&matches, &[nav(&ast, "body.0")]);
}

#[test]
fn attr_simple_program_regexp() {
    let ast = load("simpleProgram");
    let matches = query(&ast, "[name=/[asdfy]/]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.1.declarations.0.id"),
            nav(&ast, "body.3.test"),
            nav(&ast, "body.3.consequent.body.0.expression.left"),
        ],
    );
}

// -- literalSlash fixture --

#[test]
fn attr_slash_in_literal() {
    let ast = load("literalSlash");
    // JS: '[value="foo\\/bar"]' → selector: [value="foo\/bar"] → unescaped: "foo/bar"
    let matches = query(&ast, r#"[value="foo\/bar"]"#);
    assert_same(
        &matches,
        &[
            nav(&ast, "body.0.declarations.0.init"),
            nav(&ast, "body.2.declarations.0.init"),
        ],
    );
}

#[test]
fn attr_slash_in_regexp() {
    let ast = load("literalSlash");
    // JS: '[value=/foo\\/bar/]' → selector: [value=/foo\/bar/] → regex: foo\/bar (matches foo/bar)
    let matches = query(&ast, r"[value=/foo\/bar/]");
    assert_same(
        &matches,
        &[
            nav(&ast, "body.0.declarations.0.init"),
            nav(&ast, "body.2.declarations.0.init"),
        ],
    );
}

#[test]
fn attr_slash_in_char_class() {
    let ast = load("literalSlash");
    let matches = query(&ast, "[value=/foo[/]bar/]");
    assert_same(
        &matches,
        &[
            nav(&ast, "body.0.declarations.0.init"),
            nav(&ast, "body.2.declarations.0.init"),
        ],
    );
}

#[test]
fn attr_double_slash_in_literal() {
    let ast = load("literalSlash");
    let matches = query(&ast, r#"[value="foo\/\/bar"]"#);
    assert_same(
        &matches,
        &[
            nav(&ast, "body.1.declarations.0.init"),
            nav(&ast, "body.3.declarations.0.init"),
        ],
    );
}

#[test]
fn attr_double_slash_in_regexp() {
    let ast = load("literalSlash");
    let matches = query(&ast, r"[value=/foo\/\/bar/]");
    assert_same(
        &matches,
        &[
            nav(&ast, "body.1.declarations.0.init"),
            nav(&ast, "body.3.declarations.0.init"),
        ],
    );
}

#[test]
fn attr_double_slash_in_char_class() {
    let ast = load("literalSlash");
    let matches = query(&ast, "[value=/foo[/][/]bar/]");
    assert_same(
        &matches,
        &[
            nav(&ast, "body.1.declarations.0.init"),
            nav(&ast, "body.3.declarations.0.init"),
        ],
    );
}

// -- regexp flags --

#[test]
fn attr_for_loop_regexp() {
    let ast = load("forLoop");
    let matches = query(&ast, "[name=/i|foo/]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.init.left"),
            nav(&ast, "body.0.test.left"),
            nav(&ast, "body.0.test.right.object"),
            nav(&ast, "body.0.update.argument"),
            nav(&ast, "body.0.body.body.0.expression.callee.object"),
            nav(&ast, "body.0.body.body.0.expression.callee.property"),
        ],
    );
}

#[test]
fn attr_nonexistent_regexp() {
    let ast = load("conditional");
    let matches = query(&ast, "[foobar=/./]");
    assert_eq!(0, matches.len());
}

// -- not operators --

#[test]
fn attr_not_string() {
    let ast = load("conditional");
    let matches = query(&ast, r#"[name!="x"]"#);
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.consequent.body.0.expression.callee"),
            nav(&ast, "body.1.consequent.body.0.expression.left"),
        ],
    );
}

#[test]
fn attr_not_type() {
    let ast = load("conditional");
    let matches = query(&ast, "[value!=type(number)]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.1.test.left.left.right"),
            nav(&ast, "body.1.test.left.right"),
            nav(&ast, "body.1.alternate"),
        ],
    );
}

#[test]
fn attr_not_regexp() {
    let ast = load("conditional");
    let matches = query(&ast, "[name!=/x|y/]");
    assert_includes(
        &matches,
        &[nav(&ast, "body.0.consequent.body.0.expression.callee")],
    );
}

// -- comparison operators --

#[test]
fn attr_less_than() {
    let ast = load("conditional");
    let matches = query(&ast, "[body.length<2]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0.consequent"),
            nav(&ast, "body.0.alternate"),
            nav(&ast, "body.1.consequent"),
            nav(&ast, "body.1.alternate.consequent"),
        ],
    );
}

#[test]
fn attr_greater_than() {
    let ast = load("conditional");
    let matches = query(&ast, "[body.length>1]");
    assert_includes(&matches, &[&ast]);
}

#[test]
fn attr_lte() {
    let ast = load("conditional");
    let matches = query(&ast, "[body.length<=2]");
    assert_includes(
        &matches,
        &[
            &ast,
            nav(&ast, "body.0.consequent"),
            nav(&ast, "body.0.alternate"),
            nav(&ast, "body.1.consequent"),
            nav(&ast, "body.1.alternate.consequent"),
        ],
    );
}

#[test]
fn attr_gte() {
    let ast = load("conditional");
    let matches = query(&ast, "[body.length>=1]");
    assert_includes(
        &matches,
        &[
            &ast,
            nav(&ast, "body.0.consequent"),
            nav(&ast, "body.0.alternate"),
            nav(&ast, "body.1.consequent"),
            nav(&ast, "body.1.alternate.consequent"),
        ],
    );
}

// -- type check --

#[test]
fn attr_type_check() {
    let ast = load("conditional");

    let matches = query(&ast, "[test=type(object)]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.0"),
            nav(&ast, "body.1"),
            nav(&ast, "body.1.alternate"),
        ],
    );

    let matches = query(&ast, "[value=type(boolean)]");
    assert_includes(
        &matches,
        &[
            nav(&ast, "body.1.test.left.right"),
            nav(&ast, "body.1.alternate.test"),
        ],
    );
}
