/// Find type of symbol.
///
use dbuf_core::ast::elaborated::TypeExpression;

use dbuf_core::ast::parsed::TypeDefinition;

use crate::core::ast_access::ElaboratedHelper;

use crate::core::navigator::Navigator;
use crate::core::navigator::Symbol;

fn get_type(te: &TypeExpression<String>) -> Symbol {
    let TypeExpression::TypeExpression {
        name,
        dependencies: _,
    } = te;
    Symbol::Type(name.to_string())
}

pub fn find_type_impl(navigator: &Navigator, symbol: Symbol) -> Symbol {
    match &symbol {
        Symbol::Type(_) => symbol,
        Symbol::Dependency { t, dependency } => {
            let elaborated = navigator.elaborated;
            let t = elaborated.get_type(t).unwrap_or_else(|| {
                panic!("dependency not found\n{:#?}", symbol);
            });

            t.dependencies
                .iter()
                .find(|d| d.0 == dependency.as_ref())
                .map(|d| get_type(&d.1))
                .unwrap_or_else(|| {
                    panic!("alias not found\n{:#?}", symbol);
                })
        }
        Symbol::Field { constructor, field } => {
            let elaborated = navigator.elaborated;
            let cons = elaborated.get_constructor(constructor).unwrap_or_else(|| {
                panic!("field not found\n{:#?}", symbol);
            });
            cons.fields
                .iter()
                .find(|f| f.0 == field.as_ref())
                .map(|f| get_type(&f.1))
                .unwrap_or_else(|| {
                    panic!("field not found\n{:#?}", symbol);
                })
        }
        Symbol::Alias { t, branch_id, name } => {
            let parsed = navigator.parsed;
            let elaborated = navigator.elaborated;
            let body = parsed
                .iter()
                .find(|d| d.name.as_ref() == t)
                .map(|d| &d.data.body);

            if let Some(TypeDefinition::Enum(e)) = body {
                let b = e.get(*branch_id).unwrap_or_else(|| {
                    panic!("alias not found\n{:#?}", symbol);
                });
                let cons = b.constructors.first().unwrap_or_else(|| {
                    panic!("alias not found\n{:#?}", symbol);
                });
                let cons_name = cons.name.as_ref();

                let cons = elaborated.get_constructor(cons_name).unwrap_or_else(|| {
                    panic!("alias not found\n{:#?}", symbol);
                });
                cons.implicits
                    .iter()
                    .find(|i| i.0 == name.as_ref())
                    .map(|i| get_type(&i.1))
                    .unwrap_or_else(|| {
                        panic!("alias not found\n{:#?}", symbol);
                    })
            } else {
                panic!("alias not found\n{:#?}", symbol);
            }
        }
        Symbol::Constructor(cons) => {
            let elaborated = navigator.elaborated;
            let type_name = elaborated.get_constructor_type(cons).unwrap_or_else(|| {
                panic!("constructor not found\n{:#?}", symbol);
            });
            Symbol::Type(type_name.to_string())
        }
        Symbol::None => Symbol::None,
    }
}
