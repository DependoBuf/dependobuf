use std::borrow::Cow;

use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::ErrorCode::ServerError;

pub fn bad_param_error(text: &str) -> Error {
    Error {
        code: ServerError(10400),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}

pub fn internal_error(text: &str) -> Error {
    Error {
        code: ServerError(10500),
        message: Cow::Owned(text.to_owned()),
        data: None,
    }
}
