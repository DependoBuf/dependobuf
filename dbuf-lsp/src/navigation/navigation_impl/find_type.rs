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
    Symbol::Type {
        type_name: name.to_string(),
    }
}

pub fn find_type_impl(navigator: &Navigator, symbol: Symbol) -> Symbol {
    match &symbol {
        Symbol::Type { type_name: _ } => symbol,
        Symbol::Dependency {
            type_name,
            dependency,
        } => {
            let elaborated = navigator.elaborated;
            let t = elaborated.get_type(type_name).unwrap_or_else(|| {
                panic!("dependency not found\n{symbol:#?}");
            });

            t.dependencies
                .iter()
                .find(|d| d.0 == dependency.as_ref())
                .map_or_else(
                    || {
                        panic!("dependency not found\n{symbol:#?}");
                    },
                    |d| get_type(&d.1),
                )
        }
        Symbol::Field {
            type_name: _,
            constructor,
            field,
        } => {
            let elaborated = navigator.elaborated;
            let cons = elaborated.get_constructor(constructor).unwrap_or_else(|| {
                panic!("field not found\n{symbol:#?}");
            });
            cons.fields
                .iter()
                .find(|f| f.0 == field.as_ref())
                .map_or_else(
                    || {
                        panic!("field not found\n{symbol:#?}");
                    },
                    |f| get_type(&f.1),
                )
        }
        Symbol::Alias {
            type_name,
            branch_id,
            alias,
        } => {
            let parsed = navigator.parsed;
            let elaborated = navigator.elaborated;
            let body = parsed
                .iter()
                .find(|d| d.name.as_ref() == type_name)
                .map(|d| &d.data.body);

            if let Some(TypeDefinition::Enum(e)) = body {
                let b = e.get(*branch_id).unwrap_or_else(|| {
                    panic!("alias not found\n{symbol:#?}");
                });
                let cons = b.constructors.first().unwrap_or_else(|| {
                    todo!(
                        "not implemented scenario: empty branch in enum \n{:#?}",
                        symbol
                    );
                });
                let cons_name = cons.name.as_ref();

                let cons = elaborated.get_constructor(cons_name).unwrap_or_else(|| {
                    panic!("alias not found\n{symbol:#?}");
                });
                cons.implicits
                    .iter()
                    .find(|i| i.0 == alias.as_ref())
                    .map_or_else(
                        || {
                            panic!("alias not found\n{symbol:#?}");
                        },
                        |i| get_type(&i.1),
                    )
            } else {
                panic!("alias not found\n{symbol:#?}");
            }
        }
        Symbol::Constructor {
            type_name: _,
            constructor,
        } => {
            let elaborated = navigator.elaborated;
            let type_name = elaborated
                .get_constructor_type(constructor)
                .unwrap_or_else(|| {
                    panic!("constructor not found\n{symbol:#?}");
                });
            Symbol::Type {
                type_name: type_name.to_string(),
            }
        }
        Symbol::None => Symbol::None,
    }
}
