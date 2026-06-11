use crate::ast::elaborated::ConstructorNames;
use crate::ast::elaborated::{
    Constructor, Context, Module, Rec, Type, TypeExpression, ValueExpression, ValueExprs,
};
use crate::ast::operators::{OpCall, UnaryOp};
use std::hash::Hash;

pub fn map_module<A, B, F>(module: Module<A>, f: &F) -> Module<B>
where
    F: Fn(A) -> B,
    A: Clone + Hash + Eq,
    B: Clone + Ord + Hash + Eq,
{
    Module {
        constructors: module
            .constructors
            .into_iter()
            .map(|(name, ctor)| (f(name.clone()), map_constructor(ctor, f)))
            .collect(),
        types: module
            .types
            .into_iter()
            .map(|(name, ty)| (f(name.clone()), map_type(ty, f)))
            .collect(),
    }
}

fn map_type<A, B, F>(ty: Type<A>, f: &F) -> Type<B>
where
    F: Fn(A) -> B,
    A: Clone,
    B: Clone + Ord,
{
    Type {
        dependencies: map_context(ty.dependencies, f),
        constructor_names: match ty.constructor_names {
            ConstructorNames::OfMessage(name) => ConstructorNames::OfMessage(f(name)),
            ConstructorNames::OfEnum(names) => {
                ConstructorNames::OfEnum(names.into_iter().map(f).collect())
            }
        },
    }
}

fn map_constructor<A, B, F>(ctor: Constructor<A>, f: &F) -> Constructor<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    Constructor {
        implicits: map_context(ctor.implicits, f),
        fields: map_context(ctor.fields, f),
        result_type: map_type_expression(ctor.result_type, f),
    }
}

fn map_context<A, B, F>(ctx: Context<A>, f: &F) -> Context<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    ctx.into_iter()
        .map(|(name, ty)| (f(name), map_type_expression(ty, f)))
        .collect()
}

fn map_type_expression<A, B, F>(ty: TypeExpression<A>, f: &F) -> TypeExpression<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    match ty {
        TypeExpression::TypeExpression { name, dependencies } => TypeExpression::TypeExpression {
            name: f(name),
            dependencies: map_value_exprs(&dependencies, f),
        },
    }
}

fn map_value_exprs<A, B, F>(exprs: &ValueExprs<A>, f: &F) -> ValueExprs<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    Rec::from(
        exprs
            .iter()
            .cloned()
            .map(|e| map_value_expression(e, f))
            .collect::<Vec<_>>(),
    )
}

fn map_value_expression<A, B, F>(expr: ValueExpression<A>, f: &F) -> ValueExpression<B>
where
    F: Fn(A) -> B,
    A: Clone,
{
    match expr {
        ValueExpression::Variable { name, ty } => ValueExpression::Variable {
            name: f(name),
            ty: map_type_expression(ty, f),
        },
        ValueExpression::Constructor {
            name,
            implicits,
            arguments,
            result_type,
        } => ValueExpression::Constructor {
            name: f(name),
            implicits: map_value_exprs(&implicits, f),
            arguments: map_value_exprs(&arguments, f),
            result_type: map_type_expression(result_type, f),
        },
        ValueExpression::OpCall {
            op_call,
            result_type,
        } => ValueExpression::OpCall {
            op_call: map_op_call(op_call, f),
            result_type: map_type_expression(result_type, f),
        },
    }
}

