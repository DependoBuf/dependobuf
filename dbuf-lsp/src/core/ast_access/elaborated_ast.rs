//! Module exports:
//! * trait `ElaboratedHelper` - helpfull getters for elaborated ast.
//! * type `ElaboratedAst`, wich implements `ElaboratedHelper`.
//!

use dbuf_core::ast::elaborated::*;

use crate::core::dbuf_language::get_builtin_types;

pub type Str = String;

pub type ElaboratedAst = Module<Str>;

/// Trait with getters for `ElaboratedAst`.
pub trait ElaboratedHelper {
    /// returns type name of `constructor_name`.
    fn get_constructor_type(&self, constructor_name: &str) -> Option<&str>;
    /// returns Type by its `name`.
    fn get_type(&self, name: &str) -> Option<&Type<Str>>;
    /// returns Constructor by its `name`.
    fn get_constructor(&self, name: &str) -> Option<&Constructor<Str>>;
    /// returns if type with `name` exists.
    fn has_type(&self, name: &str) -> bool;
    /// returns if constructor with `name` exists.
    fn has_constructor(&self, name: &str) -> bool;
    /// returns if type or constructor with `name` exists.
    fn has_type_or_constructor(&self, name: &str) -> bool;
    /// returns if `type_name` is builtin type.
    fn is_builtin_type(&self, type_name: &str) -> bool;
    /// returns if `type_name` is message.
    fn is_message(&self, type_name: &str) -> bool;
    /// returns if `name` is dependency of `type_name`.
    fn is_type_dependency(&self, type_name: &str, name: &str) -> bool;
    /// returns if type has constructor 'name'.
    fn is_type_constructor(&self, type_name: &str, name: &str) -> bool;
    /// returns if `name` is field of `constructor`.
    fn is_constructor_field(&self, constructor_name: &str, name: &str) -> bool;
    /// returns if `name` is implicit of `constructor`.
    fn is_constructor_implicit(&self, constructor_name: &str, name: &str) -> bool;
}

impl ElaboratedHelper for ElaboratedAst {
    fn get_constructor_type(&self, constructor_name: &str) -> Option<&str> {
        self.constructors.get(constructor_name).map(|ctr| {
            let TypeExpression::TypeExpression {
                name,
                dependencies: _,
            } = &ctr.result_type;
            name.as_ref()
        })
    }

    fn get_type(&self, name: &str) -> Option<&Type<Str>> {
        self.types
            .iter()
            .find(|(type_name, _)| type_name == name)
            .map(|(_, type_definition)| type_definition)
    }

    fn has_type(&self, name: &str) -> bool {
        self.types.iter().any(|t| t.0 == name)
    }

    fn has_constructor(&self, name: &str) -> bool {
        self.constructors.keys().any(|ctr| name == ctr)
    }

    fn has_type_or_constructor(&self, name: &str) -> bool {
        self.has_type(name) || self.has_constructor(name)
    }

    fn get_constructor(&self, name: &str) -> Option<&Constructor<Str>> {
        self.constructors.get(name)
    }

    fn is_builtin_type(&self, type_name: &str) -> bool {
        get_builtin_types().contains(type_name)
    }

    fn is_message(&self, type_name: &str) -> bool {
        self.get_type(type_name)
            .is_some_and(|t| match &t.constructor_names {
                ConstructorNames::OfMessage(_) => true,
                ConstructorNames::OfEnum(_) => false,
            })
    }

    fn is_type_dependency(&self, type_name: &str, name: &str) -> bool {
        self.get_type(type_name)
            .is_some_and(|t| t.dependencies.iter().any(|d| d.0 == name))
    }

    fn is_type_constructor(&self, type_name: &str, name: &str) -> bool {
        self.get_type(type_name)
            .is_some_and(|t| match &t.constructor_names {
                ConstructorNames::OfMessage(ctr) => ctr == name,
                ConstructorNames::OfEnum(btree_set) => btree_set.contains(name),
            })
    }

    fn is_constructor_field(&self, constructor_name: &str, name: &str) -> bool {
        self.get_constructor(constructor_name)
            .is_some_and(|c| c.fields.iter().any(|f| f.0 == name))
    }

    fn is_constructor_implicit(&self, constructor_name: &str, name: &str) -> bool {
        self.get_constructor(constructor_name)
            .is_some_and(|c| c.implicits.iter().any(|f| f.0 == name))
    }
}
