//! Contains errors for LanguageServer.
//! 

use std::borrow::Cow;

use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::ErrorCode::ServerError;

/// Returns param is incorrect error.
pub fn bad_param_error(text: &str) -> Error {
    Error {
        code: ServerError(10100),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}

/// Returns iternal error.
pub fn internal_error(text: &str) -> Error {
    Error {
        code: ServerError(10200),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}

/// Returns bad rename parameters error.
pub fn bad_rename_error(text: &str) -> Error {
    Error {
        code: ServerError(10300),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}
