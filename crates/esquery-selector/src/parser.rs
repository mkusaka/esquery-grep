use winnow::combinator::{alt, delimited, opt, preceded, repeat, separated};
use winnow::prelude::*;
use winnow::token::{any, take_while};

use crate::ast::*;

/// Parse an ESQuery selector string into a Selector AST.
/// Returns None for empty or whitespace-only input.
pub fn parse(input: &str) -> Option<Selector> {
    start.parse(input).ok().flatten()
}

// ---------------------------------------------------------------------------
// Grammar rules (mirroring grammar.pegjs)
// ---------------------------------------------------------------------------

/// start = _ ss:selectors _ { ss.length === 1 ? ss[0] : { type: 'matches', selectors: ss } }
///       / _ { return void 0; }
fn start(input: &mut &str) -> ModalResult<Option<Selector>> {
    let _ = sp(input)?;
    let result = opt(selectors).parse_next(input)?;
    let _ = sp(input)?;
    Ok(match result {
        None => None,
        Some(mut ss) if ss.len() == 1 => Some(ss.remove(0)),
        Some(ss) => Some(Selector::new(SelectorKind::Matches(ss))),
    })
}

/// _ = " "*
fn sp(input: &mut &str) -> ModalResult<()> {
    take_while(0.., ' ').void().parse_next(input)
}

/// identifierName = [^ \[\],():#!=><~+.]+
fn identifier_name(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| {
        !matches!(
            c,
            ' ' | '[' | ']' | ',' | '(' | ')' | ':' | '#' | '!' | '=' | '>' | '<' | '~' | '+'
                | '.'
        )
    })
    .map(|s: &str| s.to_string())
    .parse_next(input)
}

// ---------------------------------------------------------------------------
// Binary operators
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum BinOp {
    Child,
    Sibling,
    Adjacent,
    Descendant,
}

/// binaryOp = _ ">" _ / _ "~" _ / _ "+" _ / " " _
fn binary_op(input: &mut &str) -> ModalResult<BinOp> {
    alt((
        (sp, ">", sp).map(|_| BinOp::Child),
        (sp, "~", sp).map(|_| BinOp::Sibling),
        (sp, "+", sp).map(|_| BinOp::Adjacent),
        (" ", sp).map(|_| BinOp::Descendant),
    ))
    .parse_next(input)
}

fn make_binary(op: BinOp, left: Selector, right: Selector) -> Selector {
    let left = Box::new(left);
    let right = Box::new(right);
    Selector::new(match op {
        BinOp::Child => SelectorKind::Child { left, right },
        BinOp::Sibling => SelectorKind::Sibling { left, right },
        BinOp::Adjacent => SelectorKind::Adjacent { left, right },
        BinOp::Descendant => SelectorKind::Descendant { left, right },
    })
}

// ---------------------------------------------------------------------------
// Selector lists
// ---------------------------------------------------------------------------

/// selectors = selector (_ "," _ selector)*
fn selectors(input: &mut &str) -> ModalResult<Vec<Selector>> {
    separated(1.., selector, (sp, ",", sp)).parse_next(input)
}

/// hasSelectors = hasSelector (_ "," _ hasSelector)*
fn has_selectors(input: &mut &str) -> ModalResult<Vec<Selector>> {
    separated(1.., has_selector, (sp, ",", sp)).parse_next(input)
}

/// hasSelector = binaryOp? selector
fn has_selector(input: &mut &str) -> ModalResult<Selector> {
    let op = opt(binary_op).parse_next(input)?;
    let s = selector(input)?;
    Ok(match op {
        None => s,
        Some(op) => make_binary(op, Selector::new(SelectorKind::ExactNode), s),
    })
}

/// selector = sequence (binaryOp sequence)*
fn selector(input: &mut &str) -> ModalResult<Selector> {
    let first = sequence(input)?;
    let rest: Vec<(BinOp, Selector)> = repeat(0.., (binary_op, sequence)).parse_next(input)?;
    Ok(rest.into_iter().fold(first, |left, (op, right)| {
        make_binary(op, left, right)
    }))
}

/// sequence = "!"? atom+
fn sequence(input: &mut &str) -> ModalResult<Selector> {
    let is_subject = opt("!").parse_next(input)?.is_some();
    let atoms: Vec<Selector> = repeat(1.., atom).parse_next(input)?;
    let mut node = if atoms.len() == 1 {
        atoms.into_iter().next().unwrap()
    } else {
        Selector::new(SelectorKind::Compound(atoms))
    };
    if is_subject {
        node.subject = true;
    }
    Ok(node)
}

// ---------------------------------------------------------------------------
// Atoms
// ---------------------------------------------------------------------------

/// atom = wildcard / identifier / attr / field / negation / matches / is
///      / has / firstChild / lastChild / nthChild / nthLastChild / class
fn atom(input: &mut &str) -> ModalResult<Selector> {
    alt((
        wildcard,
        identifier,
        attr,
        field,
        negation,
        matches_sel,
        is_sel,
        has_sel,
        first_child,
        last_child,
        nth_child,
        nth_last_child,
        class_sel,
    ))
    .parse_next(input)
}

