//! Module contains `LexingStage` struct - error data for lexing stage.

use std::fmt::Display;

use strum::EnumMessage;
use strum_macros::EnumMessage;
use thiserror::Error;

use super::ErrorStage;

use crate::location::{LocatedName, Offset};

/// General lexing errors structure.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("[ln {}, ch {}]: Token '{}' raised error: {}",
    {data.start.lines}, {data.start.columns}, {&data.content},
    {kind.get_documentation().expect("every enum variant has documentation")})]
pub struct LexingStage {
    /// Additional data to error.
    pub data: LocatedName<String, Offset>,
    /// Kind of error.
    pub kind: ErrorKind,
}

impl ErrorStage for LexingStage {
    fn location(&self) -> crate::location::Location<Offset> {
        (&self.data).into()
    }
}

impl From<super::Error<LexingStage>> for LexingStage {
    fn from(value: super::Error<LexingStage>) -> Self {
        value.stage
    }
}

/// All lexing error kinds.
///
/// Every variant should have doc comment, explaining it.
#[derive(Debug, Clone, PartialEq, Eq, EnumMessage)]
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
