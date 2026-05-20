use std::sync::Arc;

use super::normalize;
use crate::ast::elaborated::{TypeExpression, ValueExpression};
use crate::ast::operators::{BinaryOp, Literal, OpCall};
use crate::elaboration::builtins::{BuiltinType, get_builtin};

type Str = String;

fn str_ty() -> TypeExpression<Str> {
    get_builtin(&BuiltinType::String)
}

fn lit(s: &str) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Literal(Literal::Str(s.to_owned())),
        result_type: str_ty(),
    }
}

fn var(name: &str) -> ValueExpression<Str> {
    ValueExpression::Variable {
        name: name.to_string(),
        ty: str_ty(),
    }
}

fn cat(lhs: ValueExpression<Str>, rhs: ValueExpression<Str>) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Binary(BinaryOp::Plus, Arc::new(lhs), Arc::new(rhs)),
        result_type: str_ty(),
    }
}

fn lit_seg(strings: &[ValueExpression<Str>], i: usize) -> &str {
    if let ValueExpression::OpCall {
        op_call: OpCall::Literal(Literal::Str(s)),
        ..
    } = &strings[i]
    {
        s
    } else {
        panic!("expected string literal at {i}")
    }
}

fn is_opaque(e: &ValueExpression<Str>) -> bool {
    !matches!(
        e,
        ValueExpression::OpCall {
            op_call: OpCall::Literal(Literal::Str(_)),
            ..
        }
    )
}

#[test]
fn literals_before_and_after_var() {
    let strings = normalize(&cat(cat(cat(lit("a"), var("x")), lit("b")), lit("c")));
    assert_eq!(strings.len(), 3);
    assert_eq!(lit_seg(&strings, 0), "a");
    assert!(is_opaque(&strings[1]));
    assert_eq!(lit_seg(&strings, 2), "bc");
}

#[test]
fn literals_not_merged() {
    let strings = normalize(&cat(cat(lit("a"), var("x")), lit("b")));
    assert_eq!(strings.len(), 3);
}