fn wildcard(input: &mut &str) -> ModalResult<Selector> {
    "*".map(|_| Selector::new(SelectorKind::Wildcard))
        .parse_next(input)
}

/// identifier = "#"? identifierName
fn identifier(input: &mut &str) -> ModalResult<Selector> {
    let _ = opt("#").parse_next(input)?;
    let name = identifier_name(input)?;
    Ok(Selector::new(SelectorKind::Identifier(name)))
}

/// field = "." identifierName ("." identifierName)*
fn field(input: &mut &str) -> ModalResult<Selector> {
    let _ = ".".parse_next(input)?;
    let first = identifier_name(input)?;
    let rest: Vec<(&str, String)> = repeat(0.., (".", identifier_name)).parse_next(input)?;
    let mut name = first;
    for (dot, part) in rest {
        name.push_str(dot);
        name.push_str(&part);
    }
    Ok(Selector::new(SelectorKind::Field { name }))
}

// ---------------------------------------------------------------------------
// Attribute selectors
// ---------------------------------------------------------------------------

/// attr = "[" _ attrValue _ "]"
fn attr(input: &mut &str) -> ModalResult<Selector> {
    delimited("[", delimited(sp, attr_value, sp), "]")
        .map(|a| Selector::new(SelectorKind::Attribute(a)))
        .parse_next(input)
}

/// attrName = identifierName ("." identifierName)*
fn attr_name(input: &mut &str) -> ModalResult<String> {
    let first = identifier_name(input)?;
    let rest: Vec<(&str, String)> = repeat(0.., (".", identifier_name)).parse_next(input)?;
    let mut name = first;
    for (dot, part) in rest {
        name.push_str(dot);
        name.push_str(&part);
    }
    Ok(name)
}

/// attrEqOps = "!"? "=" → "=" | "!="
fn attr_eq_ops(input: &mut &str) -> ModalResult<AttrOperator> {
    alt(("!=".map(|_| AttrOperator::NotEq), "=".map(|_| AttrOperator::Eq))).parse_next(input)
}

/// attrOps = [><!]? "=" / [><]
fn attr_ops(input: &mut &str) -> ModalResult<AttrOperator> {
    alt((
        ">=".map(|_| AttrOperator::Gte),
        "<=".map(|_| AttrOperator::Lte),
        "!=".map(|_| AttrOperator::NotEq),
        "=".map(|_| AttrOperator::Eq),
        ">".map(|_| AttrOperator::Gt),
        "<".map(|_| AttrOperator::Lt),
    ))
    .parse_next(input)
}

/// attrValue = name attrEqOps (type/regex) / name attrOps (string/number/path) / name
fn attr_value(input: &mut &str) -> ModalResult<AttributeSelector> {
    alt((
        (attr_name, sp, attr_eq_ops, sp, alt((type_value, regex_value))).map(
            |(name, _, op, _, value)| AttributeSelector {
                name,
                operator: Some(op),
                value: Some(value),
            },
        ),
        (
            attr_name,
            sp,
            attr_ops,
            sp,
            alt((string_value, number_value, path_value)),
        )
            .map(|(name, _, op, _, value)| AttributeSelector {
                name,
                operator: Some(op),
                value: Some(value),
            }),
        attr_name.map(|name| AttributeSelector {
            name,
            operator: None,
            value: None,
        }),
    ))
    .parse_next(input)
}

// -- Attribute value types --

/// string = '"' ... '"' / "'" ... "'"
fn string_value(input: &mut &str) -> ModalResult<AttrValue> {
    alt((quoted_string('"'), quoted_string('\''))).parse_next(input)
}

fn quoted_string<'a>(
    quote: char,
) -> impl FnMut(&mut &'a str) -> ModalResult<AttrValue> {
    move |input: &mut &'a str| -> ModalResult<AttrValue> {
        let mut q = if quote == '"' { "\"" } else { "'" };
        let _ = q.parse_next(input)?;
        let mut raw = String::new();
        loop {
            let chunk: &str =
                take_while(0.., |c: char| c != '\\' && c != quote).parse_next(input)?;
            raw.push_str(chunk);
            if input.starts_with('\\') {
                let _ = "\\".parse_next(input)?;
                let c = any.parse_next(input)?;
                raw.push('\\');
                raw.push(c);
            } else {
                break;
            }
        }
        q = if quote == '"' { "\"" } else { "'" };
        let _ = q.parse_next(input)?;
        Ok(AttrValue::Literal(AttrLiteral::String(str_unescape(&raw))))
    }
}

/// number = ([0-9]* ".")? [0-9]+
fn number_value(input: &mut &str) -> ModalResult<AttrValue> {
    let leading = opt((take_while(0.., |c: char| c.is_ascii_digit()), ".")).parse_next(input)?;
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    let s = match leading {
        Some((pre, dot)) => format!("{}{}{}", pre, dot, digits),
        None => digits.to_string(),
    };
    Ok(AttrValue::Literal(AttrLiteral::Number(
        s.parse::<f64>().unwrap(),
    )))
}

