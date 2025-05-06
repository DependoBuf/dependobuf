use std::borrow::Cow;

use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::ErrorCode::ServerError;
use tower_lsp::jsonrpc::Result;

use strum_macros::EnumIter;

#[derive(EnumIter)]
pub enum FormatError {
    InsertSpaces,
    Properties,
    TrimTrailingWhitespace,
    InsertFinalNewLine,
    TrimFinalNewLines,
}

/// Returns param is incorrect error.
fn error<T>(text: &'static str, code: i64) -> Result<T> {
    Err(Error {
        code: ServerError(10100 + code),
        message: Cow::Borrowed(text),
        data: None,
    })
}

impl FormatError {
    pub fn to_jsonrpc_error<T>(&self) -> Result<T> {
        match self {
            FormatError::InsertSpaces => error("property 'insert_spaces' not true", 0),

            FormatError::Properties => error("property 'properties' not empty", 1),
            FormatError::TrimTrailingWhitespace => {
                error("property 'trim_trailing_whitespace' not none", 2)
            }
            FormatError::InsertFinalNewLine => error("property 'insert_final_newline' not none", 3),
            FormatError::TrimFinalNewLines => error("property 'trim_final_newlines' not none", 4),
        }
    }
}

impl<T> From<FormatError> for Result<T> {
    fn from(value: FormatError) -> Self {
        value.to_jsonrpc_error()
    }
}
