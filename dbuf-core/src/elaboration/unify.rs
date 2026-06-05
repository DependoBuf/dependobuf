use crate::ast::elaborated as e;
use crate::ast::operators as o;
use crate::elaboration::builtins::BuiltinType;
use crate::elaboration::subst;
use crate::error::elaborating::Error;
use crate::error::elaborating::Error::{
    ArityMismatch, ConflictingBinding, ConstructorMismatch, LiteralMismatch, OperatorTypeMismatch,
    TypeMismatch,
};
use std::hash::Hash;

pub type Bindings<Str> = Vec<(Str, e::ValueExpression<Str>)>;

/// # Errors
pub fn unify_type<Str>(
    a: &e::TypeExpression<Str>,
    b: &e::TypeExpression<Str>,
    module: &e::Module<Str>,
) -> Result<Bindings<Str>, Error>
where
    Str: Clone + Eq + Ord + Hash + From<BuiltinType> + ToString,
{
    let e::TypeExpression::TypeExpression {
        name: name_a,
        dependencies: deps_a,
    } = a;
    let e::TypeExpression::TypeExpression {
        name: name_b,
        dependencies: deps_b,
    } = b;

    if name_a != name_b {
        return Err(TypeMismatch);
    }

    let declared_arity = module
        .types
        .get(name_a)
        .map_or(0, |ty| ty.dependencies.len());

    if deps_a.len() != declared_arity || deps_b.len() != declared_arity {
        return Err(ArityMismatch {
            expected: declared_arity,
            found: deps_a.len(),
        });
    }

    unify_args(
        deps_a,
        &mut (deps_b.iter().cloned().collect::<Vec<_>>()),
        module,
    )
}

/// # Errors
pub fn unify_value<Str>(
    a: &e::ValueExpression<Str>,
    b: &e::ValueExpression<Str>,
    module: &e::Module<Str>,
) -> Result<Bindings<Str>, Error>
where
    Str: Clone + Eq + Ord + Hash + From<BuiltinType> + ToString,
{
    match (a, b) {
        (
            e::ValueExpression::Variable { name: x, ty: ty_x },
            e::ValueExpression::Variable { name: y, ty: ty_y },
        ) if x == y => unify_type(ty_x, ty_y, module),
        (
            e::ValueExpression::Variable { name: x, ty: ty_x },
            e::ValueExpression::Variable { name: y, .. },
        ) => Ok(vec![(
            y.clone(),
            e::ValueExpression::Variable {
                name: x.clone(),
                ty: ty_x.clone(),
            },
        )]),
        (e::ValueExpression::Variable { .. }, _) => Err(TypeMismatch),

        (other, e::ValueExpression::Variable { name, .. }) => {
            Ok(vec![(name.clone(), other.clone())])
        }

        (
            e::ValueExpression::Constructor {
                name: n1,
                implicits: i1,
                arguments: a1,
                ..
            },
            e::ValueExpression::Constructor {
                name: n2,
                implicits: i2,
                arguments: a2,
                ..
            },
        ) => {
            if n1 != n2 {
                return Err(ConstructorMismatch(n1.to_string(), n2.to_string()));
            }
            let left: Vec<_> = i1.iter().chain(a1.iter()).cloned().collect();
            let mut right: Vec<_> = i2.iter().chain(a2.iter()).cloned().collect();
            unify_args(&left, &mut right, module)
        }

        (
            e::ValueExpression::OpCall { op_call: op_a, .. },
            e::ValueExpression::OpCall { op_call: op_b, .. },
        ) => unify_op_call(op_a, op_b, module),

        _ => Err(TypeMismatch),
    }
}

fn unify_args<Str>(
    left: &[e::ValueExpression<Str>],
    right: &mut [e::ValueExpression<Str>],
    module: &e::Module<Str>,
) -> Result<Bindings<Str>, Error>
where
    Str: Clone + Eq + Ord + Hash + From<BuiltinType> + ToString,
{
    let mut acc = vec![];

    for i in 0..left.len() {
        let bindings = unify_value(&left[i], &right[i], module)?;

        for arg in &mut right[i + 1..] {
            *arg = subst::apply_bindings(arg.clone(), &bindings);
        }

        extend_bindings(&mut acc, bindings)?;
    }

    Ok(acc)
}