/// path = identifierName
fn path_value(input: &mut &str) -> ModalResult<AttrValue> {
    identifier_name
        .map(|n| AttrValue::Literal(AttrLiteral::Path(n)))
        .parse_next(input)
}

/// type = "type(" _ [^ )]+ _ ")"
fn type_value(input: &mut &str) -> ModalResult<AttrValue> {
    delimited(
        ("type(", sp),
        take_while(1.., |c: char| c != ' ' && c != ')'),
        (sp, ")"),
    )
    .map(|v: &str| AttrValue::Type(v.to_string()))
    .parse_next(input)
}

/// regex = "/" pattern+ "/" flags?
/// Flags must not contain duplicates (JS: `new RegExp(pattern, flags)` throws on duplicate flags).
fn regex_value(input: &mut &str) -> ModalResult<AttrValue> {
    let _ = "/".parse_next(input)?;
    let parts: Vec<String> =
        repeat(1.., alt((re_character_class, re_escape, re_chars))).parse_next(input)?;
    let _ = "/".parse_next(input)?;
    let flags: Option<&str> = opt(take_while(1.., |c: char| matches!(c, 'i' | 'm' | 's' | 'u')))
        .parse_next(input)?;
    let flags_str = flags.unwrap_or("");
    // Reject duplicate flags (JS throws "Invalid flags supplied to RegExp constructor")
    let mut seen = [false; 4]; // i, m, s, u
    for c in flags_str.chars() {
        let idx = match c {
            'i' => 0,
            'm' => 1,
            's' => 2,
            'u' => 3,
            _ => unreachable!(),
        };
        if seen[idx] {
            // Cut (not Backtrack): we've committed to regex parsing after /pattern/flags,
            // so duplicate flags must not fall back to the path_value branch.
            return Err(winnow::error::ErrMode::Cut(winnow::error::ContextError::new()));
        }
        seen[idx] = true;
    }
    Ok(AttrValue::Regex(RegexValue {
        pattern: parts.join(""),
        flags: flags_str.to_string(),
    }))
}

fn re_character_class(input: &mut &str) -> ModalResult<String> {
    let _ = "[".parse_next(input)?;
    let parts: Vec<String> = repeat(
        1..,
        alt((
            take_while(1.., |c: char| c != ']' && c != '\\')
                .map(|s: &str| s.to_string()),
            re_escape,
        )),
    )
    .parse_next(input)?;
    let _ = "]".parse_next(input)?;
    Ok(format!("[{}]", parts.join("")))
}

fn re_escape(input: &mut &str) -> ModalResult<String> {
    let _ = "\\".parse_next(input)?;
    let c = any.parse_next(input)?;
    Ok(format!("\\{}", c))
}

fn re_chars(input: &mut &str) -> ModalResult<String> {
    take_while(1.., |c: char| !matches!(c, '/' | '\\' | '['))
        .map(|s: &str| s.to_string())
        .parse_next(input)
}

// ---------------------------------------------------------------------------
// Pseudo-selectors
// ---------------------------------------------------------------------------

fn negation(input: &mut &str) -> ModalResult<Selector> {
    delimited(":not(", delimited(sp, selectors, sp), ")")
        .map(|ss| Selector::new(SelectorKind::Not(ss)))
        .parse_next(input)
}

fn matches_sel(input: &mut &str) -> ModalResult<Selector> {
    delimited(":matches(", delimited(sp, selectors, sp), ")")
        .map(|ss| Selector::new(SelectorKind::Matches(ss)))
        .parse_next(input)
}

fn is_sel(input: &mut &str) -> ModalResult<Selector> {
    delimited(":is(", delimited(sp, selectors, sp), ")")
        .map(|ss| Selector::new(SelectorKind::Matches(ss)))
        .parse_next(input)
}

fn has_sel(input: &mut &str) -> ModalResult<Selector> {
    delimited(":has(", delimited(sp, has_selectors, sp), ")")
        .map(|ss| Selector::new(SelectorKind::Has(ss)))
        .parse_next(input)
}

fn first_child(input: &mut &str) -> ModalResult<Selector> {
    ":first-child"
        .map(|_| Selector::new(SelectorKind::NthChild { index: 1 }))
        .parse_next(input)
}

fn last_child(input: &mut &str) -> ModalResult<Selector> {
    ":last-child"
        .map(|_| Selector::new(SelectorKind::NthLastChild { index: 1 }))
        .parse_next(input)
}

fn nth_child(input: &mut &str) -> ModalResult<Selector> {
    delimited(
        ":nth-child(",
        delimited(sp, take_while(1.., |c: char| c.is_ascii_digit()), sp),
        ")",
    )
    .try_map(|n: &str| n.parse::<i32>().map(|index| Selector::new(SelectorKind::NthChild { index })))
    .parse_next(input)
}

