/// ESQuery selector AST types.
/// Mirrors the structure produced by the esquery PEG parser.

/// Selector node with optional subject flag (`!` prefix).
#[derive(Debug, Clone, PartialEq)]
pub struct Selector {
    pub kind: SelectorKind,
    pub subject: bool,
}

impl Selector {
    pub fn new(kind: SelectorKind) -> Self {
        Self { kind, subject: false }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectorKind {
    Wildcard,
    Identifier(String),
    ExactNode,
    Field { name: String },
    Attribute(AttributeSelector),
    /// Class pseudo-selector (`:statement`, `:expression`, etc.).
    /// Name is stored as-is; validation happens at match time.
    Class(String),
    Compound(Vec<Selector>),
    Matches(Vec<Selector>),
    Not(Vec<Selector>),
    Has(Vec<Selector>),
    NthChild { index: i32 },
    NthLastChild { index: i32 },
    Child { left: Box<Selector>, right: Box<Selector> },
    Descendant { left: Box<Selector>, right: Box<Selector> },
    Sibling { left: Box<Selector>, right: Box<Selector> },
    Adjacent { left: Box<Selector>, right: Box<Selector> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct AttributeSelector {
    pub name: String,
    pub operator: Option<AttrOperator>,
    pub value: Option<AttrValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttrOperator {
    /// `=`
    Eq,
    /// `!=`
    NotEq,
    /// `>`
    Gt,
    /// `>=`
    Gte,
    /// `<`
    Lt,
    /// `<=`
    Lte,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrValue {
    Literal(AttrLiteral),
    /// `type(...)` value
    Type(String),
    Regex(RegexValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrLiteral {
    String(String),
    Number(f64),
    /// Unquoted identifier used as a value
    Path(String),
}

#[derive(Debug, Clone)]
pub struct RegexValue {
    pub pattern: String,
    pub flags: String,
}

impl PartialEq for RegexValue {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern && self.flags == other.flags
    }
}
