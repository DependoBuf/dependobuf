use std::borrow::Cow;

use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::ErrorCode::ServerError;
use tower_lsp::jsonrpc::Result;

/// Returns param is incorrect error.
fn bad_param_error(text: &'static str, code: i64) -> Error {
    assert!((0..100).contains(&code));
    Error {
        code: ServerError(10100 + code),
        message: Cow::Borrowed(text),
        data: None,
    }
}

pub fn bad_insert_spaces<T>() -> Result<T> {
    Err(bad_param_error("property 'insert_spaces' not true", 0))
}

pub fn bad_properties<T>() -> Result<T> {
    Err(bad_param_error("property 'properties' not empty", 1))
}

pub fn bad_trim_trailing_whitespace<T>() -> Result<T> {
    Err(bad_param_error(
        "property 'trim_trailing_whitespace' not none",
        2,
    ))
}

pub fn bad_insert_final_newline<T>() -> Result<T> {
    Err(bad_param_error(
        "property 'insert_final_newline' not none",
        3,
    ))
}

pub fn bad_trim_final_newlines<T>() -> Result<T> {
    Err(bad_param_error(
        "property 'trim_final_newlines' not none",
        4,
    ))
}
