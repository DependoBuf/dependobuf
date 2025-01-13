/// For sharing of terms, an `Rc` is used.
/// Consider migrating to `Arc` when going multicore.
pub type RecTerm<U, B, N> = std::rc::Rc<Term<U, B, N>>;

/// Parameterized grammar of terms in DependoBuf.
///
/// This includes patterns, term-level expressions and type-level expressions.
/// `U` is a type of unary operators, `B` is a type of binary operators
/// and `N` is a type of names.
#[derive(Clone, Debug)]
pub enum Term<U, B, N> {
    /// Application of a unary operator to the term.
    Unary(U, RecTerm<U, B, N>),
    /// Application of a binary operator.
    Binary(B, RecTerm<U, B, N>, RecTerm<U, B, N>),
    /// Call-by-name with a list of arguments.
    /// Dependent on the context, can be one of:
    /// * Constructor call in a pattern;
    /// * Constructor call in a value;
    /// * Type constructor.
    Call(N, Vec<Term<U, B, N>>),
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

/// Unary operators used in DependoBuf expressions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp<Name> {
    /// Access the (series of) fields in a record.
    Access(Vec<Name>),
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

/// Constructor body is a list of typed variables.
pub type ConstructorBody<Name> = Context<Name>;

/// Single branch of a DependoBuf enum type.
#[derive(Clone, Debug)]
pub struct EnumBranch<Name> {
    /// List of pattern to match dependencies against.
    pub patterns: Vec<Pattern<Name>>,
    /// List of available constructors on successful pattern match.
    pub constructors: Vec<(Name, ConstructorBody<Name>)>,
}

/// Definition of a DependoBuf type.
#[derive(Clone, Debug)]
pub enum TypeDefinition<Name> {
    /// Message has a single constructor.
    Message(ConstructorBody<Name>),
    /// Enum can have several branches.
    Enum(Vec<EnumBranch<Name>>),
}

/// Declaration of a DependoBuf type.
#[derive(Clone, Debug)]
pub struct TypeDeclaration<Name> {
    /// Name of a type.
    pub name: Name,
    /// List of dependencies.
    pub dependencies: Context<Name>,
    /// Definition.
    pub body: TypeDefinition<Name>,
}

/// A complete AST of a single DependoBuf module
/// is a list of type declarations.
pub type AST<Name> = Vec<TypeDeclaration<Name>>;
