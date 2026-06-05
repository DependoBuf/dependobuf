use std::sync::Arc;

use super::{Poly, normalize};
use crate::ast::elaborated::{TypeExpression, ValueExpression};
use crate::ast::operators::{BinaryOp, Literal, OpCall, UnaryOp};
use crate::elaboration::builtins::{BuiltinType, get_builtin};

type Str = String;

fn bool_ty() -> TypeExpression<Str> {
    get_builtin(&BuiltinType::Bool)
}

fn lit(b: bool) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Literal(Literal::Bool(b)),
        result_type: bool_ty(),
    }
}

fn var(name: &str) -> ValueExpression<Str> {
    ValueExpression::Variable {
        name: name.to_string(),
        ty: bool_ty(),
    }
}

fn and(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Binary(BinaryOp::BinaryAnd, Arc::new(lhs), Arc::new(rhs)),
        result_type: bool_ty(),
    }
}

fn or(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Binary(BinaryOp::BinaryOr, Arc::new(lhs), Arc::new(rhs)),
        result_type: bool_ty(),
    }
}

fn not(arg: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Unary(UnaryOp::Bang, Arc::new(arg)),
        result_type: bool_ty(),
    }
}

#[test]
fn literal() {
    let nf_true = normalize(&lit(true));
    let nf_false = normalize(&lit(false));
    assert!(
        &nf_true
            .poly
            .iter()
            .any(std::collections::BTreeMap::is_empty)
    );
    assert!(&nf_false.poly.is_empty());
}

#[test]
fn and_false() {
    let nf = normalize(&and(var("x"), lit(false)));
    assert!(&nf.poly.is_empty());
}

#[test]
fn and_true() {
    let nf = normalize(&and(var("x"), lit(true)));
    assert_eq!(nf.vars.len(), 1);
    assert_eq!(nf.poly.len(), 1);
}

#[test]
fn or_true() {
    let nf = normalize(&or(var("x"), lit(true)));
    assert!(&nf.poly.iter().any(std::collections::BTreeMap::is_empty));
}

#[test]
fn or_false() {
    let nf = normalize(&or(var("x"), lit(false)));
    assert_eq!(nf.vars.len(), 1);
    assert_eq!(nf.poly.len(), 1);
}

#[test]
fn contradiction() {
    let nf = normalize(&and(var("x"), not(var("x"))));
    assert!(&nf.poly.is_empty());
}
#[test]
fn double_negation() {
    let nf_orig = normalize(&var("x"));
    let nf_new = normalize(&not(not(var("x"))));
    assert_eq!(nf_orig.poly, nf_new.poly);
}

#[test]
fn demorgan_and() {
    assert_bool_eq(
        not(and(var("x"), var("y"))),
        or(not(var("x")), not(var("y"))),
    );
}

#[test]
fn demorgan_or() {
    assert_bool_eq(
        not(or(var("x"), var("y"))),
        and(not(var("x")), not(var("y"))),
    );
}

#[track_caller]
fn assert_bool_eq(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) {
    let nf_lhs = normalize(&lhs);
    let nf_rhs = normalize(&rhs);
    assert_eq!(
        nf_lhs.poly, nf_rhs.poly,
        "boolean expressions not equal: lhs={:?}, rhs={:?}",
        nf_lhs.poly, nf_rhs.poly,
    );

    let nf_diff = normalize(&and(lhs, not(rhs)));
    assert!(
        nf_diff.poly.is_empty(),
        "a & !b should be false but got {:?}",
        nf_diff.poly,
    );
}

fn poly_true() -> Poly {
    [std::collections::BTreeMap::new()].into()
}

#[test]
fn and_commutativity() {
    assert_bool_eq(and(var("x"), var("y")), and(var("y"), var("x")));
}

#[test]
fn or_commutativity() {
    assert_bool_eq(or(var("x"), var("y")), or(var("y"), var("x")));
}

#[test]
fn and_associativity() {
    assert_bool_eq(
        and(and(var("x"), var("y")), var("z")),
        and(var("x"), and(var("y"), var("z"))),
    );
}

#[test]
fn or_associativity() {
    assert_bool_eq(
        or(or(var("x"), var("y")), var("z")),
        or(var("x"), or(var("y"), var("z"))),
    );
}

#[test]
fn idempotence_and() {
    assert_bool_eq(and(var("x"), var("x")), var("x"));
}

#[test]
fn idempotence_or() {
    assert_bool_eq(or(var("x"), var("x")), var("x"));
}

#[test]
fn true_and_x() {
    assert_bool_eq(and(lit(true), var("x")), var("x"));
}

#[test]
fn false_or_x() {
    assert_bool_eq(or(lit(false), var("x")), var("x"));
}

#[test]
fn tautology() {
    let nf = normalize(&or(var("x"), not(var("x"))));
    assert_eq!(nf.poly, poly_true());
}

#[test]
fn tautology_case_split() {
    assert_bool_eq(
        or(or(and(var("a"), var("b")), not(var("a"))), not(var("b"))),
        lit(true),
    );
}

#[test]
fn tautology_excluded_middle_complex() {
    assert_bool_eq(
        or(
            or(and(var("x"), var("y")), and(var("x"), not(var("y")))),
            not(var("x")),
        ),
        lit(true),
    );
}

#[test]
fn triple_demorgan_and() {
    assert_bool_eq(
        not(and(and(var("x"), var("y")), var("z"))),
        or(or(not(var("x")), not(var("y"))), not(var("z"))),
    );
}

#[test]
fn triple_demorgan_or() {
    assert_bool_eq(
        not(or(or(var("x"), var("y")), var("z"))),
        and(and(not(var("x")), not(var("y"))), not(var("z"))),
    );
}

#[test]
fn distributivity_full_equality() {
    assert_bool_eq(
        and(var("x"), or(var("y"), var("z"))),
        or(and(var("x"), var("y")), and(var("x"), var("z"))),
    );
}
