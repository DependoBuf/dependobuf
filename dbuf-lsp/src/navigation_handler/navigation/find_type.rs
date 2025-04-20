// TODO: alias

use dbuf_core::ast::parsed::{ExpressionNode, TypeDefinition, TypeExpression};

use crate::common::ast_access::{ElaboratedHelper, Loc, Str};

use crate::common::navigator::Navigator;
use crate::common::navigator::Symbol;

fn get_type_in_type_expr(te: &TypeExpression<Loc, Str>) -> Symbol {
    if let ExpressionNode::FunCall { fun, args: _ } = &te.node {
        Symbol::Type(fun.to_string())
    } else {
        panic!("bad type expression");
    }
}

pub fn find_type_impl(navigator: &Navigator, symbol: Symbol) -> Symbol {
    match &symbol {
        Symbol::Type(_) => symbol,
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
                    return get_type_in_type_expr(expr);
                }
            }
            panic!("dependency not found\n{:#?}", symbol);
        }
        Symbol::Field { constructor, field } => {
            let elaborated = navigator.elaborated;
            let t = elaborated.get_constructor_type(constructor).unwrap();
            let body = navigator
                .parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .map(|d| &d.data.body);

            match body {
                Some(TypeDefinition::Message(m)) => {
                    let te = m.iter().find(|f| f.name.as_ref() == field).map(|f| &f.data);
                    if let Some(expr) = te {
                        return get_type_in_type_expr(expr);
                    }
                }
                Some(TypeDefinition::Enum(branches)) => {
                    for b in branches.iter() {
                        for c in b.constructors.iter() {
                            if c.name.as_ref() != constructor {
                                continue;
                            }
                            let te = c.iter().find(|f| f.name.as_ref() == field);
                            if let Some(expr) = te {
                                return get_type_in_type_expr(expr);
                            }
                            break;
                        }
                    }
                }
                None => {}
            };
            panic!("field not found\n{:#?}", symbol);
        }
        Symbol::Alias {
            t: _,
            branch_id: _,
            name: _,
        } => todo!(),
        Symbol::Constructor(_) => symbol,
        Symbol::None => Symbol::None,
    }
}
