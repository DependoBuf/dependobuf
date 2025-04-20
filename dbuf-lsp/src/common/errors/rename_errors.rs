use std::borrow::Cow;

use tower_lsp::jsonrpc::Error;
use tower_lsp::jsonrpc::ErrorCode::ServerError;
use tower_lsp::jsonrpc::Result;

/// Returns param is incorrect error.
fn bad_rename_error(text: &'static str, code: i64) -> Error {
    assert!((0..100).contains(&code));
    Error {
        code: ServerError(10200 + code),
        message: Cow::Borrowed(text),
        data: None,
    }
}

fn bad_rename_from_string(text: String, code: i64) -> Error {
    assert!((0..100).contains(&code));
    Error {
        code: ServerError(10200 + code),
        message: Cow::Owned(text),
        data: None,
    }
}

pub fn rename_to_empty_error<T>() -> Result<T> {
    Err(bad_rename_error("rename to empty string", 0))
}

pub fn rename_to_builtin_type_error<T>() -> Result<T> {
    Err(bad_rename_error("rename to builtin type is forbidden", 1))
}

pub fn rename_to_keyword_error<T>() -> Result<T> {
    Err(bad_rename_error("rename to keyword is forbidden", 2))
}

pub fn rename_of_buildin_type_error<T>() -> Result<T> {
    Err(bad_rename_error("buildin type can't be renamed", 3))
}

pub fn rename_to_old_error<T>() -> Result<T> {
    Err(bad_rename_error("rename to old name is useless", 4))
}

pub fn rename_none_symbol_error<T>() -> Result<T> {
    Err(bad_rename_error("none symbol can't be renamed", 5))
}

pub fn rename_to_bad_type_error<T>(new_name: &str) -> Result<T> {
    Err(bad_rename_from_string(
        format!("'{}'is not correct type name", new_name),
        10,
    ))
}

pub fn rename_to_bad_dependency_error<T>(new_name: &str) -> Result<T> {
    Err(bad_rename_from_string(
        format!("'{}'is not correct dependency name", new_name),
        11,
    ))
}

pub fn rename_to_bad_field_error<T>(new_name: &str) -> Result<T> {
    Err(bad_rename_from_string(
        format!("'{}'is not correct field name", new_name),
        12,
    ))
}

pub fn rename_to_existing_type_error<T>(new_name: &str) -> Result<T> {
    Err(bad_rename_from_string(
        format!("constructor or type '{}' exist", new_name),
        20,
    ))
}

pub fn rename_to_existing_resource_error<T>(type_name: &str, new_field_name: &str) -> Result<T> {
    Err(bad_rename_from_string(
        format!("type '{}' already contains '{}'", type_name, new_field_name),
        21,
    ))
}

pub fn rename_of_alias_error<T>() -> Result<T> {
    Err(bad_rename_error("alias rename is not supported yet", 98))
}

pub fn rename_of_constructor_error<T>() -> Result<T> {
    Err(bad_rename_error(
        "constructors rename is not supported yet",
        99,
    ))
}
