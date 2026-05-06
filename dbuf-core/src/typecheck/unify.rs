use crate::ast::elaborated as e;
use crate::ast::operators as o;
use crate::typecheck::subst;

pub type Bindings<Str> = Vec<(Str, e::ValueExpression<Str>)>;

pub type UnifyResult<Str> = (Bindings<Str>, Bindings<Str>);

#[derive(Debug, PartialEq)]
pub enum UnifyError {
    TypeNameMismatch,
    ArityMismatch,
    ConstructorNameMismatch,
    OperatorMismatch,
    LiteralMismatch,
    ConflictingBinding,
    CannotUnify,
}

/// # Errors
pub fn unify_type<Str>(
    a: &e::TypeExpression<Str>,
    b: &e::TypeExpression<Str>,
    module: &e::Module<Str>,
) -> Result<UnifyResult<Str>, UnifyError>
where
    Str: Clone + Eq + Ord,
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
        return Err(UnifyError::TypeNameMismatch);
    }

    let declared_arity = module
        .types
        .iter()
        .find(|(name, _)| name == name_a)
        .map_or(0, |(_, ty)| ty.dependencies.len());

    if deps_a.len() != declared_arity || deps_b.len() != declared_arity {
        return Err(UnifyError::ArityMismatch);
    }

    unify_args(
        deps_a.iter().cloned().collect(),
        deps_b.iter().cloned().collect(),
    )
}

/// # Errors
pub fn unify_value<Str>(
    a: &e::ValueExpression<Str>,
    b: &e::ValueExpression<Str>,
) -> Result<UnifyResult<Str>, UnifyError>
where
    Str: Clone + Eq,
{
    match (a, b) {
        (
            e::ValueExpression::Variable { name: x, .. },
            e::ValueExpression::Variable { name: y, .. },
        ) if x == y => Ok((vec![], vec![])),
        //TODO: check types are equal
        (
            e::ValueExpression::Variable { name: x, ty: ty_x },
            e::ValueExpression::Variable { name: y, ty: _ty_y },
        ) => Ok((
            vec![],
            vec![(
                y.clone(),
                e::ValueExpression::Variable {
                    name: x.clone(),
                    ty: ty_x.clone(),
                },
            )],
        )),
        (e::ValueExpression::Variable { .. }, _) => Err(UnifyError::CannotUnify),

        (other, e::ValueExpression::Variable { name, .. }) => {
            Ok((vec![], vec![(name.clone(), other.clone())]))
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
                return Err(UnifyError::ConstructorNameMismatch);
            }
            let left: Vec<_> = i1.iter().chain(a1.iter()).cloned().collect();
            let right: Vec<_> = i2.iter().chain(a2.iter()).cloned().collect();
            unify_args(left, right)
        }

        (
            e::ValueExpression::OpCall { op_call: op_a, .. },
            e::ValueExpression::OpCall { op_call: op_b, .. },
        ) => unify_op_call(op_a, op_b),

        _ => Err(UnifyError::CannotUnify),
    }
}

fn unify_args<Str>(
    mut left: Vec<e::ValueExpression<Str>>,
    mut right: Vec<e::ValueExpression<Str>>,
) -> Result<UnifyResult<Str>, UnifyError>
where
    Str: Clone + Eq,
{
    let mut acc_left: Bindings<Str> = vec![];
    let mut acc_right: Bindings<Str> = vec![];

    for i in 0..left.len() {
        let (bl, br) = unify_value(&left[i], &right[i])?;

        for arg in &mut left[i + 1..] {
            *arg = subst::apply_bindings(arg.clone(), &bl);
        }
        for arg in &mut right[i + 1..] {
            *arg = subst::apply_bindings(arg.clone(), &br);
        }

        extend_bindings(&mut acc_left, bl)?;
        extend_bindings(&mut acc_right, br)?;
    }

    Ok((acc_left, acc_right))
}

fn unify_op_call<Str>(
    a: &o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>,
    b: &o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>,
) -> Result<UnifyResult<Str>, UnifyError>
where
    Str: Clone + Eq,
{
    match (a, b) {
        (o::OpCall::Literal(la), o::OpCall::Literal(lb)) => {
            if la == lb {
                Ok((vec![], vec![]))
            } else {
                Err(UnifyError::LiteralMismatch)
            }
        }
        (o::OpCall::Unary(op_a, expr_a), o::OpCall::Unary(op_b, expr_b)) => {
            if !unary_ops_match(op_a, op_b) {
                return Err(UnifyError::OperatorMismatch);
            }
            unify_value(expr_a, expr_b)
        }
        (o::OpCall::Binary(op_a, la, ra), o::OpCall::Binary(op_b, lb, rb)) => {
            if op_a != op_b {
                return Err(UnifyError::OperatorMismatch);
            }
            unify_args(
                vec![(**la).clone(), (**ra).clone()],
                vec![(**lb).clone(), (**rb).clone()],
            )
        }
        _ => Err(UnifyError::OperatorMismatch),
    }
}

fn unary_ops_match<Str: Eq>(a: &o::UnaryOp<Str>, b: &o::UnaryOp<Str>) -> bool {
    match (a, b) {
        (o::UnaryOp::Access(fa), o::UnaryOp::Access(fb)) => fa == fb,
        (o::UnaryOp::Minus, o::UnaryOp::Minus) | (o::UnaryOp::Bang, o::UnaryOp::Bang) => true,
        _ => false,
    }
}