fn map_op_call<A, B, F>(
    op_call: OpCall<A, Rec<ValueExpression<A>>>,
    f: &F,
) -> OpCall<B, Rec<ValueExpression<B>>>
where
    F: Fn(A) -> B,
    A: Clone,
{
    match op_call {
        OpCall::Literal(lit) => OpCall::Literal(lit),
        OpCall::Unary(op, expr) => {
            let op = match op {
                UnaryOp::Access(name) => UnaryOp::Access(f(name)),
                UnaryOp::Minus => UnaryOp::Minus,
                UnaryOp::Bang => UnaryOp::Bang,
            };
            OpCall::Unary(op, Rec::new(map_value_expression((*expr).clone(), f)))
        }
        OpCall::Binary(op, lhs, rhs) => OpCall::Binary(
            op,
            Rec::new(map_value_expression((*lhs).clone(), f)),
            Rec::new(map_value_expression((*rhs).clone(), f)),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::operators::{BinaryOp, Literal, UnaryOp};
    use indexmap::IndexMap;
    use std::collections::BTreeMap;

    fn suf(s: String) -> String {
        s + "_"
    }

    fn ty(name: &str) -> TypeExpression<String> {
        TypeExpression::TypeExpression {
            name: name.to_owned(),
            dependencies: Rec::new([]),
        }
    }

    fn var(name: &str) -> ValueExpression<String> {
        ValueExpression::Variable {
            name: name.to_owned(),
            ty: ty("T"),
        }
    }

    #[test]
    fn variable_maps_name_and_type() {
        assert_eq!(
            map_value_expression(var("x"), &suf),
            ValueExpression::Variable {
                name: "x_".to_owned(),
                ty: ty("T_")
            },
        );
    }

    #[test]
    fn unary_access_maps_field_name() {
        let expr = ValueExpression::OpCall {
            op_call: OpCall::Unary(UnaryOp::Access("f".to_owned()), Rec::new(var("x"))),
            result_type: ty("Int"),
        };
        assert_eq!(
            map_value_expression(expr, &suf),
            ValueExpression::OpCall {
                op_call: OpCall::Unary(
                    UnaryOp::Access("f_".to_owned()),
                    Rec::new(ValueExpression::Variable {
                        name: "x_".to_owned(),
                        ty: ty("T_")
                    }),
                ),
                result_type: ty("Int_"),
            }
        );
    }

    #[test]
    fn literal_preserves_value() {
        let expr = ValueExpression::OpCall {
            op_call: OpCall::Literal(Literal::Int(42)),
            result_type: ty("Int"),
        };
        assert_eq!(
            map_value_expression(expr, &suf),
            ValueExpression::OpCall {
                op_call: OpCall::Literal(Literal::Int(42)),
                result_type: ty("Int_"),
            }
        );
    }

    #[test]
    fn binary_maps_both_operands() {
        let expr = ValueExpression::OpCall {
            op_call: OpCall::Binary(BinaryOp::Plus, Rec::new(var("a")), Rec::new(var("b"))),
            result_type: ty("Int"),
        };
        assert_eq!(
            map_value_expression(expr, &suf),
            ValueExpression::OpCall {
                op_call: OpCall::Binary(
                    BinaryOp::Plus,
                    Rec::new(ValueExpression::Variable {
                        name: "a_".to_owned(),
                        ty: ty("T_")
                    }),
                    Rec::new(ValueExpression::Variable {
                        name: "b_".to_owned(),
                        ty: ty("T_")
                    }),
                ),
                result_type: ty("Int_"),
            }
        );
    }

    #[test]
    fn module_maps_all_names() {
        let module = Module {
            types: IndexMap::from([(
                "Foo".to_owned(),
                Type {
                    dependencies: vec![("n".to_owned(), ty("Nat"))],
                    constructor_names: ConstructorNames::OfMessage("Foo".to_owned()),
                },
            )]),
            constructors: BTreeMap::from([(
                "Foo".to_owned(),
                Constructor {
                    implicits: vec![],
                    fields: vec![("x".to_owned(), ty("Int"))],
                    result_type: ty("Foo"),
                },
            )]),
        };
        let result = map_module(module, &suf);
        assert!(result.types.contains_key("Foo_"));
        assert_eq!(
            result.types["Foo_"].dependencies,
            vec![("n_".to_owned(), ty("Nat_"))]
        );
        assert!(result.constructors.contains_key("Foo_"));
        assert_eq!(
            result.constructors["Foo_"].fields,
            vec![("x_".to_owned(), ty("Int_"))]
        );
        assert_eq!(result.constructors["Foo_"].result_type, ty("Foo_"));
    }
}
