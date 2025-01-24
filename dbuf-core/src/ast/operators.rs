/// Possible shapes of DependoBuf operator calls.
///
/// NOTE: this includes literals as they can be viewed as "nullary operators".
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum OpCall<S, T> {
    /// Literal (nullary operator call).
    Literal(Literal<S>),
    /// Unary operator call.
    Unary(UnaryOp<S>, T),
    /// Binary operator call.
    Binary(BinaryOp, T, T),
}

/// Literals used in DependoBuf expressions.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Literal<S> {
    Bool(bool),
    Double(f64),
    Int(i64),
    UInt(u64),
    Str(S)
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
    /// Binary plus.
    Plus,
    /// Binary minus.
    Minus,
    /// Binary star (multiplication).types
    Star,
    /// Binary slash (division).
    Slash,
    /// Binary and.
    And,
    /// Binary or.
    Or,
}
