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

#[derive(EnumDiscriminants)]
pub enum Error {
    FormatError(FormatError),
    RenameError(RenameError),
}

impl Error {
    fn get_code(&self) -> i64 {
        let offset = self.discriminant().get_code_offset();
        let code = match &self {
            Error::FormatError(format_error) => format_error.get_code(),
            Error::RenameError(rename_error) => rename_error.get_code(),
        };
        offset + code
    }

    fn get_message(&self) -> String {
        match &self {
            Error::FormatError(format_error) => format_error.to_string(),
            Error::RenameError(rename_error) => rename_error.to_string(),
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
            ErrorDiscriminants::FormatError => 10000,
            ErrorDiscriminants::RenameError => 10100,
        }
    }
}

impl From<FormatError> for Error {
    fn from(value: FormatError) -> Self {
        Error::FormatError(value)
    }
}

impl From<RenameError> for Error {
    fn from(value: RenameError) -> Self {
        Error::RenameError(value)
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
