//! Module contains `ParsingStage` struct - error data for parsing stage.
//!
//! Note:
//! Convertation from cst to ast considered to produce `Error<ParsingStage>` errors.

use std::fmt::Display;

use strum_macros::EnumMessage;
use thiserror::Error;

use super::ErrorStage;

use crate::cst::Token;
use crate::error::LexingError;
use crate::location::{Location, Offset};

/// General parsing error structure.
#[derive(Clone, Debug, Error)]
#[error("Parsing Error")]
pub struct ParsingStage {
    pub found: Option<Token>,
    pub expected: Vec<ExpectedPattern>,
    pub at: Location<Offset>,
    pub extra: Option<ErrorExtra>,
}

impl ErrorStage for ParsingStage {
    fn location(&self) -> Location<Offset> {
        self.at
    }
}

impl From<super::Error<ParsingStage>> for ParsingStage {
    fn from(value: super::Error<ParsingStage>) -> Self {
        value.stage
    }
}

/// Call chain ends with dot.
///
/// Argument: Location of whole call chain
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BadCallChain(pub Location<Offset>);

/// Missing comma in defintion.
///
/// Argument: Location of line with no comma
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingComma(pub Location<Offset>);

/// Typed hole found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedHole;

/// Error during lexing phase.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParserLexingError(pub LexingError);

/// Extra information about error.
#[derive(Clone, Debug, EnumMessage, PartialEq, Eq)]
pub enum ErrorExtra {
    /// Call chain ends with dot.
    BadCallChain(BadCallChain),
    /// Missing comma in defintion.
    MissingComma(MissingComma),
    /// Typed hole found.
    TypedHole(TypedHole),
    /// Error during lexing phase.
    LexingError(ParserLexingError),
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
