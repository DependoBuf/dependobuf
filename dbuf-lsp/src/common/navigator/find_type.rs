use dbuf_core::ast::parsed::{ExpressionNode, TypeDefinition, TypeExpression};

use crate::common::ast_access::{ElaboratedHelper, Loc, Str};

use super::{Navigator, Symbol};

fn get_type_in_type_expr(te: &TypeExpression<Loc, Str>) -> Symbol {
    if let ExpressionNode::FunCall { fun, args: _ } = &te.node {
        Symbol::Type(fun.to_string())
    } else {
        Symbol::None
    }
}

pub fn find_type_impl(navigator: &Navigator, symbol: &Symbol) -> Symbol {
    match symbol {
        Symbol::Type(t) => Symbol::Type(t.to_owned()),
        Symbol::Dependency { t, dependency } => {
            let dependencies = navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .map(|d| &d.data.dependencies);

            if let Some(dependencies) = dependencies {
                let te = dependencies
                    .iter()
                    .find(|d| d.name.as_ref() == dependency)
                    .map(|d| &d.data);
                if let Some(expr) = te {
                    get_type_in_type_expr(expr)
                } else {
                    Symbol::None
                }
            } else {
                Symbol::None
            }
        }
        Symbol::Field { constructor, field } => {
            let t = navigator.elaborated.get_constructor_type(constructor);
            let type_name = match t {
                Some(t) => t,
                None => return Symbol::None,
            };

            let body = navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == type_name)
                .map(|d| &d.data.body);

            match body {
                Some(TypeDefinition::Message(m)) => {
                    let te = m.iter().find(|f| f.name.as_ref() == field).map(|f| &f.data);

                    if let Some(expr) = te {
                        get_type_in_type_expr(expr)
                    } else {
                        Symbol::None
                    }
                }
                Some(TypeDefinition::Enum(branches)) => {
                    for b in branches.iter() {
                        let my_ctr = b
                            .constructors
                            .iter()
                            .find(|c| c.name.as_ref() == constructor);

                        if let Some(ctr) = my_ctr {
                            let te = ctr.iter().find(|f| f.name.as_ref() == field);

                            if let Some(expr) = te {
                                return get_type_in_type_expr(expr);
                            }
                            break;
                        }
                    }
                    Symbol::None
                }
                None => Symbol::None,
            }
        }
        Symbol::Constructor(_) => Symbol::None, // Not implemented
        Symbol::None => Symbol::None,
    }
}
