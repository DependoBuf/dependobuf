//! Contains dbuf language specific information, like builtin types or constants.
//!

use std::{collections::HashSet, sync::LazyLock};

use std::string::ToString;

static BUILTIN_TYPES: LazyLock<HashSet<String>> = LazyLock::new(|| {
    HashSet::from(["Int", "String", "Bool", "Unsigned", "Float"].map(ToString::to_string))
});
static KEYWORDS: LazyLock<HashSet<String>> =
    LazyLock::new(|| HashSet::from(["message", "enum"].map(ToString::to_string)));

/// Returns builtint types set.
pub fn get_builtin_types() -> &'static HashSet<String> {
    &BUILTIN_TYPES
}

/// Returns dbuf keywords set.
pub fn get_keywords() -> &'static HashSet<String> {
    &KEYWORDS
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
        if iterator.next().is_some_and(char::is_uppercase) && iterator.all(char::is_alphanumeric) {
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
        if iterator.next().is_some_and(char::is_lowercase) && iterator.all(char::is_alphanumeric) {
            Ok(FieldName { field: value })
        } else {
            Err(())
        }
    }
}
