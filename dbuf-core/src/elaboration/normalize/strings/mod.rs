use crate::ast::elaborated::{TypeExpression, ValueExpression};
use crate::ast::operators::{BinaryOp, Literal, OpCall};
use crate::elaboration::operators::{make_binary, make_lit};

#[cfg(test)]
mod tests;

pub fn normalize<Str: Clone + PartialEq>(expr: &ValueExpression<Str>) -> Vec<ValueExpression<Str>> {
    match expr {
        ValueExpression::OpCall { op_call, .. } => match op_call {
            OpCall::Literal(Literal::Str(_)) => vec![expr.clone()],
            OpCall::Binary(BinaryOp::Plus, lhs, rhs) => {
                let mut strings = normalize(lhs);
                strings.extend(normalize(rhs));
                let mut result = vec![];
                for seg in strings {
                    if let ValueExpression::OpCall {
                        op_call: OpCall::Literal(Literal::Str(new_s)),
                        ..
                    } = &seg
                    {
                        if let Some(ValueExpression::OpCall {
                            op_call: OpCall::Literal(Literal::Str(prev_s)),
                            ..
                        }) = result.last_mut()
                        {
                            prev_s.push_str(new_s);
                        } else {
                            result.push(seg);
                        }
                    } else {
                        result.push(seg);
                    }
                }
                result
            }
            _ => vec![expr.clone()],
        },
        _ => vec![expr.clone()],
    }
}

pub fn strings_to_expr<Str: Clone + PartialEq>(
    strings: &[ValueExpression<Str>],
    result_type: TypeExpression<Str>,
) -> ValueExpression<Str> {
    let mut iter = strings.iter().cloned();
    let Some(first) = iter.next() else {
        return make_lit(Literal::Str(String::new()), result_type);
    };
    iter.fold(first, |acc, e| {
        make_binary(BinaryOp::Plus, acc, e, result_type.clone())
    })
}
