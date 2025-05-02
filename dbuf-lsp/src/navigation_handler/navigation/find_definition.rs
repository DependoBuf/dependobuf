//! Find definition of symbol
//!

use dbuf_core::ast::parsed::Pattern;
use dbuf_core::ast::parsed::PatternNode;
use dbuf_core::ast::parsed::TypeDefinition;
use tower_lsp::lsp_types::Range;

use crate::core::ast_access::ElaboratedHelper;
use crate::core::ast_access::LocStringHelper;
use crate::core::ast_access::LocationHelpers;
use crate::core::ast_access::{Loc, Str};
use crate::core::dbuf_language::get_bultin_types;

use crate::core::navigator::Navigator;
use crate::core::navigator::Symbol;

pub fn find_definition_impl(navigator: &Navigator, symbol: &Symbol) -> Option<Range> {
    match symbol {
        Symbol::Type(t) => {
            if get_bultin_types().contains(t) {
                return None;
            }
            navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .map(|d| d.name.get_location().to_lsp())
                .unwrap_or_else(|| {
                    panic!("type not found\n{:#?}", symbol);
                })
                .into()
        }
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
                    .unwrap_or_else(|| {
                        panic!("dependency not found\n{:#?}", symbol);
                    })
                    .into()
            } else {
                panic!("dependency not found\n{:#?}", symbol);
            }
        }
        Symbol::Field { constructor, field } => {
            let elaborated = navigator.elaborated;
            let t = elaborated
                .get_constructor_type(constructor)
                .unwrap_or_else(|| {
                    panic!("field not found\n{:#?}", symbol);
                });
            let body = navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .map(|d| &d.data.body);

            match body {
                Some(TypeDefinition::Message(m)) => {
                    return m
                        .iter()
                        .find(|f| f.name.as_ref() == field)
                        .map(|f| f.name.get_location().to_lsp())
                        .unwrap_or_else(|| {
                            panic!("field not found\n{:#?}", symbol);
                        })
                        .into()
                }
                Some(TypeDefinition::Enum(branches)) => {
                    for b in branches.iter() {
                        for c in b.constructors.iter() {
                            if c.name.as_ref() != constructor {
                                continue;
                            }
                            return c
                                .iter()
                                .find(|f| f.name.as_ref() == field)
                                .map(|f| f.name.get_location().to_lsp())
                                .unwrap_or_else(|| {
                                    panic!("field not found\n{:#?}", symbol);
                                })
                                .into();
                        }
                    }
                }
                None => {}
            }
            panic!("field not found\n{:#?}", symbol);
        }
        Symbol::Alias { t, branch_id, name } => {
            let parsed = navigator.parsed;
            let body = parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .map(|d| &d.data.body);

            if let Some(TypeDefinition::Enum(e)) = body {
                let b = e.get(*branch_id).unwrap_or_else(|| {
                    panic!("alias not found\n{:#?}", symbol);
                });

                for p in b.patterns.iter() {
                    let ans = find_alias_in_pattern(p, name);
                    if ans.is_some() {
                        return ans;
                    }
                }
            }
            panic!("alias not found\n{:#?}", symbol);
        }
        Symbol::Constructor(constructor) => {
            let elaborated = navigator.elaborated;
            let t = elaborated
                .get_constructor_type(constructor)
                .unwrap_or_else(|| {
                    panic!("constructor not found\n{:#?}", symbol);
                });

            let declaration = navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .unwrap_or_else(|| {
                    panic!("constructor not found\n{:#?}", symbol);
                });

            match &declaration.body {
                TypeDefinition::Message(_) => {
                    panic!("symbol::constructor shouldn't be returned for messages");
                }
                TypeDefinition::Enum(branches) => {
                    for b in branches.iter() {
                        for c in b.constructors.iter() {
                            if c.name.as_ref() != constructor {
                                continue;
                            }
                            return c.name.get_location().to_lsp().into();
                        }
                    }
                }
            }
            panic!("constructor not found\n{:#?}", symbol);
        }
        Symbol::None => None,
    }
}

fn find_alias_in_pattern(p: &Pattern<Loc, Str>, alias: &String) -> Option<Range> {
    match &p.node {
        PatternNode::ConstructorCall { name: _, fields } => {
            for f in fields.iter().rev() {
                let ans = find_alias_in_pattern(&f.data, alias);
                if ans.is_some() {
                    return ans;
                }
            }
            None
        }
        PatternNode::Variable { name } => {
            if name.as_ref() == alias {
                name.get_location().to_lsp().into()
            } else {
                None
            }
        }
        PatternNode::Literal(_) => None,
        PatternNode::Underscore => None,
    }
}
