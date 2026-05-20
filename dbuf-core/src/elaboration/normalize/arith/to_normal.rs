use super::types::{ArithCoeff, Mono, NormalForm, Poly};
use crate::ast::elaborated::ValueExpression;
use crate::ast::operators::{BinaryOp, OpCall, UnaryOp};
use crate::elaboration::normalize::find_var;

pub fn normalize<Str, C>(expr: &ValueExpression<Str>) -> NormalForm<Str, C>
where
    Str: Clone + PartialEq,
    C: ArithCoeff,
{
    let mut vars = Vec::new();
    let poly = expr_to_poly(expr, &mut vars);
    NormalForm { vars, poly }
}

fn expr_to_poly<Str, C>(
    expr: &ValueExpression<Str>,
    vars: &mut Vec<ValueExpression<Str>>,
) -> Poly<C>
where
    Str: Clone + PartialEq,
    C: ArithCoeff,
{
    match expr {
        ValueExpression::Variable { .. } => {
            let idx = find_var(vars, expr.clone());
            var_poly(idx)
        }

        ValueExpression::OpCall { op_call, .. } => match op_call {
            OpCall::Literal(lit) => {
                if let Some(c) = C::try_from_literal(lit) {
                    const_poly(c)
                } else {
                    let idx = find_var(vars, expr.clone());
                    var_poly(idx)
                }
            }

            OpCall::Unary(UnaryOp::Minus, arg) => poly_neg(expr_to_poly(arg, vars)),

            OpCall::Binary(BinaryOp::Plus, lhs, rhs) => {
                poly_add(expr_to_poly(lhs, vars), expr_to_poly(rhs, vars))
            }
            OpCall::Binary(BinaryOp::Minus, lhs, rhs) => {
                poly_sub(expr_to_poly(lhs, vars), expr_to_poly(rhs, vars))
            }
            OpCall::Binary(BinaryOp::Star, lhs, rhs) => {
                poly_mul(&expr_to_poly(lhs, vars), &expr_to_poly(rhs, vars))
            }
            _ => {
                let idx = find_var(vars, expr.clone());
                var_poly(idx)
            }
        },
        ValueExpression::Constructor { .. } => {
            let idx = find_var(vars, expr.clone());
            var_poly(idx)
        }
    }
}

fn const_poly<C: ArithCoeff>(n: C) -> Poly<C> {
    if n.is_zero() {
        return Poly::new();
    }
    [(Mono::new(), n)].into_iter().collect()
}

fn var_poly<C: ArithCoeff>(idx: usize) -> Poly<C> {
    let mono: Mono = [(idx, 1)].into_iter().collect();
    [(mono, C::one())].into_iter().collect()
}

fn poly_neg<C: ArithCoeff>(p: Poly<C>) -> Poly<C> {
    p.into_iter().map(|(m, c)| (m, c.coeff_neg())).collect()
}

fn poly_add<C: ArithCoeff>(mut lhs: Poly<C>, rhs: Poly<C>) -> Poly<C> {
    for (mono, coeff) in rhs {
        let entry = lhs.entry(mono).or_insert(C::zero());
        *entry = *entry + coeff;
    }
    lhs.retain(|_, c| !c.is_zero());
    lhs
}

fn poly_sub<C: ArithCoeff>(mut lhs: Poly<C>, rhs: Poly<C>) -> Poly<C> {
    for (mono, coeff) in rhs {
        let entry = lhs.entry(mono).or_insert(C::zero());
        *entry = *entry - coeff;
    }
    lhs.retain(|_, c| !c.is_zero());
    lhs
}

fn mono_mul(lhs: &Mono, rhs: &Mono) -> Mono {
    let mut result = lhs.clone();
    for (idx, exp) in rhs {
        *result.entry(*idx).or_insert(0) += exp;
    }
    result
}

fn poly_mul<C: ArithCoeff>(lhs: &Poly<C>, rhs: &Poly<C>) -> Poly<C> {
    let mut result = Poly::new();
    for (lm, lc) in lhs {
        for (rm, rc) in rhs {
            let coeff = *lc * *rc;
            if !coeff.is_zero() {
                let entry = result.entry(mono_mul(lm, rm)).or_insert(C::zero());
                *entry = *entry + coeff;
            }
        }
    }
    result.retain(|_, c| !c.is_zero());
    result
}
