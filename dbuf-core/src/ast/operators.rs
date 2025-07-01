/// Possible shapes of DependoBuf operator calls.
///
/// NOTE: this includes literals as they can be viewed as "nullary operators".
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum OpCall<S, T> {
    /// Literal (nullary operator call).
    Literal(Literal),
    /// Unary operator call.
    Unary(UnaryOp<S>, T),
    /// Binary operator call.
    Binary(BinaryOp, T, T),
}

/// Literals used in DependoBuf expressions.
#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum Literal {
    Bool(bool),
    Double(f64),
    Int(i64),
    UInt(u64),
    Str(String),
}

/// Unary operators used in DependoBuf expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp<S> {
    /// Access the field in a record.
    Access(S),
    /// Unary minus.
    Minus,
    /// Unary bang.
    Bang,
}

/// Binary operators used in DependoBuf expressions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    /// Addition
    Plus,
    /// Subtraction
    Minus,
    /// Multiplication
    Star,
    /// Division
    Slash,
    /// Binary and.
    BinaryAnd,
    /// Binary or.
    BinaryOr,
}
