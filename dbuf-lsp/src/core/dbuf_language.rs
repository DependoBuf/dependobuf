//! Contains dbuf language specific information, like builtin types or constants.
//!

use std::{collections::HashSet, sync::OnceLock};

static BUILTIN_TYPES: OnceLock<HashSet<String>> = OnceLock::new();
static KEYWORDS: OnceLock<HashSet<String>> = OnceLock::new();

/// Returns builtint types set.
pub fn get_builtin_types() -> &'static HashSet<String> {
    BUILTIN_TYPES.get_or_init(|| {
        HashSet::from(["Int", "String", "Bool", "Unsigned", "Float"].map(|t| t.to_string()))
    })
}

/// Returns dbuf keywords set.
pub fn get_keywords() -> &'static HashSet<String> {
    KEYWORDS.get_or_init(|| HashSet::from(["message", "enum"].map(|t| t.to_string())))
}

/// Checks if `type_name` is correct name for Type or Constructor.
pub fn is_correct_type_name(type_name: &str) -> bool {
    let mut iterator = type_name.chars();
    iterator.next().map(char::is_uppercase).unwrap_or(false) && iterator.all(char::is_alphanumeric)
}

/// Checks if `field_name` is correct name for field.
pub fn is_correct_field_name(field_name: &str) -> bool {
    let mut iterator = field_name.chars();
    iterator.next().map(char::is_lowercase).unwrap_or(false) && iterator.all(char::is_alphanumeric)
}

/// Checks if `dependency_name` is correct name for dependency.
pub fn is_correct_dependency_name(dependency_name: &str) -> bool {
    is_correct_field_name(dependency_name)
}