fn extend_bindings<Str>(acc: &mut Bindings<Str>, new: Bindings<Str>) -> Result<(), UnifyError>
where
    Str: Clone + Eq,
{
    for (name, val) in new {
        match acc.iter().find(|(n, _)| n == &name) {
            Some((_, existing)) if existing == &val => {}
            Some(_) => return Err(UnifyError::ConflictingBinding),
            None => acc.push((name, val)),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::elaborated::{self as e, ConstructorNames};
    use crate::typecheck::unify::UnifyError::CannotUnify;
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
            op_call: o::OpCall::Literal(crate::ast::operators::Literal::Int(v)),
            result_type: nat_ty(),
        }
    }

    fn test_module() -> e::Module<String> {
        e::Module {
            types: vec![
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
            ],
            constructors: BTreeMap::new(),
        }
    }

    #[test]
    fn value_same_variable_is_empty() {
        assert_eq!(unify_value(&var("x"), &var("x")), Ok((vec![], vec![])));
    }

    #[test]
    fn value_variable_left_is_error() {
        assert_eq!(unify_value(&var("x"), &zero()), Err(CannotUnify));
    }

    #[test]
    fn value_variable_right_produces_right_binding() {
        assert_eq!(
            unify_value(&zero(), &var("y")),
            Ok((vec![], vec![("y".to_owned(), zero())]))
        );
    }

    #[test]
    fn value_two_different_variables_binds_right_to_left() {
        assert_eq!(
            unify_value(&var("x"), &var("y")),
            Ok((vec![], vec![("y".to_owned(), var("x"))]))
        );
    }

    #[test]
    fn value_constructors_equal_no_args() {
        assert_eq!(unify_value(&zero(), &zero()), Ok((vec![], vec![])));
    }

    #[test]
    fn value_constructors_variable_arg_left() {
        assert_eq!(unify_value(&suc(var("x")), &suc(zero())), Err(CannotUnify));
    }

    #[test]
    fn value_constructors_variable_arg_right() {
        assert_eq!(
            unify_value(&suc(zero()), &suc(var("y"))),
            Ok((vec![], vec![("y".to_owned(), zero())]))
        );
    }

    #[test]
    fn value_constructors_equal_both_same_variable() {
        assert_eq!(
            unify_value(&suc(var("x")), &suc(var("x"))),
            Ok((vec![], vec![]))
        );
    }

    #[test]
    fn value_constructor_name_mismatch() {
        assert_eq!(
            unify_value(&zero(), &suc(zero())),
            Err(UnifyError::ConstructorNameMismatch)
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
    fn value_incremental_substitution_applied_to_remaining_args() {
        assert_eq!(
            unify_value(&pair(zero(), zero()), &pair(var("x"), var("x"))),
            Ok((vec![], vec![("x".to_owned(), zero())]))
        );
    }

    #[test]
    fn value_conflicting_bindings() {
        assert_eq!(
            unify_value(&pair(zero(), suc(zero())), &pair(var("x"), var("x"))),
            Err(UnifyError::ConstructorNameMismatch)
        );
    }

    #[test]
    fn value_conflicting_bindings_2() {
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
            ),
            Err(CannotUnify)
        );
    }

    #[test]
    fn value_literal_equal() {
        assert_eq!(
            unify_value(&lit_int(42), &lit_int(42)),
            Ok((vec![], vec![]))
        );
    }

    #[test]
    fn value_literal_mismatch() {
        assert_eq!(
            unify_value(&lit_int(1), &lit_int(2)),
            Err(UnifyError::LiteralMismatch)
        );
    }

    #[test]
    fn value_mismatched_variants() {
        assert_eq!(unify_value(&zero(), &lit_int(0)), Err(CannotUnify));
    }

    #[test]
    fn type_equal_no_deps() {
        let module = test_module();
        assert_eq!(
            unify_type(&nat_ty(), &nat_ty(), &module),
            Ok((vec![], vec![]))
        );
    }

    #[test]
    fn type_name_mismatch() {
        let module = test_module();
        assert_eq!(
            unify_type(&nat_ty(), &vec_ty(zero()), &module),
            Err(UnifyError::TypeNameMismatch)
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
            Err(UnifyError::ArityMismatch)
        );
    }

    #[test]
    fn type_equal_with_concrete_dep() {
        let module = test_module();
        assert_eq!(
            unify_type(&vec_ty(zero()), &vec_ty(zero()), &module),
            Ok((vec![], vec![]))
        );
    }

    #[test]
    fn type_variable_dep_left() {
        let module = test_module();
        assert_eq!(
            unify_type(&vec_ty(var("n")), &vec_ty(zero()), &module),
            Err(CannotUnify)
        );
    }

    #[test]
    fn type_variable_dep_right() {
        let module = test_module();
        assert_eq!(
            unify_type(&vec_ty(zero()), &vec_ty(var("m")), &module),
            Ok((vec![], vec![("m".to_owned(), zero())]))
        );
    }

    #[test]
    fn type_dep_mismatch() {
        let module = test_module();
        assert_eq!(
            unify_type(&vec_ty(zero()), &vec_ty(suc(zero())), &module),
            Err(UnifyError::ConstructorNameMismatch)
        );
    }
}
