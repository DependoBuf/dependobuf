use crate::ast::elaborated as e;
use crate::ast::operators as o;
use crate::elaboration::builtins::BuiltinType;
use crate::elaboration::normalize;

pub fn apply_bindings<Str>(
    mut expr: e::ValueExpression<Str>,
    bindings: &[(Str, e::ValueExpression<Str>)],
) -> e::ValueExpression<Str>
where
    Str: Clone + Eq + From<BuiltinType>,
{
    for (var, replacement) in bindings {
        let (new_expr, needs) = subst_value(expr, var, replacement);
        expr = if needs {
            normalize::simplify(&new_expr)
        } else {
            new_expr
        }
    }
    expr
}

pub fn apply_bindings_to_type<Str>(
    mut ty: e::TypeExpression<Str>,
    bindings: &[(Str, e::ValueExpression<Str>)],
) -> e::TypeExpression<Str>
where
    Str: Clone + Eq + From<BuiltinType>,
{
    for (var, replacement) in bindings {
        ty = subst_type(ty, var, replacement);
    }
    ty
}

fn subst_value<Str>(
    expr: e::ValueExpression<Str>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> (e::ValueExpression<Str>, bool)
where
    Str: Clone + Eq + From<BuiltinType>,
{
    match expr {
        e::ValueExpression::Variable { name, .. } if &name == var => (replacement.clone(), true),

        e::ValueExpression::Variable { name, ty } => {
            let ty = subst_type(ty, var, replacement);
            (e::ValueExpression::Variable { name, ty }, false)
        }

        e::ValueExpression::OpCall {
            op_call,
            result_type,
        } => {
            let (op_call, op_needs) = subst_op_call(op_call, var, replacement);
            let result_type = subst_type(result_type, var, replacement);
            let value = e::ValueExpression::OpCall {
                op_call,
                result_type,
            };
            (normalize::simplify(&value), op_needs)
        }

        e::ValueExpression::Constructor {
            name,
            implicits,
            arguments,
            result_type,
        } => {
            let implicits = subst_value_exprs(&implicits, var, replacement);
            let arguments = subst_value_exprs(&arguments, var, replacement);
            let result_type = subst_type(result_type, var, replacement);
            (
                e::ValueExpression::Constructor {
                    name,
                    implicits,
                    arguments,
                    result_type,
                },
                false,
            )
        }
    }
}

pub fn subst_type<Str>(
    ty: e::TypeExpression<Str>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> e::TypeExpression<Str>
where
    Str: Clone + Eq + From<BuiltinType>,
{
    let e::TypeExpression::TypeExpression { name, dependencies } = ty;
    let dependencies = subst_value_exprs(&dependencies, var, replacement);
    e::TypeExpression::TypeExpression { name, dependencies }
}

fn subst_value_exprs<Str>(
    exprs: &e::ValueExprs<Str>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> e::ValueExprs<Str>
where
    Str: Clone + Eq + From<BuiltinType>,
{
    let substituted: Vec<_> = exprs
        .iter()
        .cloned()
        .map(|e| {
            let (new_expr, needs) = subst_value(e, var, replacement);
            if needs {
                normalize::simplify(&new_expr)
            } else {
                new_expr
            }
        })
        .collect();
    e::Rec::from(substituted.as_slice())
}

fn subst_op_call<Str>(
    op: o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> (o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>, bool)
where
    Str: Clone + Eq + From<BuiltinType>,
{
    match op {
        o::OpCall::Literal(lit) => (o::OpCall::Literal(lit), false),

        o::OpCall::Unary(unary_op, arg) => {
            let (new_arg, needs) = subst_value((*arg).clone(), var, replacement);
            (o::OpCall::Unary(unary_op, e::Rec::new(new_arg)), needs)
        }

        o::OpCall::Binary(bin_op, lhs, rhs) => {
            let (new_lhs, lhs_needs) = subst_value((*lhs).clone(), var, replacement);
            let (new_rhs, rhs_needs) = subst_value((*rhs).clone(), var, replacement);
            (
                o::OpCall::Binary(bin_op, e::Rec::new(new_lhs), e::Rec::new(new_rhs)),
                lhs_needs || rhs_needs,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::elaborated as e;
    use crate::ast::operators as o;
    use crate::elaboration::builtins::{BuiltinType, get_builtin};

    type Str = String;

    fn int_ty() -> e::TypeExpression<Str> {
        get_builtin(&BuiltinType::Int)
    }

    fn nat_ty() -> e::TypeExpression<Str> {
        e::TypeExpression::TypeExpression {
            name: "Nat".to_owned(),
            dependencies: e::Rec::new([]),
        }
    }

    fn var_int(name: &str) -> e::ValueExpression<Str> {
        e::ValueExpression::Variable {
            name: name.to_owned(),
            ty: int_ty(),
        }
    }

    fn lit(n: i64) -> e::ValueExpression<Str> {
        e::ValueExpression::OpCall {
            op_call: o::OpCall::Literal(o::Literal::Int(n)),
            result_type: int_ty(),
        }
    }

    fn add(a: e::ValueExpression<Str>, b: e::ValueExpression<Str>) -> e::ValueExpression<Str> {
        e::ValueExpression::OpCall {
            op_call: o::OpCall::Binary(o::BinaryOp::Plus, e::Rec::new(a), e::Rec::new(b)),
            result_type: int_ty(),
        }
    }

    fn neg(a: e::ValueExpression<Str>) -> e::ValueExpression<Str> {
        e::ValueExpression::OpCall {
            op_call: o::OpCall::Unary(o::UnaryOp::Minus, e::Rec::new(a)),
            result_type: int_ty(),
        }
    }

    fn zero() -> e::ValueExpression<Str> {
        e::ValueExpression::Constructor {
            name: "Zero".to_owned(),
            implicits: e::Rec::new([]),
            arguments: e::Rec::new([]),
            result_type: nat_ty(),
        }
    }

    fn suc(n: e::ValueExpression<Str>) -> e::ValueExpression<Str> {
        e::ValueExpression::Constructor {
            name: "Suc".to_owned(),
            implicits: e::Rec::new([]),
            arguments: e::Rec::new([n]),
            result_type: nat_ty(),
        }
    }

    #[test]
    fn subst_in_opcall_binary() {
        let expr = add(var_int("x"), lit(1));
        let result = apply_bindings(expr, &[("x".to_owned(), lit(5))]);
        assert_eq!(result, lit(6));
    }

    #[test]
    fn subst_in_opcall_unary() {
        let expr = neg(var_int("x"));
        let result = apply_bindings(expr, &[("x".to_owned(), lit(3))]);
        assert_eq!(result, neg(lit(3)));
    }

    #[test]
    fn subst_in_constructor() {
        let n_var = e::ValueExpression::Variable {
            name: "n".to_owned(),
            ty: nat_ty(),
        };
        let expr = suc(n_var);
        let result = apply_bindings(expr, &[("n".to_owned(), zero())]);
        assert_eq!(result, suc(zero()));
    }
}
