use crate::ast::elaborated as e;
use crate::ast::operators as o;

pub fn apply_bindings<Str>(
    mut expr: e::ValueExpression<Str>,
    bindings: &[(Str, e::ValueExpression<Str>)],
) -> e::ValueExpression<Str>
where
    Str: Clone + Eq,
{
    for (var, replacement) in bindings {
        expr = subst_value(expr, var, replacement);
    }
    expr
}

pub fn apply_bindings_to_type<Str>(
    mut ty: e::TypeExpression<Str>,
    bindings: &[(Str, e::ValueExpression<Str>)],
) -> e::TypeExpression<Str>
where
    Str: Clone + Eq,
{
    for (var, replacement) in bindings {
        ty = subst_type(ty, var, replacement);
    }
    ty
}

pub fn subst_value<Str>(
    expr: e::ValueExpression<Str>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> e::ValueExpression<Str>
where
    Str: Clone + Eq,
{
    match expr {
        e::ValueExpression::Variable { ref name, .. } if name == var => replacement.clone(),

        e::ValueExpression::Variable { name, ty } => e::ValueExpression::Variable {
            name,
            ty: subst_type(ty, var, replacement),
        },

        e::ValueExpression::OpCall {
            op_call,
            result_type,
        } => e::ValueExpression::OpCall {
            op_call: subst_op_call(op_call, var, replacement),
            result_type: subst_type(result_type, var, replacement),
        },

        e::ValueExpression::Constructor {
            name,
            implicits,
            arguments,
            result_type,
        } => e::ValueExpression::Constructor {
            name,
            implicits: subst_value_exprs(&implicits, var, replacement),
            arguments: subst_value_exprs(&arguments, var, replacement),
            result_type: subst_type(result_type, var, replacement),
        },
    }
}

pub fn subst_type<Str>(
    ty: e::TypeExpression<Str>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> e::TypeExpression<Str>
where
    Str: Clone + Eq,
{
    let e::TypeExpression::TypeExpression { name, dependencies } = ty;
    e::TypeExpression::TypeExpression {
        name,
        dependencies: subst_value_exprs(&dependencies, var, replacement),
    }
}

fn subst_value_exprs<Str>(
    exprs: &e::ValueExprs<Str>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> e::ValueExprs<Str>
where
    Str: Clone + Eq,
{
    let substituted: Vec<_> = exprs
        .iter()
        .cloned()
        .map(|e| subst_value(e, var, replacement))
        .collect();
    e::Rec::from(substituted.as_slice())
}

fn subst_op_call<Str>(
    op: o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>,
    var: &Str,
    replacement: &e::ValueExpression<Str>,
) -> o::OpCall<Str, e::Rec<e::ValueExpression<Str>>>
where
    Str: Clone + Eq,
{
    match op {
        o::OpCall::Literal(lit) => o::OpCall::Literal(lit),

        o::OpCall::Unary(o::UnaryOp::Access(f), arg) => o::OpCall::Unary(
            o::UnaryOp::Access(f),
            e::Rec::new(subst_value((*arg).clone(), var, replacement)),
        ),

        o::OpCall::Unary(unary_op, arg) => o::OpCall::Unary(
            unary_op,
            e::Rec::new(subst_value((*arg).clone(), var, replacement)),
        ),

        o::OpCall::Binary(bin_op, lhs, rhs) => o::OpCall::Binary(
            bin_op,
            e::Rec::new(subst_value((*lhs).clone(), var, replacement)),
            e::Rec::new(subst_value((*rhs).clone(), var, replacement)),
        ),
    }
}
