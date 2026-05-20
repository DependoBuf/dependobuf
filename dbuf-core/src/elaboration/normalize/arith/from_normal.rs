use crate::ast::elaborated::{TypeExpression, ValueExpression};
use crate::ast::operators::{BinaryOp, UnaryOp};
use crate::elaboration::operators;

use super::types::{ArithCoeff, Mono, NormalForm};

pub fn poly_to_expr<Str, C>(
    nf: &NormalForm<Str, C>,
    result_type: TypeExpression<Str>,
) -> ValueExpression<Str>
where
    Str: Clone + PartialEq,
    C: ArithCoeff,
{
    if nf.poly.is_empty() {
        return operators::make_lit(C::zero().to_literal(), result_type);
    }

    let mut pos: Vec<ValueExpression<Str>> = Vec::new();
    let mut neg: Vec<ValueExpression<Str>> = Vec::new();

    for (mono, &coeff) in &nf.poly {
        if coeff.is_neg() {
            neg.push(mono_to_expr(
                mono,
                coeff.coeff_abs(),
                &nf.vars,
                &result_type,
            ));
        } else {
            pos.push(mono_to_expr(mono, coeff, &nf.vars, &result_type));
        }
    }

    let pos_sum = pos
        .into_iter()
        .reduce(|acc, t| operators::make_binary(BinaryOp::Plus, acc, t, result_type.clone()));
    let neg_sum = neg
        .into_iter()
        .reduce(|acc, t| operators::make_binary(BinaryOp::Plus, acc, t, result_type.clone()));

    match (pos_sum, neg_sum) {
        (None, None) => operators::make_lit(C::zero().to_literal(), result_type),
        (Some(p), None) => p,
        (None, Some(n)) => operators::make_unary(UnaryOp::Minus, n, result_type),
        (Some(p), Some(n)) => operators::make_binary(BinaryOp::Minus, p, n, result_type),
    }
}

fn mono_to_expr<Str, C>(
    mono: &Mono,
    coeff: C,
    vars: &[ValueExpression<Str>],
    result_type: &TypeExpression<Str>,
) -> ValueExpression<Str>
where
    Str: Clone + PartialEq,
    C: ArithCoeff,
{
    let mut product: Option<ValueExpression<Str>> = None;
    for (&idx, &exp) in mono {
        for _ in 0..exp {
            let factor = vars[idx].clone();
            product = Some(match product {
                None => factor,
                Some(acc) => {
                    operators::make_binary(BinaryOp::Star, acc, factor, result_type.clone())
                }
            });
        }
    }

    match (coeff == C::one(), product) {
        (_, None) => operators::make_lit(coeff.to_literal(), result_type.clone()),
        (true, Some(p)) => p,
        (false, Some(p)) => operators::make_binary(
            BinaryOp::Star,
            operators::make_lit(coeff.to_literal(), result_type.clone()),
            p,
            result_type.clone(),
        ),
    }
}
