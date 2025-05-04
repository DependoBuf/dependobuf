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

/// Type, that contains correct `type name`.
#[derive(Clone, Copy)]
pub struct TypeName<'a> {
    name: &'a str,
}

impl TypeName<'_> {
    pub fn get(&self) -> &str {
        self.name
    }
}

impl<'a> TryFrom<&'a str> for TypeName<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut iterator = value.chars();
        if iterator.next().map(char::is_uppercase).unwrap_or(false)
            && iterator.all(char::is_alphanumeric)
        {
            Ok(TypeName { name: value })
        } else {
            Err(())
        }
    }
}

/// Type, that contains correct `field name`.
#[derive(Clone, Copy)]
pub struct FieldName<'a> {
    field: &'a str,
}

impl FieldName<'_> {
    pub fn get(&self) -> &str {
        self.field
    }
}

impl<'a> TryFrom<&'a str> for FieldName<'a> {
    type Error = ();

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        let mut iterator = value.chars();
        if iterator.next().map(char::is_lowercase).unwrap_or(false)
            && iterator.all(char::is_alphanumeric)
        {
            Ok(FieldName { field: value })
        } else {
            Err(())
        }
    }
}
