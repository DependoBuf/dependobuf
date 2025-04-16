//! Find definition of symbol
//!

use dbuf_core::ast::parsed::TypeDefinition;
use tower_lsp::lsp_types::Range;

use crate::common::ast_access::ElaboratedHelper;
use crate::common::ast_access::LocationHelpers;

use super::Navigator;
use super::Symbol;

pub fn find_definition_impl(navigator: &Navigator, symbol: &Symbol) -> Option<Range> {
    match symbol {
        Symbol::Type(t) => navigator
            .parsed
            .iter()
            .find(|d| d.name.as_ref() == t)
            .map(|d| d.name.get_location().to_lsp()),
        Symbol::Dependency { t, dependency } => {
            let dep = navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .map(|d| &d.data.dependencies);
            if let Some(dependencies) = dep {
                dependencies
                    .iter()
                    .find(|d| d.name.as_ref() == dependency)
                    .map(|d| d.name.get_location().to_lsp())
            } else {
                None
            }
        }
        Symbol::Field { constructor, field } => {
            let t = navigator.elaborated.get_constructor_type(constructor);
            let type_name = match t {
                Some(t) => t,
                None => return None,
            };

            let body = navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == type_name)
                .map(|d| &d.data.body);

            match body {
                Some(TypeDefinition::Message(m)) => m
                    .iter()
                    .find(|f| f.name.as_ref() == field)
                    .map(|f| f.name.get_location().to_lsp()),
                Some(TypeDefinition::Enum(_e)) => {
                    panic!("enums are not supported")
                }
                None => None,
            }
        }
        Symbol::Constructor(_) => None, // Not implemented
        Symbol::None => None,
    }
}
