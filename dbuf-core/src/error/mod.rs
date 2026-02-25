//! Module contains `Error` enum that contains
//! every possible error can appear during compilation.

pub mod elaborating;
pub mod lexing;
pub mod parsing;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Lexing Error: {0}")]
    LexingError(lexing::Error),
    #[error("Parsing Error: {0}")]
    ParsingError(parsing::Error),
    #[error("Elaborating Error: {0}")]
    ElaboratingError(elaborating::Error),
}

impl From<lexing::Error> for Error {
    fn from(value: lexing::Error) -> Self {
        Self::LexingError(value)
    }
}

impl From<parsing::Error> for Error {
    fn from(value: parsing::Error) -> Self {
        Self::ParsingError(value)
    }
}

impl From<elaborating::Error> for Error {
    fn from(value: elaborating::Error) -> Self {
        Self::ElaboratingError(value)
    }
}
