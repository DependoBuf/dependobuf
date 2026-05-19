use crate::ast::elaborated as e;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

const BUILTIN_NAMES: &[&str] = &["Bool", "Int", "UInt", "String"];

/// Return type expressions for builtin types
#[must_use]
pub fn builtins<Str: Eq + Hash + From<String> + Clone>() -> Vec<e::TypeExpression<Str>> {
    BUILTIN_NAMES
        .iter()
        .map(|builtin| e::TypeExpression::TypeExpression {
            name: Str::from((*builtin).to_string()),
            dependencies: e::Rec::new([]),
        })
        .collect()
}

/// Return module only with builtin type
#[must_use]
pub fn builtins_module<Str: Eq + Hash + From<String> + Clone>() -> e::Module<Str> {
    e::Module {
        types: builtins::<Str>()
            .into_iter()
            .map(|e::TypeExpression::TypeExpression { name, .. }| {
                (
                    name,
                    e::Type {
                        dependencies: vec![],
                        constructor_names: e::ConstructorNames::OfEnum(BTreeSet::new()),
                    },
                )
            })
            .collect(),
        constructors: BTreeMap::new(),
    }
}

/// Create builtin type expression by name.
#[must_use]
pub fn get_builtin<Str: From<String>>(type_name: &str) -> e::TypeExpression<Str> {
    e::TypeExpression::TypeExpression {
        name: type_name.to_string().into(),
        dependencies: e::Rec::new([]),
    }
}
