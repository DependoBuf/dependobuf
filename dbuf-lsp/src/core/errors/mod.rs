//! Contains errors for Language Server.
//!

/// `textDocument/formatting` errors
mod format_errors;
/// `textDocument/rename` errors.
mod rename_errors;

#[cfg(test)]
mod tests;

use std::borrow::Cow;

use strum::IntoDiscriminant;
use strum_macros::EnumDiscriminants;
use tower_lsp::jsonrpc::{self, ErrorCode};

pub use format_errors::FormatError;
pub use rename_errors::RenameError;

pub use dbuf_core::error::ElaboratingError;
pub use dbuf_core::error::ParsingError;

#[derive(EnumDiscriminants)]
pub enum Error {
    Format(FormatError),
    Rename(RenameError),
    Parsing(ParsingError),
    ElaboratingError(ElaboratingError),
}

impl Error {
    fn get_code(&self) -> i64 {
        let offset = self.discriminant().get_code_offset();
        let code = match &self {
            Error::Format(format_error) => format_error.get_code(),
            Error::Rename(rename_error) => rename_error.get_code(),
            Error::Parsing(_parsing_error) => 0,
            Error::ElaboratingError(_elaborating_error) => 0,
        };
        offset + code
    }

    fn get_message(&self) -> String {
        match &self {
            Error::Format(format_error) => format_error.to_string(),
            Error::Rename(rename_error) => rename_error.to_string(),
            Error::Parsing(parsing_error) => parsing_error.to_string(),
            Error::ElaboratingError(elaborating_error) => elaborating_error.to_string(),
        }
    }

    pub fn to_jsonrpc_error(&self) -> jsonrpc::Error {
        let code = self.get_code();
        let message = self.get_message();
        jsonrpc::Error {
            code: ErrorCode::ServerError(code),
            message: Cow::Owned(message),
            data: None,
        }
    }
}

impl ErrorDiscriminants {
    fn get_code_offset(self) -> i64 {
        match self {
            ErrorDiscriminants::Format => 10000,
            ErrorDiscriminants::Rename => 10100,
            ErrorDiscriminants::Parsing => 1,
            ErrorDiscriminants::ElaboratingError => 2,
        }
    }
}

impl From<FormatError> for Error {
    fn from(value: FormatError) -> Self {
        Error::Format(value)
    }
}

impl From<RenameError> for Error {
    fn from(value: RenameError) -> Self {
        Error::Rename(value)
    }
}

impl From<ParsingError> for Error {
    fn from(value: ParsingError) -> Self {
        Error::Parsing(value)
    }
}

impl From<ElaboratingError> for Error {
    fn from(value: ElaboratingError) -> Self {
        Error::ElaboratingError(value)
    }
}

impl From<Error> for jsonrpc::Error {
    fn from(value: Error) -> Self {
        value.to_jsonrpc_error()
    }
}

impl From<FormatError> for jsonrpc::Error {
    fn from(value: FormatError) -> Self {
        Error::from(value).into()
    }
}

impl From<RenameError> for jsonrpc::Error {
    fn from(value: RenameError) -> Self {
        Error::from(value).into()
    }
}
