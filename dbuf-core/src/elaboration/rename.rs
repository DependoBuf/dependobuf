use crate::ast::elaborated::ConstructorNames;
use crate::ast::elaborated::{
    Constructor, Context, Module, Rec, Type, TypeExpression, ValueExpression, ValueExprs,
};
use crate::ast::operators::{OpCall, UnaryOp};
use std::collections::HashSet;
use std::hash::Hash;

pub trait Rename: Sized {
    #[must_use]
    fn with_suffix(&self, suffix: &str) -> Self;
}

impl Rename for String {
    fn with_suffix(&self, suffix: &str) -> Self {
        format!("{self}{suffix}")
    }
}

impl Rename for crate::arena::InternedString {
    fn with_suffix(&self, suffix: &str) -> Self {
        Self::from(format!("{}{suffix}", self.as_ref()))
    }
}

pub fn map_module<A, B, F>(module: Module<A>, f: &F) -> Module<B>
where
    F: Fn(A) -> B,
    A: Clone,
    B: Clone + Ord,
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

#[must_use]
pub fn add_suffix_context<Str>(context: Context<Str>, suffix: &str) -> Context<Str>
where
    Str: Clone + Eq + Hash + Rename,
{
    let implicit_names: HashSet<Str> = context.iter().map(|(name, _)| name.clone()).collect();

    let f = |name: Str| {
        if implicit_names.contains(&name) {
            name.with_suffix(suffix)
        } else {
            name
        }
    };

    map_context(context, &f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::elaborated::{
        Constructor, ConstructorNames, Type, TypeExpression, ValueExpression,
    };
    use crate::ast::operators::{BinaryOp, Literal, OpCall, UnaryOp};
    use std::collections::BTreeMap;

    fn nat_ty() -> TypeExpression<String> {
        TypeExpression::TypeExpression {
            name: "Nat".to_owned(),
            dependencies: Rec::new([]),
        }
    }

    fn int_ty() -> TypeExpression<String> {
        TypeExpression::TypeExpression {
            name: "Int".to_owned(),
            dependencies: Rec::new([]),
        }
    }

    fn bool_ty() -> TypeExpression<String> {
        TypeExpression::TypeExpression {
            name: "Bool".to_owned(),
            dependencies: Rec::new([]),
        }
    }

    #[test]
    fn string_rename_with_suffix() {
        assert_eq!("hello".to_owned().with_suffix("_dep"), "hello_dep");
    }

    #[test]
    fn map_module_renames_all_name_variants() {
        let var_n = || ValueExpression::Variable {
            name: "n".to_owned(),
            ty: int_ty(),
        };

        let ctor_dep = ValueExpression::Constructor {
            name: "Suc".to_owned(),
            implicits: Rec::new([]),
            arguments: Rec::new([var_n()]),
            result_type: nat_ty(),
        };
        let access_dep = ValueExpression::OpCall {
            op_call: OpCall::Unary(UnaryOp::Access("field".to_owned()), Rec::new(var_n())),
            result_type: int_ty(),
        };
        let minus_dep = ValueExpression::OpCall {
            op_call: OpCall::Unary(UnaryOp::Minus, Rec::new(var_n())),
            result_type: int_ty(),
        };
        let bang_dep = ValueExpression::OpCall {
            op_call: OpCall::Unary(
                UnaryOp::Bang,
                Rec::new(ValueExpression::Variable {
                    name: "b".to_owned(),
                    ty: bool_ty(),
                }),
            ),
            result_type: bool_ty(),
        };
        let binary_dep = ValueExpression::OpCall {
            op_call: OpCall::Binary(
                BinaryOp::Plus,
                Rec::new(var_n()),
                Rec::new(ValueExpression::OpCall {
                    op_call: OpCall::Literal(Literal::Int(1)),
                    result_type: int_ty(),
                }),
            ),
            result_type: int_ty(),
        };

        let module: Module<String> = Module {
            types: vec![
                (
                    "Nat".to_owned(),
                    Type {
                        dependencies: vec![],
                        constructor_names: ConstructorNames::OfEnum(
                            ["Zero".to_owned(), "Suc".to_owned()].into_iter().collect(),
                        ),
                    },
                ),
                (
                    "Msg".to_owned(),
                    Type {
                        dependencies: vec![("n".to_owned(), nat_ty())],
                        constructor_names: ConstructorNames::OfMessage("Msg".to_owned()),
                    },
                ),
            ],
            constructors: {
                let mut m = BTreeMap::new();
                m.insert(
                    "Ctor".to_owned(),
                    Constructor {
                        implicits: vec![("n".to_owned(), int_ty())],
                        fields: vec![("val".to_owned(), nat_ty())],
                        result_type: TypeExpression::TypeExpression {
                            name: "T".to_owned(),
                            dependencies: Rec::from(
                                [ctor_dep, access_dep, minus_dep, bang_dep, binary_dep].as_slice(),
                            ),
                        },
                    },
                );
                m
            },
        };

        let mapped = map_module(module, &|s: String| s + "_x");

        assert!(mapped.types.iter().any(|(n, _)| n == "Nat_x"));
        assert!(mapped.constructors.contains_key("Ctor_x"));

        let msg = mapped.types.iter().find(|(n, _)| n == "Msg_x").unwrap();
        assert!(matches!(&msg.1.constructor_names, ConstructorNames::OfMessage(n) if n == "Msg_x"));
    }
}
