use std::sync::Arc;

use super::{Mono, NormalForm, normalize};
use crate::ast::elaborated::{TypeExpression, ValueExpression};
use crate::ast::operators::{BinaryOp, Literal, OpCall, UnaryOp};
use crate::elaboration::builtins::{BuiltinType, get_builtin};

type Str = String;

fn int_ty() -> TypeExpression<Str> {
    get_builtin(&BuiltinType::Int)
}

fn lit(n: i64) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Literal(Literal::Int(n)),
        result_type: int_ty(),
    }
}

fn var(name: &str) -> ValueExpression<Str> {
    ValueExpression::Variable {
        name: name.to_string(),
        ty: int_ty(),
    }
}

fn add(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Binary(BinaryOp::Plus, Arc::new(lhs), Arc::new(rhs)),
        result_type: int_ty(),
    }
}

fn sub(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Binary(BinaryOp::Minus, Arc::new(lhs), Arc::new(rhs)),
        result_type: int_ty(),
    }
}

fn mul(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Binary(BinaryOp::Star, Arc::new(lhs), Arc::new(rhs)),
        result_type: int_ty(),
    }
}

fn neg(arg: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Unary(UnaryOp::Minus, Arc::new(arg)),
        result_type: int_ty(),
    }
}

fn sq(e: ValueExpression<Str>) -> ValueExpression<Str> {
    mul(e.clone(), e)
}

fn cb(e: ValueExpression<Str>) -> ValueExpression<Str> {
    mul(sq(e.clone()), e)
}

fn mono(pairs: &[(usize, u32)]) -> Mono {
    pairs.iter().copied().collect()
}

fn poly_coeff(nf: &NormalForm<Str, i64>, m: &Mono) -> i64 {
    *nf.poly.get(m).unwrap_or(&0)
}

#[track_caller]
fn assert_poly_eq(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) {
    let diff = normalize::<Str, i64>(&sub(lhs, rhs));
    assert!(
        diff.poly.is_empty(),
        "polynomials are not equal; expected: 0, but got: {:?}",
        diff.poly,
    );
}

#[test]
fn literal_constant() {
    let nf = normalize::<Str, i64>(&lit(42));
    assert!(nf.vars.is_empty());
    assert_eq!(poly_coeff(&nf, &mono(&[])), 42);
    assert_eq!(nf.poly.len(), 1);
}

#[test]
fn single_variable() {
    let nf = normalize::<Str, i64>(&var("x"));
    assert_eq!(nf.vars, vec![var("x")]);
    assert_eq!(poly_coeff(&nf, &mono(&[(0, 1)])), 1);
}

#[test]
fn add_same_variable() {
    let nf = normalize::<Str, i64>(&add(var("x"), var("x")));
    assert_eq!(nf.vars.len(), 1);
    assert_eq!(poly_coeff(&nf, &mono(&[(0, 1)])), 2);
    assert_eq!(nf.poly.len(), 1);
}

#[test]
fn sub_cancels() {
    let nf = normalize::<Str, i64>(&sub(var("x"), var("x")));
    assert!(nf.poly.is_empty());
}

#[test]
fn mul_commutativity() {
    assert_poly_eq(mul(var("x"), var("y")), mul(var("y"), var("x")));
}

#[test]
fn add_associativity() {
    assert_poly_eq(
        add(add(var("a"), var("b")), var("c")),
        add(var("a"), add(var("b"), var("c"))),
    );
}

#[test]
fn double_negation() {
    assert_poly_eq(neg(neg(var("x"))), var("x"));
}

#[test]
fn distributivity() {
    assert_poly_eq(
        mul(add(var("a"), var("b")), add(var("c"), var("d"))),
        add(
            add(mul(var("a"), var("c")), mul(var("a"), var("d"))),
            add(mul(var("b"), var("c")), mul(var("b"), var("d"))),
        ),
    );
}

#[test]
fn cube_of_sum() {
    assert_poly_eq(
        cb(add(var("x"), var("y"))),
        add(
            add(cb(var("x")), mul(lit(3), mul(sq(var("x")), var("y")))),
            add(mul(lit(3), mul(var("x"), sq(var("y")))), cb(var("y"))),
        ),
    );
}

#[test]
fn product() {
    assert_poly_eq(
        mul(
            mul(add(var("x"), lit(1)), add(var("x"), lit(2))),
            add(var("x"), lit(3)),
        ),
        add(
            add(cb(var("x")), mul(lit(6), sq(var("x")))),
            add(mul(lit(11), var("x")), lit(6)),
        ),
    );
}

#[test]
fn complex_product() {
    let x2 = sq(var("x"));
    let y2 = sq(var("y"));
    let z2 = sq(var("z"));
    assert_poly_eq(
        mul(
            add(var("x"), var("y")),
            mul(
                sub(var("y"), var("z")),
                mul(add(var("x"), var("z")), lit(2)),
            ),
        ),
        mul(
            lit(2),
            add(
                mul(x2.clone(), var("y")),
                add(
                    neg(mul(x2, var("z"))),
                    add(
                        mul(var("x"), y2.clone()),
                        add(
                            neg(mul(var("x"), z2.clone())),
                            add(mul(y2, var("z")), neg(mul(var("y"), z2))),
                        ),
                    ),
                ),
            ),
        ),
    );
}
