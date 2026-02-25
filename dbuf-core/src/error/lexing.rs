//! Module contains `lexing::Error` - errors that appear
//! during lexing phase

use std::fmt::Display;

use strum::EnumMessage;
use strum_macros::EnumMessage;
use thiserror::Error;

use crate::ast::parsed::location::Offset;

/// General lexing errors structure.
#[derive(Debug, Clone, PartialEq, Error)]
#[error("[ln {}, ch {}]: Token '{}' raised error: {}",
    {data.at.lines}, {data.at.columns}, {&data.current},
    {kind.get_documentation().expect("every enum variant has documentation")})]
pub struct Error {
    /// Additional data to error.
    pub data: ErrorData,
    /// Kind of error.
    pub kind: ErrorKind,
}

/// Common lexing error data.
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorData {
    /// Starting position of token, raised error.
    pub at: Offset,
    /// String representation of token, raised error.
    pub current: String,
}

/// All lexing error kinds.
///
/// Every variant should have doc comment, explaining it.
#[derive(Debug, Clone, PartialEq, EnumMessage)]
pub enum ErrorKind {
    /// Integer is too huge.
    IntegerOverflow,
    /// Integer is incorrect.
    InvalidInteger,
    /// Float is incorrect.
    InvalidFloat,
    /// String literal is incorrect.
    InvalidStringLiteral,
    /// `LCIdentifier` is incorrect. May contain only [a-zA-Z0-9].
    InvalidLCIdentifier,
    /// `UCIdentifier` is incorrect. May contain only [a-zA-Z0-9].
    InvalidUCIdentifier,
    /// Unknown token.
    UnknownToken,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_documentation()
            .expect("every enum variant has documentation")
            .fmt(f)
    }
}
