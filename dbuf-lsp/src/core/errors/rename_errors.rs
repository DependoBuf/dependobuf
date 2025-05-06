use std::borrow::Cow;

use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::ErrorCode::ServerError;
use tower_lsp::jsonrpc::Result;

use strum_macros::EnumIter;

#[derive(EnumIter)]
pub enum RenameError {
    ToEmpty,
    ToBuiltin,
    ToKeyword,
    OfBuiltin,
    ToPrevious,
    OfNone,
    ToBadType(String),
    ToBadDependency(String),
    ToBadField(String),
    ToExistingType(String),
    ToExistingResource { t: String, r: String },
    OfAlias,
    OfConstructor,
}

/// Returns rename error.
fn error<T>(text: &'static str, code: i64) -> Result<T> {
    assert!((0..100).contains(&code));
    Err(Error {
        code: ServerError(10200 + code),
        message: Cow::Borrowed(text),
        data: None,
    })
}

/// Returns rename error from string.
fn error_from_string<T>(text: String, code: i64) -> Result<T> {
    assert!((0..100).contains(&code));
    Err(Error {
        code: ServerError(10200 + code),
        message: Cow::Owned(text),
        data: None,
    })
}

impl RenameError {
    pub fn to_jsonrpc_error<T>(&self) -> Result<T> {
        match self {
            RenameError::ToEmpty => error("rename to empty string", 0),
            RenameError::ToBuiltin => error("rename to builtin type is forbidden", 1),
            RenameError::ToKeyword => error("rename to keyword is forbidden", 2),
            RenameError::OfBuiltin => error("builtin type can't be renamed", 3),
            RenameError::ToPrevious => error("rename to old name is useless", 4),
            RenameError::OfNone => error("none symbol can't be renamed", 5),
            RenameError::ToBadType(t) => {
                error_from_string(format!("'{t}'is not correct type name"), 10)
            }
            RenameError::ToBadDependency(d) => {
                error_from_string(format!("'{d}'is not correct dependency name"), 11)
            }
            RenameError::ToBadField(f) => {
                error_from_string(format!("'{f}'is not correct field name"), 12)
            }
            RenameError::ToExistingType(t) => {
                error_from_string(format!("constructor or type '{t}' exist"), 20)
            }
            RenameError::ToExistingResource { t, r } => {
                error_from_string(format!("type '{t}' already contains '{r}'"), 21)
            }
            RenameError::OfAlias => error("alias rename is not supported yet", 98),
            RenameError::OfConstructor => error("constructors rename is not supported yet", 99),
        }
    }
}

impl<T> From<RenameError> for Result<T> {
    fn from(value: RenameError) -> Self {
        value.to_jsonrpc_error()
    }
}
