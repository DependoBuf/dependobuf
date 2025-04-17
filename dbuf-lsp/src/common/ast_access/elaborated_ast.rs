//! Module exports:
//! * trait ElaboratedHelper - helpfull getters for Elaborated ast.
//! * type ElaboratedAst, wich implements ElaboratedHelper.
//!

use dbuf_core::ast::elaborated::*;

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
    /// returns any constructor of `type_name`, if type is constructable.
    fn get_any_constructor(&self, type_name: &str) -> Option<&Str>;
    /// returns if type or constructor with `name` exists.
    fn has_type_or_constructor(&self, name: &str) -> bool;
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

    fn get_any_constructor(&self, type_name: &str) -> Option<&Str> {
        let t = self.get_type(type_name);
        if let Some(t) = t {
            match &t.constructor_names {
                ConstructorNames::OfMessage(ctr) => {
                    return Some(ctr);
                }
                ConstructorNames::OfEnum(ctrs) => {
                    if let Some(f) = ctrs.first() {
                        return Some(f);
                    } else {
                        return None;
                    }
                }
            }
        }
        None
    }

    fn has_type_or_constructor(&self, name: &str) -> bool {
        if self.types.iter().any(|t| t.0 == name) {
            return true;
        }
        if self.types.iter().any(|t| t.0 == name) {
            return true;
        }
        if self.constructors.keys().any(|ctr| name == ctr) {
            return true;
        }
        false
    }

    fn get_constructor(&self, name: &str) -> Option<&Constructor<Str>> {
        self.constructors.get(name)
    }
}
