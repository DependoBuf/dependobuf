use std::borrow::Cow;

use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::ErrorCode::ServerError;

pub fn bad_param_error(text: &str) -> Error {
    Error {
        code: ServerError(10100),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}

pub fn internal_error(text: &str) -> Error {
    Error {
        code: ServerError(10200),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}

pub fn bad_rename_error(text: &str) -> Error {
    Error {
        code: ServerError(10300),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}
