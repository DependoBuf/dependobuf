/// Parameterized grammar of terms in DependoBuf.
///
/// This includes patterns, term-level expressions and type-level expressions.
/// `U` is a type of unary operators, `B` is a type of binary operators
/// and `N` is a type of names.
#[derive(Clone, Debug)]
pub enum Term<U, B, N> {
    /// Application of a unary operator to the term.
    Unary(U, Rec<Term<U, B, N>>),
    /// Application of a binary operator.
    Binary(B, Rec<Term<U, B, N>>, Rec<Term<U, B, N>>),
    /// Call-by-name with a list of arguments.
    /// Dependent on the context, can be one of:
    /// * Constructor call in a pattern;
    /// * Constructor call in a value;
    /// * Type constructor.
    Call(N, Rec<[Term<U, B, N>]>),
    /// Boolean literal.
    Bool(bool),
    /// Floating point literal.
    Double(f64),
    /// Integer literal.
    Int(i64),
    /// Unsigned literal.
    UInt(u64),
    /// String literal.
    Str(String),
    /// A fits-all wildcard.
    /// 1. When used as a pattern, matches anything.
    /// 2. When used as an expression, reports the expected type.
    Wildcard,
}

/// For sharing of terms, an `Rc` is used.
/// Consider migrating to `Arc` when going multicore.
pub type Rec<T> = std::rc::Rc<T>;

/// Unary operators used in DependoBuf expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp<Name> {
    /// Access the field in a record.
    Access(Name),
    /// Unary minus.
    Minus,
    /// Unary bang.
    Bang,
}

/// Binary operators used in DependoBuf expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    /// Binary plus.
    Plus,
    /// Binary minus.
    Minus,
    /// Binary star (multiplication).
    Star,
    /// Binary slash (division).
    Slash,
    /// Binary and.
    And,
    /// Binary or.
    Or,
}

/// Empty sum type. "no options available"
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Void {}

/// Pattern is a term where operators are not available.
pub type Pattern<Name> = Term<Void, Void, Name>;

/// Expression is a term where all operators are available.
pub type Expression<Name> = Term<UnaryOp<Name>, BinaryOp, Name>;

/// Type-level expression is the same as term-level expression.
pub type TypeExpression<Name> = Expression<Name>;

/// Context is a list of typed variables.
pub type Context<Name> = Vec<(Name, TypeExpression<Name>)>;
