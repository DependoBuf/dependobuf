//! Contains errors for LanguageServer.
//!

/// `textDocument/formatting` errors. Uses codes 10100..10200
mod format_errors;
/// `textDocument/rename` errors. Uses codes 10200..10300
mod rename_errors;

#[cfg(test)]
mod tests;

use tower_lsp::jsonrpc::Result;

pub use format_errors::FormatError;
pub use rename_errors::RenameError;

pub enum Error {
    FormatError(FormatError),
    RenameError(RenameError),
}

impl Error {
    #[allow(dead_code, reason = "every code use FormatError/RenameError directly")]
    pub fn to_jsonrpc_error<T>(&self) -> Result<T> {
        match self {
            Error::FormatError(format_error) => format_error.to_jsonrpc_error(),
            Error::RenameError(rename_error) => rename_error.to_jsonrpc_error(),
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
