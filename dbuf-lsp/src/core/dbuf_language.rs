//! Contains dbuf language specific information, like builtin types or constants.
//!

use std::{collections::HashSet, sync::OnceLock};

static BUILTIN_TYPES: OnceLock<HashSet<String>> = OnceLock::new();
static KEYWORDS: OnceLock<HashSet<String>> = OnceLock::new();

/// Returns builtint types set.
pub fn get_builtin_types() -> &'static HashSet<String> {
    BUILTIN_TYPES.get_or_init(|| {
        let mut m = HashSet::new();
        let types = ["Int", "String", "Bool", "Unsigned", "Float"];
        types.iter().for_each(|&s| {
            m.insert(s.to_string());
        });
        m
    })
}

/// Returns dbuf keywords set.
pub fn get_keywords() -> &'static HashSet<String> {
    KEYWORDS.get_or_init(|| {
        let mut m = HashSet::new();
        let types = ["message", "enum"];
        types.iter().for_each(|&s| {
            m.insert(s.to_string());
        });
        m
    })
}

/// Checks if `type_name` is correct name for Type or Constructor.
pub fn is_correct_type_name(type_name: &str) -> bool {
    let mut iterator = type_name.chars();
    if iterator.next().map(|c| !c.is_uppercase()).unwrap_or(true) {
        return false;
    }
    if !iterator.all(|c| c.is_alphanumeric()) {
        return false;
    }
    true
}

/// Checks if `field_name` is correct name for field.
pub fn is_correct_field_name(field_name: &str) -> bool {
    let mut iterator = field_name.chars();
    if iterator.next().map(|c| !c.is_lowercase()).unwrap_or(true) {
        return false;
    }
    if !iterator.all(|c| c.is_alphanumeric()) {
        return false;
    }
    true
}

/// Checks if `dependency_name` is correct name for dependency.
pub fn is_correct_dependency_name(dependency_name: &str) -> bool {
    is_correct_field_name(dependency_name)
}
