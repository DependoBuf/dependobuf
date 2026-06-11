use crate::ast::elaborated as e;
use crate::elaboration::builtins::BuiltinType;
use crate::elaboration::{subst, type_of, unify};
use crate::error::elaborating::Error::{self, ArityMismatch};
use std::hash::Hash;

/// # Errors
pub fn application<Str>(
    constructor: &e::Constructor<Str>,
    arg: &e::ValueExpression<Str>,
    module: &e::Module<Str>,
) -> Result<e::Constructor<Str>, Error>
where
    Str: Clone + Eq + Ord + From<BuiltinType> + ToString + Hash,
{
    let e::Constructor {
        implicits,
        fields,
        result_type,
    } = constructor;

    let ((_var_name, field_type), rest_fields) = fields.split_first().ok_or(ArityMismatch {
        expected: 1,
        found: 0,
    })?;

    let arg_type = type_of(arg);
    let implicit_bindings = unify::unify_type(&arg_type, field_type, module)?;

    let new_fields = rest_fields
        .iter()
        .map(|(name, ty)| {
            let ty = subst::apply_bindings_to_type(ty.clone(), &implicit_bindings);
            (name.clone(), ty)
        })
        .collect();

    let new_result_type = subst::apply_bindings_to_type(result_type.clone(), &implicit_bindings);

    Ok(e::Constructor {
        implicits: implicits.clone(),
        fields: new_fields,
        result_type: new_result_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::elaborated::{self as e, ConstructorNames};
    use crate::error::elaborating::Error::ArityMismatch;
    use indexmap::IndexMap;
    use std::collections::BTreeMap;

    fn a_ty() -> e::TypeExpression<String> {
        e::TypeExpression::TypeExpression {
            name: "A".to_owned(),
            dependencies: e::Rec::new([]),
        }
    }

    fn b_ty(x: e::ValueExpression<String>) -> e::TypeExpression<String> {
        e::TypeExpression::TypeExpression {
            name: "B".to_owned(),
            dependencies: e::Rec::new([x]),
        }
    }

    fn var(name: &str) -> e::ValueExpression<String> {
        e::ValueExpression::Variable {
            name: name.to_owned(),
            ty: a_ty(),
        }
    }

    fn ctor_a() -> e::ValueExpression<String> {
        e::ValueExpression::Constructor {
            name: "CA".to_owned(),
            implicits: e::Rec::new([]),
            arguments: e::Rec::new([]),
            result_type: a_ty(),
        }
    }

    fn test_module() -> e::Module<String> {
        e::Module {
            types: IndexMap::from([
                (
                    "A".to_owned(),
                    e::Type {
                        dependencies: vec![],
                        constructor_names: ConstructorNames::OfMessage("A".to_owned()),
                    },
                ),
                (
                    "B".to_owned(),
                    e::Type {
                        dependencies: vec![("x".to_owned(), a_ty())],
                        constructor_names: ConstructorNames::OfMessage("B".to_owned()),
                    },
                ),
            ]),
            constructors: BTreeMap::new(),
        }
    }

    #[test]
    fn no_fields_returns_arity_error() {
        let ctor = e::Constructor {
            implicits: vec![],
            fields: vec![],
            result_type: a_ty(),
        };
        assert_eq!(
            application(&ctor, &ctor_a(), &test_module()).unwrap_err(),
            ArityMismatch {
                expected: 1,
                found: 0
            }
        );
    }

    #[test]
    fn applies_last_field() {
        let ctor = e::Constructor {
            implicits: vec![],
            fields: vec![("f".to_owned(), a_ty())],
            result_type: a_ty(),
        };
        let result = application(&ctor, &ctor_a(), &test_module()).unwrap();
        assert!(result.fields.is_empty());
        assert_eq!(result.result_type, a_ty());
    }

    #[test]
    fn substitutes_type_variable_into_remaining_fields() {
        let ctor = e::Constructor {
            implicits: vec![],
            fields: vec![
                ("f".to_owned(), b_ty(var("x"))),
                ("g".to_owned(), b_ty(var("x"))),
            ],
            result_type: b_ty(var("x")),
        };
        let arg = e::ValueExpression::Variable {
            name: "v".to_owned(),
            ty: b_ty(ctor_a()),
        };
        let result = application(&ctor, &arg, &test_module()).unwrap();
        assert_eq!(result.fields, vec![("g".to_owned(), b_ty(ctor_a()))]);
        assert_eq!(result.result_type, b_ty(ctor_a()));
    }
}
