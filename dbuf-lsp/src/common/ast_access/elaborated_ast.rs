//! Module exports:
//! * trait ElaboratedHelper - helpfull getters for Elaborated ast.
//! * type ElaboratedAst, wich implements ElaboratedHelper.
//!

use dbuf_core::ast::elaborated::*;

use crate::common::dbuf_language::get_bultin_types;

pub type Str = String;

pub type ElaboratedAst = Module<Str>;

/// Trait with getters for ElaboratedAst.
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
    /// returns if `type_name` is buildin type.
    fn is_buildin_type(&self, type_name: &str) -> bool;
    /// returns if `type_name` is message.
    fn is_message(&self, type_name: &str) -> bool;
    /// returns if `name` is dependency of `type_name`.
    fn is_type_dependency(&self, type_name: &str, name: &str) -> bool;
    /// returns if type has constructor 'name'.
    fn is_type_constructor(&self, type_name: &str, name: &str) -> bool;
    /// returns if `name` is field of `constructor`.
    fn is_constructor_field(&self, constructor_name: &str, name: &str) -> bool;
}

impl ElaboratedHelper for ElaboratedAst {
    fn get_constructor_type(&self, constructor_name: &str) -> Option<&str> {
        if let Some(ctr) = self.constructors.get(constructor_name) {
            if let Expression::Type {
                name,
                dependencies: _,
            } = &ctr.result_type
            {
                return Some(name);
            }
        }
        None
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

    fn is_buildin_type(&self, type_name: &str) -> bool {
        get_bultin_types().contains(type_name)
    }

    fn is_message(&self, type_name: &str) -> bool {
        if let Some(t) = self.get_type(type_name) {
            if let ConstructorNames::OfMessage(_) = t.constructor_names {
                return true;
            }
        }
        false
    }

    fn is_type_dependency(&self, type_name: &str, name: &str) -> bool {
        if let Some(t) = self.get_type(type_name) {
            t.dependencies.iter().any(|d| d.0 == name)
        } else {
            false
        }
    }

    fn is_type_constructor(&self, type_name: &str, name: &str) -> bool {
        if let Some(t) = self.get_type(type_name) {
            match &t.constructor_names {
                ConstructorNames::OfMessage(ctr) => ctr == name,
                ConstructorNames::OfEnum(btree_set) => btree_set.contains(name),
            }
        } else {
            false
        }
    }

    fn is_constructor_field(&self, constructor_name: &str, name: &str) -> bool {
        if let Some(c) = self.get_constructor(constructor_name) {
            c.fields.iter().any(|f| f.0 == name)
        } else {
            false
        }
    }
}
