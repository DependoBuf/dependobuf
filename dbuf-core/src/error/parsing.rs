//! Module contains `parsing::Error` - errors that appear
//! during parsing phase

use std::fmt::Display;

use strum_macros::EnumMessage;
use thiserror::Error;

use crate::cst::Location;
use crate::cst::Token;

/// General parsing error structure.
#[derive(Clone, Debug, Error)]
#[error("Parsing Error")]
pub struct Error {
    pub found: Option<Token>,
    pub expected: Vec<ExpectedPattern>,
    pub at: Location,
    pub extra: Option<ErrorExtra>,
}

/// Extra information about error.
#[derive(Clone, Debug, EnumMessage)]
pub enum ErrorExtra {
    /// Call chain ends with dot.
    BadCallChain(Location),
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
