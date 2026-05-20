use super::types::{Mono, NormalForm, Poly};
use crate::ast::elaborated::ValueExpression;
use crate::ast::operators::{BinaryOp, Literal, OpCall, UnaryOp};
use crate::elaboration::normalize::find_var;
use std::collections::btree_map::Entry;

pub fn normalize<Str: Clone + PartialEq>(expr: &ValueExpression<Str>) -> NormalForm<Str> {
    let mut vars = Vec::new();
    let poly = expr_to_poly(expr, &mut vars);
    NormalForm { vars, poly }
}

fn expr_to_poly<Str: Clone + PartialEq>(
    expr: &ValueExpression<Str>,
    vars: &mut Vec<ValueExpression<Str>>,
) -> Poly {
    if let ValueExpression::OpCall { op_call, .. } = expr {
        match op_call {
            OpCall::Literal(Literal::Bool(b)) => {
                if *b {
                    poly_true()
                } else {
                    poly_false()
                }
            }
            OpCall::Unary(UnaryOp::Bang, arg) => poly_not(expr_to_poly(arg, vars)),
            OpCall::Binary(BinaryOp::BinaryAnd, lhs, rhs) => {
                poly_and(&expr_to_poly(lhs, vars), &expr_to_poly(rhs, vars))
            }
            OpCall::Binary(BinaryOp::BinaryOr, lhs, rhs) => {
                poly_or(expr_to_poly(lhs, vars), expr_to_poly(rhs, vars))
            }
            _ => {
                let idx = find_var(vars, expr.clone());
                poly_var(idx)
            }
        }
    } else {
        let idx = find_var(vars, expr.clone());
        poly_var(idx)
    }
}

fn poly_false() -> Poly {
    Poly::new()
}

fn poly_true() -> Poly {
    [Mono::new()].into()
}

fn poly_var(idx: usize) -> Poly {
    Poly::from([Mono::from([(idx, true)])])
}

fn poly_or(mut lhs: Poly, rhs: Poly) -> Poly {
    lhs.extend(rhs);

    if lhs.contains(&Mono::new()) {
        return poly_true();
    }
    lhs
}

fn mono_and(lhs: &Mono, rhs: &Mono) -> Option<Mono> {
    let mut result = lhs.clone();
    for (&idx, &pol) in rhs {
        match result.entry(idx) {
            Entry::Occupied(e) => {
                if *e.get() != pol {
                    return None;
                }
            }
            Entry::Vacant(e) => {
                e.insert(pol);
            }
        }
    }
    Some(result)
}

fn poly_and(lhs: &Poly, rhs: &Poly) -> Poly {
    let mut result = Poly::new();
    for lc in lhs {
        for rc in rhs {
            if let Some(merged) = mono_and(lc, rc) {
                result.insert(merged);
            }
        }
    }
    if result.contains(&Mono::new()) {
        return poly_true();
    }
    result
}

fn poly_not(poly: Poly) -> Poly {
    if poly.is_empty() {
        return poly_true();
    }

    let mut result = poly_true();
    for mono in poly {
        if mono.is_empty() {
            return poly_false();
        }
        let neg_mono: Poly = mono
            .into_iter()
            .map(|(idx, pol)| Mono::from([(idx, !pol)]))
            .collect();

        result = poly_and(&result, &neg_mono);
    }
    result
}