fn nth_last_child(input: &mut &str) -> ModalResult<Selector> {
    delimited(
        ":nth-last-child(",
        delimited(sp, take_while(1.., |c: char| c.is_ascii_digit()), sp),
        ")",
    )
    .try_map(|n: &str| n.parse::<i32>().map(|index| Selector::new(SelectorKind::NthLastChild { index })))
    .parse_next(input)
}

/// class = ":" identifierName
/// Only accepts known ESQuery class names: statement, expression, declaration, function, pattern.
/// Unknown class names cause a parse failure (JS throws "Unknown class name" at match time).
fn class_sel(input: &mut &str) -> ModalResult<Selector> {
    preceded(":", identifier_name)
        .verify(|name: &String| {
            matches!(
                name.to_lowercase().as_str(),
                "statement" | "expression" | "declaration" | "function" | "pattern"
            )
        })
        .map(|name| Selector::new(SelectorKind::Class(name)))
        .parse_next(input)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn str_unescape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('b') => result.push('\u{0008}'),
                Some('f') => result.push('\u{000C}'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('v') => result.push('\u{000B}'),
                Some(other) => result.push(other),
                None => {}
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input() {
        assert_eq!(parse(""), None);
        assert_eq!(parse("   "), None);
    }

    #[test]
    fn wildcard_selector() {
        let s = parse("*").unwrap();
        assert_eq!(s.kind, SelectorKind::Wildcard);
    }

    #[test]
    fn identifier_selector() {
        let s = parse("IfStatement").unwrap();
        assert_eq!(s.kind, SelectorKind::Identifier("IfStatement".into()));
    }

    #[test]
    fn hash_identifier() {
        let s = parse("#IfStatement").unwrap();
        assert_eq!(s.kind, SelectorKind::Identifier("IfStatement".into()));
    }

    #[test]
    fn descendant_selector() {
        let s = parse("Program IfStatement").unwrap();
        match &s.kind {
            SelectorKind::Descendant { left, right } => {
                assert_eq!(left.kind, SelectorKind::Identifier("Program".into()));
                assert_eq!(right.kind, SelectorKind::Identifier("IfStatement".into()));
            }
            _ => panic!("expected Descendant, got {:?}", s.kind),
        }
    }

    #[test]
    fn child_selector() {
        let s = parse("IfStatement > BinaryExpression").unwrap();
        match &s.kind {
            SelectorKind::Child { left, right } => {
                assert_eq!(left.kind, SelectorKind::Identifier("IfStatement".into()));
                assert_eq!(
                    right.kind,
                    SelectorKind::Identifier("BinaryExpression".into())
                );
            }
            _ => panic!("expected Child"),
        }
    }

    #[test]
    fn attribute_name_only() {
        let s = parse("[name]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.name, "name");
                assert!(a.operator.is_none());
                assert!(a.value.is_none());
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn attribute_string_value() {
        let s = parse("[name=\"foo\"]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.name, "name");
                assert_eq!(a.operator, Some(AttrOperator::Eq));
                assert_eq!(a.value, Some(AttrValue::Literal(AttrLiteral::String("foo".into()))));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn attribute_number_value() {
        let s = parse("[value=21.35]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::Eq));
                assert_eq!(a.value, Some(AttrValue::Literal(AttrLiteral::Number(21.35))));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn attribute_regex_value() {
        let s = parse("[name=/foo/i]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::Eq));
                assert_eq!(
                    a.value,
                    Some(AttrValue::Regex(RegexValue {
                        pattern: "foo".into(),
                        flags: "i".into(),
                    }))
                );
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn field_selector() {
        let s = parse(".test").unwrap();
        assert_eq!(s.kind, SelectorKind::Field { name: "test".into() });
    }

    #[test]
    fn nested_field() {
        let s = parse(".declarations.init").unwrap();
        assert_eq!(
            s.kind,
            SelectorKind::Field {
                name: "declarations.init".into()
            }
        );
    }

    #[test]
    fn not_selector() {
        let s = parse(":not(IfStatement)").unwrap();
        match &s.kind {
            SelectorKind::Not(ss) => {
                assert_eq!(ss.len(), 1);
                assert_eq!(ss[0].kind, SelectorKind::Identifier("IfStatement".into()));
            }
            _ => panic!("expected Not"),
        }
    }

    #[test]
    fn matches_selector() {
        let s = parse(":matches(IfStatement, ForStatement)").unwrap();
        match &s.kind {
            SelectorKind::Matches(ss) => {
                assert_eq!(ss.len(), 2);
            }
            _ => panic!("expected Matches"),
        }
    }

    #[test]
    fn is_selector() {
        let s = parse(":is(IfStatement)").unwrap();
        // :is() produces same AST as :matches()
        match &s.kind {
            SelectorKind::Matches(ss) => {
                assert_eq!(ss.len(), 1);
            }
            _ => panic!("expected Matches (from :is)"),
        }
    }

    #[test]
    fn has_selector_test() {
        let s = parse(":has(Identifier)").unwrap();
        match &s.kind {
            SelectorKind::Has(ss) => {
                assert_eq!(ss.len(), 1);
                assert_eq!(ss[0].kind, SelectorKind::Identifier("Identifier".into()));
            }
            _ => panic!("expected Has"),
        }
    }

    #[test]
    fn has_with_child_op() {
        let s = parse(":has(> Identifier)").unwrap();
        match &s.kind {
            SelectorKind::Has(ss) => {
                assert_eq!(ss.len(), 1);
                match &ss[0].kind {
                    SelectorKind::Child { left, right } => {
                        assert_eq!(left.kind, SelectorKind::ExactNode);
                        assert_eq!(right.kind, SelectorKind::Identifier("Identifier".into()));
                    }
                    _ => panic!("expected Child in Has"),
                }
            }
            _ => panic!("expected Has"),
        }
    }

    #[test]
    fn first_child_test() {
        let s = parse(":first-child").unwrap();
        assert_eq!(s.kind, SelectorKind::NthChild { index: 1 });
    }

    #[test]
    fn last_child_test() {
        let s = parse(":last-child").unwrap();
        assert_eq!(s.kind, SelectorKind::NthLastChild { index: 1 });
    }

    #[test]
    fn nth_child_test() {
        let s = parse(":nth-child(2)").unwrap();
        assert_eq!(s.kind, SelectorKind::NthChild { index: 2 });
    }

    #[test]
    fn nth_last_child_test() {
        let s = parse(":nth-last-child(3)").unwrap();
        assert_eq!(s.kind, SelectorKind::NthLastChild { index: 3 });
    }

    #[test]
    fn class_selector() {
        let s = parse(":statement").unwrap();
        assert_eq!(s.kind, SelectorKind::Class("statement".into()));
    }

    #[test]
    fn compound_selector() {
        let s = parse("Identifier[name=\"x\"]").unwrap();
        match &s.kind {
            SelectorKind::Compound(atoms) => {
                assert_eq!(atoms.len(), 2);
                assert_eq!(atoms[0].kind, SelectorKind::Identifier("Identifier".into()));
                match &atoms[1].kind {
                    SelectorKind::Attribute(a) => assert_eq!(a.name, "name"),
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected Compound, got {:?}", s.kind),
        }
    }

    #[test]
    fn subject_indicator() {
        let s = parse("!IfStatement Identifier").unwrap();
        match &s.kind {
            SelectorKind::Descendant { left, right } => {
                assert!(left.subject);
                assert_eq!(left.kind, SelectorKind::Identifier("IfStatement".into()));
                assert!(!right.subject);
            }
            _ => panic!("expected Descendant"),
        }
    }

    #[test]
    fn top_level_comma() {
        let s = parse("IfStatement, ForStatement").unwrap();
        match &s.kind {
            SelectorKind::Matches(ss) => {
                assert_eq!(ss.len(), 2);
                assert_eq!(ss[0].kind, SelectorKind::Identifier("IfStatement".into()));
                assert_eq!(ss[1].kind, SelectorKind::Identifier("ForStatement".into()));
            }
            _ => panic!("expected Matches (comma-separated)"),
        }
    }

    #[test]
    fn type_value_in_attr() {
        let s = parse("[test=type(object)]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.value, Some(AttrValue::Type("object".into())));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn attribute_not_eq() {
        let s = parse("[name!=type(number)]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::NotEq));
                assert_eq!(a.value, Some(AttrValue::Type("number".into())));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn attribute_with_spaces() {
        let s = parse("[value  =  21.35]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.name, "value");
                assert_eq!(a.value, Some(AttrValue::Literal(AttrLiteral::Number(21.35))));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn leading_trailing_spaces() {
        let s = parse(" A ").unwrap();
        assert_eq!(s.kind, SelectorKind::Identifier("A".into()));
    }

    #[test]
    fn sibling_selector() {
        let s = parse("A ~ B").unwrap();
        match &s.kind {
            SelectorKind::Sibling { left, right } => {
                assert_eq!(left.kind, SelectorKind::Identifier("A".into()));
                assert_eq!(right.kind, SelectorKind::Identifier("B".into()));
            }
            _ => panic!("expected Sibling"),
        }
    }

    #[test]
    fn adjacent_selector() {
        let s = parse("A + B").unwrap();
        match &s.kind {
            SelectorKind::Adjacent { left, right } => {
                assert_eq!(left.kind, SelectorKind::Identifier("A".into()));
                assert_eq!(right.kind, SelectorKind::Identifier("B".into()));
            }
            _ => panic!("expected Adjacent"),
        }
    }

    #[test]
    fn single_quoted_string() {
        let s = parse("[name='foo']").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.value, Some(AttrValue::Literal(AttrLiteral::String("foo".into()))));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn escape_in_string() {
        let s = parse(r#"[name="foo\nbar"]"#).unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(
                    a.value,
                    Some(AttrValue::Literal(AttrLiteral::String("foo\nbar".into())))
                );
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn dotted_attr_name() {
        let s = parse("[left.name=\"x\"]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.name, "left.name");
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn regex_with_character_class() {
        let s = parse("[name=/[a-z]+/]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                match &a.value {
                    Some(AttrValue::Regex(r)) => {
                        assert_eq!(r.pattern, "[a-z]+");
                        assert_eq!(r.flags, "");
                    }
                    _ => panic!("expected regex value"),
                }
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn comparison_operators() {
        let s = parse("[body.length>1]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::Gt));
                assert_eq!(a.value, Some(AttrValue::Literal(AttrLiteral::Number(1.0))));
            }
            _ => panic!("expected Attribute"),
        }

        let s = parse("[body.length>=1]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::Gte));
            }
            _ => panic!("expected Attribute"),
        }

        let s = parse("[body.length<2]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::Lt));
            }
            _ => panic!("expected Attribute"),
        }

        let s = parse("[body.length<=2]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::Lte));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn multi_digit_nth_child() {
        let s = parse(":nth-child(10)").unwrap();
        assert_eq!(s.kind, SelectorKind::NthChild { index: 10 });
    }

    // -- Additional edge cases from esquery test suite --

    #[test]
    fn path_value_unquoted() {
        let s = parse("[prefix=true]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.value, Some(AttrValue::Literal(AttrLiteral::Path("true".into()))));
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn regex_escaped_slash() {
        let s = parse(r"[value=/foo\/bar/]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => match &a.value {
                Some(AttrValue::Regex(r)) => assert_eq!(r.pattern, "foo\\/bar"),
                _ => panic!("expected regex"),
            },
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn regex_slash_in_character_class() {
        let s = parse("[value=/foo[/]bar/]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => match &a.value {
                Some(AttrValue::Regex(r)) => assert_eq!(r.pattern, "foo[/]bar"),
                _ => panic!("expected regex"),
            },
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn regex_hex_escape() {
        let s = parse(r"[value=/foo\x2Fbar/]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => match &a.value {
                Some(AttrValue::Regex(r)) => assert_eq!(r.pattern, "foo\\x2Fbar"),
                _ => panic!("expected regex"),
            },
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn regex_multiple_flags() {
        let s = parse(r"[name=/\u{61}|[SDFY]/iu]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => match &a.value {
                Some(AttrValue::Regex(r)) => {
                    assert_eq!(r.flags, "iu");
                }
                _ => panic!("expected regex"),
            },
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn not_eq_regex() {
        let s = parse("[name!=/x|y/]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.operator, Some(AttrOperator::NotEq));
                match &a.value {
                    Some(AttrValue::Regex(r)) => assert_eq!(r.pattern, "x|y"),
                    _ => panic!("expected regex"),
                }
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn chained_child_selectors() {
        let s = parse("IfStatement > BinaryExpression > Identifier").unwrap();
        // Left-associative: (IfStatement > BinaryExpression) > Identifier
        match &s.kind {
            SelectorKind::Child { left, right } => {
                assert_eq!(right.kind, SelectorKind::Identifier("Identifier".into()));
                match &left.kind {
                    SelectorKind::Child { left: ll, right: lr } => {
                        assert_eq!(ll.kind, SelectorKind::Identifier("IfStatement".into()));
                        assert_eq!(lr.kind, SelectorKind::Identifier("BinaryExpression".into()));
                    }
                    _ => panic!("expected nested Child"),
                }
            }
            _ => panic!("expected Child"),
        }
    }

    #[test]
    fn subject_on_wildcard() {
        let s = parse("!* > [name=\"foo\"]").unwrap();
        match &s.kind {
            SelectorKind::Child { left, .. } => {
                assert!(left.subject);
                assert_eq!(left.kind, SelectorKind::Wildcard);
            }
            _ => panic!("expected Child"),
        }
    }

    #[test]
    fn subject_on_compound() {
        let s = parse("![left.name=\"x\"][right.value=1]").unwrap();
        assert!(s.subject);
        match &s.kind {
            SelectorKind::Compound(atoms) => assert_eq!(atoms.len(), 2),
            _ => panic!("expected Compound"),
        }
    }

    #[test]
    fn subject_on_field() {
        let s = parse("!.test").unwrap();
        assert!(s.subject);
        assert_eq!(s.kind, SelectorKind::Field { name: "test".into() });
    }

    #[test]
    fn has_with_compound_arg() {
        let s = parse("ExpressionStatement:has([name=\"foo\"][type=\"Identifier\"])").unwrap();
        match &s.kind {
            SelectorKind::Compound(atoms) => {
                assert_eq!(atoms.len(), 2);
                assert_eq!(
                    atoms[0].kind,
                    SelectorKind::Identifier("ExpressionStatement".into())
                );
                match &atoms[1].kind {
                    SelectorKind::Has(ss) => {
                        assert_eq!(ss.len(), 1);
                        match &ss[0].kind {
                            SelectorKind::Compound(inner) => assert_eq!(inner.len(), 2),
                            _ => panic!("expected Compound in Has"),
                        }
                    }
                    _ => panic!("expected Has"),
                }
            }
            _ => panic!("expected Compound"),
        }
    }

    #[test]
    fn chained_has() {
        let s = parse("BinaryExpression:has(Identifier[name=\"x\"]):has(Literal[value=\"test\"])").unwrap();
        match &s.kind {
            SelectorKind::Compound(atoms) => {
                assert_eq!(atoms.len(), 3); // BinaryExpression, :has(...), :has(...)
            }
            _ => panic!("expected Compound"),
        }
    }

    #[test]
    fn nested_has() {
        let s = parse("Program:has(IfStatement:has(Literal[value=true], Literal[value=false]))").unwrap();
        assert!(s.kind != SelectorKind::Wildcard); // Just verify it parses
    }

    #[test]
    fn matches_with_nth_child() {
        let s = parse(":matches(:nth-child(2), :nth-last-child(2))").unwrap();
        match &s.kind {
            SelectorKind::Matches(ss) => assert_eq!(ss.len(), 2),
            _ => panic!("expected Matches"),
        }
    }

    #[test]
    fn not_compound_with_nth() {
        let s = parse(":not(:nth-child(2)):nth-last-child(2)").unwrap();
        match &s.kind {
            SelectorKind::Compound(atoms) => assert_eq!(atoms.len(), 2),
            _ => panic!("expected Compound"),
        }
    }

    #[test]
    fn has_with_child_op_compound() {
        let s = parse(":has(> Identifier[name=\"x\"])").unwrap();
        match &s.kind {
            SelectorKind::Has(ss) => {
                assert_eq!(ss.len(), 1);
                match &ss[0].kind {
                    SelectorKind::Child { left, right } => {
                        assert_eq!(left.kind, SelectorKind::ExactNode);
                        match &right.kind {
                            SelectorKind::Compound(atoms) => assert_eq!(atoms.len(), 2),
                            _ => panic!("expected Compound"),
                        }
                    }
                    _ => panic!("expected Child"),
                }
            }
            _ => panic!("expected Has"),
        }
    }

    #[test]
    fn has_with_multiple_child_ops() {
        let s = parse(":has(> LogicalExpression.test, > Identifier[name=\"x\"])").unwrap();
        match &s.kind {
            SelectorKind::Has(ss) => assert_eq!(ss.len(), 2),
            _ => panic!("expected Has"),
        }
    }

    #[test]
    fn deep_nested_field() {
        let s = parse(".body.declarations.init").unwrap();
        assert_eq!(
            s.kind,
            SelectorKind::Field {
                name: "body.declarations.init".into()
            }
        );
    }

    #[test]
    fn attr_with_equals_in_value() {
        let s = parse("[operator=\"=\"]").unwrap();
        match &s.kind {
            SelectorKind::Attribute(a) => {
                assert_eq!(a.name, "operator");
                assert_eq!(
                    a.value,
                    Some(AttrValue::Literal(AttrLiteral::String("=".into())))
                );
            }
            _ => panic!("expected Attribute"),
        }
    }

    #[test]
    fn all_string_escapes() {
        let s = parse(r"Literal[value='\b\f\n\r\t\v and just a \ back\slash']").unwrap();
        match &s.kind {
            SelectorKind::Compound(atoms) => {
                match &atoms[1].kind {
                    SelectorKind::Attribute(a) => match &a.value {
                        Some(AttrValue::Literal(AttrLiteral::String(v))) => {
                            assert!(v.contains('\u{0008}')); // \b
                            assert!(v.contains('\u{000C}')); // \f
                            assert!(v.contains('\n'));
                            assert!(v.contains('\r'));
                            assert!(v.contains('\t'));
                            assert!(v.contains('\u{000B}')); // \v
                        }
                        _ => panic!("expected string value"),
                    },
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected Compound"),
        }
    }

    #[test]
    fn non_standard_escape() {
        let s = parse(r#"Literal[value="\z"]"#).unwrap();
        match &s.kind {
            SelectorKind::Compound(atoms) => {
                match &atoms[1].kind {
                    SelectorKind::Attribute(a) => {
                        assert_eq!(
                            a.value,
                            Some(AttrValue::Literal(AttrLiteral::String("z".into())))
                        );
                    }
                    _ => panic!("expected Attribute"),
                }
            }
            _ => panic!("expected Compound"),
        }
    }

    /// Verify all selector strings from the esquery test suite parse successfully.
    #[test]
    fn all_esquery_test_selectors_parse() {
        let selectors = vec![
            // queryType
            "Identifier",
            "IfStatement",
            "Program",
            // queryWildcard
            "*",
            // queryAttribute
            "[name=\"x\"]",
            "[callee.name=\"foo\"]",
            "[operator]",
            "[prefix=true]",
            "Literal[value=21.35]",
            "Literal[value  =  21.35]",
            "[operator=\"=\"]",
            "[object.name=\"foo\"]",
            "[kind=\"var\"]",
            "[id.name=\"foo\"]",
            "[left]",
            "[id.name=\"y\"]",
            "[body]",
            "[name=/x|foo/]",
            "FunctionDeclaration[params.0.name=x]",
            "[name=/[asdfy]/]",
            r"[value=/foo\/bar/]",
            r"[value=/foo\x2Fbar/]",
            r"[value=/foo\u002Fbar/]",
            "[value=/foo[/]bar/]",
            r"[value=/foo\/\/bar/]",
            "[value=/foo[/][/]bar/]",
            r"[value=/foo\x2F\x2Fbar/]",
            r"[value=/foo\u002F\u002Fbar/]",
            r"[name=/\u{61}|[SDFY]/iu]",
            r"[value=/\f.\r/s]",
            r"[value=/^\r/m]",
            "[name=/i|foo/]",
            "[foobar=/./]",
            "[name!=\"x\"]",
            "[value!=type(number)]",
            "[name!=/x|y/]",
            "[body.length<2]",
            "[body.length>1]",
            "[body.length<=2]",
            "[body.length>=1]",
            "[test=type(object)]",
            "[value=type(boolean)]",
            // queryCompound
            "[left.name=\"x\"][right.value=1]",
            "[left.name=\"x\"]:matches(*)",
            // queryComplex
            "IfStatement > BinaryExpression",
            "IfStatement > BinaryExpression > Identifier",
            "IfStatement BinaryExpression",
            "VariableDeclaration ~ IfStatement",
            "VariableDeclaration + ExpressionStatement",
            "NonExistingNodeType > *",
            // queryField
            ".test",
            ".declarations.init",
            ".body.declarations.init",
            // queryPseudoChild
            ":first-child",
            ":last-child",
            ":nth-child(2)",
            ":nth-last-child(2)",
            ":nth-child(10)",
            ":nth-last-child(10)",
            ":nth-last-child(1)",
            ":nth-child(3)",
            ":matches(:nth-child(2), :nth-last-child(2))",
            ":not(:nth-child(2)):nth-last-child(2)",
            ":nth-last-child(2) > :nth-child(2)",
            ":nth-last-child(2) :nth-child(2)",
            "*:has(:nth-child(2))",
            // querySubject
            "!IfStatement Identifier",
            "!* > [name=\"foo\"]",
            "!:nth-child(1) [name=\"y\"]",
            "!:nth-last-child(1) [name=\"y\"]",
            "![test] [name=\"y\"]",
            "![generator=type(boolean)] > BlockStatement",
            "![operator=/=+/] > [name=\"x\"]",
            "!.test",
            "!:matches(*) > [name=\"foo\"]",
            "!:not(BlockStatement) > [name=\"foo\"]",
            "![left.name=\"x\"][right.value=1]",
            "* !AssignmentExpression",
            "* > !AssignmentExpression",
            "!VariableDeclaration ~ IfStatement",
            "!VariableDeclaration ~ !IfStatement",
            "!VariableDeclaration + !ExpressionStatement",
            "Identifier + Identifier",
            "Identifier ~ Identifier",
            // queryNot
            ":not(IfStatement)",
            ":not(*)",
            // queryMatches
            ":matches(IfStatement, ForStatement)",
            ":is(IfStatement)",
            // queryHas
            "ExpressionStatement:has([name=\"foo\"][type=\"Identifier\"])",
            "IfStatement:has(LogicalExpression [name=\"foo\"], LogicalExpression [name=\"x\"])",
            "BinaryExpression:has(Identifier[name=\"x\"]):has(Literal[value=\"test\"])",
            "Program:has(IfStatement:has(Literal[value=true], Literal[value=false]))",
            ":has([value=\"impossible\"])",
            "IfStatement:has(> Identifier[name=\"x\"])",
            "IfStatement:has(> LogicalExpression.test, > Identifier[name=\"x\"])",
            // queryClass
            ":statement",
            ":expression",
            ":declaration",
            ":function",
            ":pattern",
        ];
        for sel in &selectors {
            assert!(
                parse(sel).is_some(),
                "Failed to parse selector: {:?}",
                sel
            );
        }
    }

    #[test]
    fn nth_child_overflow_does_not_panic() {
        // i32 overflow should return None (parse failure), not panic
        assert_eq!(parse(":nth-child(999999999999999999999999999999)"), None);
        assert_eq!(parse(":nth-last-child(999999999999999999999999999999)"), None);
    }

    #[test]
    fn duplicate_regex_flags_rejected_at_parse() {
        // JS: [name=/x/ii] throws "Invalid flags supplied to RegExp constructor 'ii'"
        // Must NOT fall back to path_value and parse as Literal(Path("/x/ii"))
        assert_eq!(parse("[name=/x/ii]"), None);
        assert_eq!(parse("[name=/x/ss]"), None);
        // Valid flags still work
        assert!(parse("[name=/x/ims]").is_some());
    }
}
