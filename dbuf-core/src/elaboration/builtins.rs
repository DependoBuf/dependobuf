use crate::arena::InternedString;
use crate::ast::elaborated as e;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinType {
    Bool,
    Int,
    UInt,
    String,
}

impl BuiltinType {
    pub const ALL: &'static [Self] = &[Self::Bool, Self::Int, Self::UInt, Self::String];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bool => "Bool",
            Self::Int => "Int",
            Self::UInt => "UInt",
            Self::String => "String",
        }
    }
}

impl From<BuiltinType> for String {
    fn from(value: BuiltinType) -> Self {
        value.as_str().to_owned()
    }
}

impl From<BuiltinType> for InternedString {
    fn from(value: BuiltinType) -> Self {
        String::from(value).into()
    }
}

impl From<&str> for BuiltinType {
    fn from(value: &str) -> Self {
        match value {
            "Bool" => Self::Bool,
            "Int" => Self::Int,
            "UInt" => Self::UInt,
            "String" => Self::String,
            _ => unreachable!(),
        }
    }
}

#[must_use]
pub fn builtins<Str: Eq + Hash + From<BuiltinType> + Clone>() -> Vec<e::TypeExpression<Str>> {
    BuiltinType::ALL.iter().map(|bt| get_builtin(bt)).collect()
}

#[must_use]
pub fn builtins_module<Str: Eq + Hash + From<BuiltinType> + Clone>() -> e::Module<Str> {
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

#[must_use]
pub fn get_builtin<Str: From<BuiltinType>>(ty: &BuiltinType) -> e::TypeExpression<Str> {
    e::TypeExpression::TypeExpression {
        name: Str::from(*ty),
        dependencies: e::Rec::new([]),
    }
}

#[must_use]
pub fn is_builtin_type<Str: Clone + PartialEq + From<BuiltinType>>(
    ty: &e::TypeExpression<Str>,
) -> bool {
    BuiltinType::ALL.iter().any(|bt| ty == &get_builtin(bt))
}
