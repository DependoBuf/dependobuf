//! Module contains `parsing::Error` - errors that appear
//! during parsing phase

use std::fmt::Display;

use strum_macros::EnumMessage;
use thiserror::Error;

use crate::cst::Token;
use crate::location::{Location, Offset};

/// General parsing error structure.
#[derive(Clone, Debug, Error)]
#[error("Parsing Error")]
pub struct Error {
    pub found: Option<Token>,
    pub expected: Vec<ExpectedPattern>,
    pub at: Location<Offset>,
    pub extra: Option<ErrorExtra>,
}

/// Extra information about error.
#[derive(Clone, Debug, EnumMessage, PartialEq, Eq)]
pub enum ErrorExtra {
    /// Call chain ends with dot.
    ///
    /// Argument: Location of whole call chain
    BadCallChain(Location<Offset>),
    /// Missing comma in defintion.
    ///
    /// Argument: Location of line with no comma
    MissingComma(Location<Offset>),
    /// Typed hole found.
    TypedHole,
}

/// Possible expected tokens for parser.
#[derive(Clone, Debug, PartialEq)]
pub enum ExpectedPattern {
    Token(Token),
    Label(&'static str),
    Any,
    SomethingElse,
    EndOfInput,
}

impl Display for ExpectedPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpectedPattern::Token(token) => token.fmt(f),
            ExpectedPattern::Label(l) => l.fmt(f),
            ExpectedPattern::Any => "Any".fmt(f),
            ExpectedPattern::SomethingElse => "Something else".fmt(f),
            ExpectedPattern::EndOfInput => "End of input".fmt(f),
        }
    }
}
