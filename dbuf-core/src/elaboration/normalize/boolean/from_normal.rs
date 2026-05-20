use crate::ast::elaborated::{TypeExpression, ValueExpression};
use crate::ast::operators::{BinaryOp, Literal, UnaryOp};
use crate::elaboration::operators::{make_binary, make_lit, make_unary};

use super::types::{Mono, NormalForm};

pub fn poly_to_expr<Str: Clone + PartialEq>(
    nf: &NormalForm<Str>,
    result_type: TypeExpression<Str>,
) -> ValueExpression<Str> {
    let mut poly = nf
        .poly
        .iter()
        .map(|mono| mono_to_expr(mono, &nf.vars, &result_type));
    let Some(first) = poly.next() else {
        return make_lit(Literal::Bool(false), result_type);
    };
    poly.fold(first, |acc, e| {
        make_binary(BinaryOp::BinaryOr, acc, e, result_type.clone())
    })
}

fn mono_to_expr<Str: Clone + PartialEq>(
    mono: &Mono,
    vars: &[ValueExpression<Str>],
    result_type: &TypeExpression<Str>,
) -> ValueExpression<Str> {
    let mut lits = mono.iter().map(|(&idx, &pol)| {
        let e = vars[idx].clone();
        if pol {
            e
        } else {
            make_unary(UnaryOp::Bang, e, result_type.clone())
        }
    });
    let Some(first) = lits.next() else {
        return make_lit(Literal::Bool(true), result_type.clone());
    };
    lits.fold(first, |acc, e| {
        make_binary(BinaryOp::BinaryAnd, acc, e, result_type.clone())
    })
}
