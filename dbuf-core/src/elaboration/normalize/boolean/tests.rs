use std::sync::Arc;

use super::normalize;
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
    let lhs = normalize(&not(and(var("x"), var("y"))));
    let rhs = normalize(&or(not(var("x")), not(var("y"))));
    assert_eq!(lhs.poly.len(), rhs.poly.len());
}

#[test]
fn demorgan_or() {
    let lhs = normalize(&not(or(var("x"), var("y"))));
    let rhs = normalize(&and(not(var("x")), not(var("y"))));
    assert_eq!(lhs.poly.len(), rhs.poly.len());
}

#[test]
fn distributivity() {
    let lhs = normalize(&and(var("x"), or(var("y"), var("z"))));
    let rhs = normalize(&or(and(var("x"), var("y")), and(var("x"), var("z"))));
    assert_eq!(lhs.poly.len(), rhs.poly.len());
}