fn unify_op_call<Str>(
    a: &o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>,
    b: &o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>,
    module: &e::Module<Str>,
) -> Result<Bindings<Str>, Error>
where
    Str: Clone + Eq + Ord + Hash + From<BuiltinType> + ToString,
{
    match (a, b) {
        (o::OpCall::Literal(la), o::OpCall::Literal(lb)) => {
            if la == lb {
                Ok(vec![])
            } else {
                Err(LiteralMismatch(la.clone(), lb.clone()))
            }
        }
        (o::OpCall::Unary(op_a, expr_a), o::OpCall::Unary(op_b, expr_b)) => {
            if op_a != op_b {
                return Err(OperatorTypeMismatch);
            }
            unify_value(expr_a, expr_b, module)
        }
        (o::OpCall::Binary(op_a, la, ra), o::OpCall::Binary(op_b, lb, rb)) => {
            if op_a != op_b {
                return Err(OperatorTypeMismatch);
            }
            unify_args(
                &[(**la).clone(), (**ra).clone()],
                &mut [(**lb).clone(), (**rb).clone()],
                module,
            )
        }
        _ => Err(OperatorTypeMismatch),
    }
}

fn extend_bindings<Str>(acc: &mut Bindings<Str>, new: Bindings<Str>) -> Result<(), Error>
where
    Str: Clone + Eq + ToString,
{
    for (name, val) in new {
        match acc.iter().find(|(n, _)| n == &name) {
            Some((_, existing)) if existing == &val => {}
            Some(_) => return Err(ConflictingBinding(name.to_string())),
            None => acc.push((name, val)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::elaborated::{self as e, ConstructorNames};
    use crate::ast::operators::Literal;
    use crate::elaboration::builtins;
    use crate::error::elaborating::Error::{
        ArityMismatch, ConstructorMismatch, LiteralMismatch, OperatorTypeMismatch, TypeMismatch,
    };
    use std::collections::BTreeMap;

    fn nat_ty() -> e::TypeExpression<String> {
        e::TypeExpression::TypeExpression {
            name: "Nat".to_owned(),
            dependencies: e::Rec::new([]),
        }
    }

    fn vec_ty(n: e::ValueExpression<String>) -> e::TypeExpression<String> {
        e::TypeExpression::TypeExpression {
            name: "Vec".to_owned(),
            dependencies: e::Rec::new([n]),
        }
    }

    fn var(name: &str) -> e::ValueExpression<String> {
        e::ValueExpression::Variable {
            name: name.to_owned(),
            ty: nat_ty(),
        }
    }

    fn zero() -> e::ValueExpression<String> {
        e::ValueExpression::Constructor {
            name: "Zero".to_owned(),
            implicits: e::Rec::new([]),
            arguments: e::Rec::new([]),
            result_type: nat_ty(),
        }
    }

    fn suc(n: e::ValueExpression<String>) -> e::ValueExpression<String> {
        e::ValueExpression::Constructor {
            name: "Suc".to_owned(),
            implicits: e::Rec::new([]),
            arguments: e::Rec::new([n]),
            result_type: nat_ty(),
        }
    }

    fn lit_int(v: i64) -> e::ValueExpression<String> {
        e::ValueExpression::OpCall {
            op_call: o::OpCall::Literal(Literal::Int(v)),
            result_type: builtins::get_builtin(&BuiltinType::Int),
        }
    }

    fn neg(val: e::ValueExpression<String>) -> e::ValueExpression<String> {
        e::ValueExpression::OpCall {
            op_call: o::OpCall::Unary(o::UnaryOp::Minus, e::Rec::new(val)),
            result_type: builtins::get_builtin(&BuiltinType::Int),
        }
    }

    fn bang(val: e::ValueExpression<String>) -> e::ValueExpression<String> {
        e::ValueExpression::OpCall {
            op_call: o::OpCall::Unary(o::UnaryOp::Bang, e::Rec::new(val)),
            result_type: builtins::get_builtin(&BuiltinType::Bool),
        }
    }

    fn bin_op(
        op: o::BinaryOp,
        ty: BuiltinType,
        a: e::ValueExpression<String>,
        b: e::ValueExpression<String>,
    ) -> e::ValueExpression<String> {
        e::ValueExpression::OpCall {
            op_call: o::OpCall::Binary(op, e::Rec::new(a), e::Rec::new(b)),
            result_type: builtins::get_builtin(&ty),
        }
    }

    fn add_op(
        a: e::ValueExpression<String>,
        b: e::ValueExpression<String>,
    ) -> e::ValueExpression<String> {
        bin_op(o::BinaryOp::Plus, BuiltinType::Int, a, b)
    }

    fn sub_op(
        a: e::ValueExpression<String>,
        b: e::ValueExpression<String>,
    ) -> e::ValueExpression<String> {
        bin_op(o::BinaryOp::Minus, BuiltinType::Int, a, b)
    }

    fn test_module() -> e::Module<String> {
        e::Module {
            types: [
                (
                    "Nat".to_owned(),
                    e::Type {
                        dependencies: vec![],
                        constructor_names: ConstructorNames::OfEnum(
                            ["Zero", "Suc"].map(str::to_owned).into_iter().collect(),
                        ),
                    },
                ),
                (
                    "Vec".to_owned(),
                    e::Type {
                        dependencies: vec![("n".to_owned(), nat_ty())],
                        constructor_names: ConstructorNames::OfEnum(
                            ["Nil", "Cons"].map(str::to_owned).into_iter().collect(),
                        ),
                    },
                ),
            ]
            .into_iter()
            .collect(),
            constructors: BTreeMap::new(),
        }
    }

    #[test]
    fn same_binds() {
        let m = test_module();
        assert_eq!(unify_value(&var("x"), &var("x"), &m), Ok(vec![]));
    }

    #[test]
    fn left_binds() {
        let m = test_module();
        assert_eq!(unify_value(&var("x"), &zero(), &m), Err(TypeMismatch));
    }

    #[test]
    fn right_binds() {
        let m = test_module();
        assert_eq!(
            unify_value(&zero(), &var("y"), &m),
            Ok(vec![("y".to_owned(), zero())])
        );
    }

    #[test]
    fn var_binds() {
        let m = test_module();
        assert_eq!(
            unify_value(&var("x"), &var("y"), &m),
            Ok(vec![("y".to_owned(), var("x"))])
        );
    }

    fn pair(
        a: e::ValueExpression<String>,
        b: e::ValueExpression<String>,
    ) -> e::ValueExpression<String> {
        e::ValueExpression::Constructor {
            name: "Pair".to_owned(),
            implicits: e::Rec::new([]),
            arguments: e::Rec::new([a, b]),
            result_type: nat_ty(),
        }
    }

    #[test]
    fn value_conflicting_bindings() {
        let m = test_module();
        assert_eq!(
            unify_value(&pair(zero(), suc(zero())), &pair(var("x"), var("x")), &m),
            Err(ConstructorMismatch("Suc".to_owned(), "Zero".to_owned()))
        );
    }

    #[test]
    fn value_conflicting_bindings_2() {
        let m = test_module();
        let tuple = |a, b, c| e::ValueExpression::Constructor {
            name: "Tuple".to_owned(),
            implicits: e::Rec::new([]),
            arguments: e::Rec::new([a, b, c]),
            result_type: nat_ty(),
        };

        assert_eq!(
            unify_value(
                &tuple(suc(suc(var("x"))), var("y"), suc(suc(zero()))),
                &tuple(suc(var("z")), suc(suc(suc(zero()))), suc(var("z"))),
                &m,
            ),
            Err(TypeMismatch)
        );
    }

    #[test]
    fn value_literal_mismatch() {
        let m = test_module();
        assert_eq!(
            unify_value(&lit_int(1), &lit_int(2), &m),
            Err(LiteralMismatch(Literal::Int(1), Literal::Int(2)))
        );
    }

    #[test]
    fn type_name_mismatch() {
        let module = test_module();
        assert_eq!(
            unify_type(&nat_ty(), &vec_ty(zero()), &module),
            Err(TypeMismatch)
        );
    }

    #[test]
    fn type_arity_mismatch() {
        let module = test_module();
        let bad = e::TypeExpression::TypeExpression {
            name: "Vec".to_owned(),
            dependencies: e::Rec::new([]),
        };
        assert_eq!(
            unify_type(&bad, &vec_ty(zero()), &module),
            Err(ArityMismatch {
                expected: 1,
                found: 0
            })
        );
    }

    #[test]
    fn ctor_mismatch() {
        let module = test_module();
        assert_eq!(
            unify_type(&vec_ty(zero()), &vec_ty(suc(zero())), &module),
            Err(ConstructorMismatch("Zero".to_owned(), "Suc".to_owned()))
        );
    }

    #[test]
    fn unify_type_no_deps_success() {
        let module = test_module();
        assert_eq!(unify_type(&nat_ty(), &nat_ty(), &module), Ok(vec![]));
    }

    #[test]
    fn unify_type_with_deps_success() {
        let module = test_module();
        assert_eq!(
            unify_type(&vec_ty(zero()), &vec_ty(var("n")), &module),
            Ok(vec![("n".to_owned(), zero())])
        );
    }

    #[test]
    fn ctor_unify_success() {
        let m = test_module();
        assert_eq!(
            unify_value(&suc(zero()), &suc(var("x")), &m),
            Ok(vec![("x".to_owned(), zero())])
        );
    }

    #[test]
    fn ctor_vs_opcall_mismatch() {
        let m = test_module();
        assert_eq!(unify_value(&zero(), &lit_int(1), &m), Err(TypeMismatch));
    }

    #[test]
    fn opcall_vs_ctor_mismatch() {
        let m = test_module();
        assert_eq!(unify_value(&lit_int(1), &zero(), &m), Err(TypeMismatch));
    }

    #[test]
    fn same_literal_success() {
        let m = test_module();
        assert_eq!(unify_value(&lit_int(42), &lit_int(42), &m), Ok(vec![]));
    }

    #[test]
    fn unary_same_op_success() {
        let m = test_module();
        assert_eq!(
            unify_value(&neg(var("x")), &neg(var("y")), &m),
            Ok(vec![("y".to_owned(), var("x"))])
        );
    }

    #[test]
    fn unary_op_mismatch() {
        let m = test_module();
        assert_eq!(
            unify_value(&neg(zero()), &bang(zero()), &m),
            Err(OperatorTypeMismatch)
        );
    }

    #[test]
    fn binary_op_mismatch() {
        let m = test_module();
        assert_eq!(
            unify_value(&add_op(zero(), zero()), &sub_op(zero(), zero()), &m),
            Err(OperatorTypeMismatch)
        );
    }

    #[test]
    fn opcall_kind_mismatch_literal_vs_unary() {
        let m = test_module();
        assert_eq!(
            unify_value(&lit_int(1), &neg(zero()), &m),
            Err(OperatorTypeMismatch)
        );
    }

    #[test]
    fn opcall_kind_mismatch_unary_vs_binary() {
        let m = test_module();
        assert_eq!(
            unify_value(&neg(zero()), &add_op(zero(), zero()), &m),
            Err(OperatorTypeMismatch)
        );
    }

    #[test]
    fn binary_op_success() {
        let m = test_module();
        assert_eq!(
            unify_value(&add_op(zero(), var("x")), &add_op(zero(), var("y")), &m),
            Ok(vec![("y".to_owned(), var("x"))])
        );
    }
}
