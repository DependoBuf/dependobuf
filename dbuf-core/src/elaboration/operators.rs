use crate::ast::elaborated as e;
use crate::ast::operators as o;
use crate::elaboration::builtins::BuiltinType;
use crate::elaboration::{builtins, subst};
use crate::error::elaborating::Error;
use crate::error::elaborating::Error::{
    TypeMismatch, UnknownConstructor, UnknownField, UnknownType, UnsupportedSyntax,
};
use std::fmt::Debug;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;

#[must_use]
pub fn unary_accepted_types<S>(op: &o::UnaryOp<S>) -> &[BuiltinType] {
    match op {
        o::UnaryOp::Access(_) => &[],
        o::UnaryOp::Minus => &[BuiltinType::Int],
        o::UnaryOp::Bang => &[BuiltinType::Bool],
    }
}

#[must_use]
pub fn binary_accepted_types(op: &o::BinaryOp) -> &[BuiltinType] {
    match op {
        o::BinaryOp::Plus => &[BuiltinType::UInt, BuiltinType::Int, BuiltinType::String],
        o::BinaryOp::Minus => &[BuiltinType::Int],
        o::BinaryOp::Star => &[BuiltinType::UInt, BuiltinType::Int],
        o::BinaryOp::BinaryAnd | o::BinaryOp::BinaryOr => &[BuiltinType::Bool],
    }
}

#[must_use]
pub fn literal_to_type<Str: From<BuiltinType>>(literal: &o::Literal) -> e::TypeExpression<Str> {
    builtins::get_builtin(&match literal {
        o::Literal::Bool(_) => BuiltinType::Bool,
        o::Literal::Int(_) => BuiltinType::Int,
        o::Literal::UInt(_) => BuiltinType::UInt,
        o::Literal::Str(_) => BuiltinType::String,
    })
}

/// # Errors
pub fn check_literal<Str: Eq + From<BuiltinType>>(
    literal: &o::Literal,
    expected_type: &e::TypeExpression<Str>,
) -> Result<(), Error> {
    match literal {
        literal if *expected_type == literal_to_type(literal) => Ok(()),
        o::Literal::UInt(_) if *expected_type == builtins::get_builtin(&BuiltinType::Int) => Ok(()),
        o::Literal::Int(value)
            if *expected_type == builtins::get_builtin(&BuiltinType::UInt) && *value >= 0 =>
        {
            Ok(())
        }
        _ => Err(TypeMismatch),
    }
}

#[must_use]
pub fn make_lit<Str: Clone>(
    literal: o::Literal,
    result_type: e::TypeExpression<Str>,
) -> e::ValueExpression<Str> {
    e::ValueExpression::OpCall {
        op_call: o::OpCall::Literal(literal),
        result_type,
    }
}

#[must_use]
pub fn make_unary<Str: Clone>(
    op: o::UnaryOp<Str>,
    arg: e::ValueExpression<Str>,
    result_type: e::TypeExpression<Str>,
) -> e::ValueExpression<Str> {
    e::ValueExpression::OpCall {
        op_call: o::OpCall::Unary(op, Arc::new(arg)),
        result_type,
    }
}

#[must_use]
pub fn make_binary<Str: Clone>(
    op: o::BinaryOp,
    lhs: e::ValueExpression<Str>,
    rhs: e::ValueExpression<Str>,
    result_type: e::TypeExpression<Str>,
) -> e::ValueExpression<Str> {
    e::ValueExpression::OpCall {
        op_call: o::OpCall::Binary(op, Arc::new(lhs), Arc::new(rhs)),
        result_type,
    }
}

/// # Errors
pub fn resolve_field_access<Str: Debug + Clone + Hash + Eq + Ord + From<BuiltinType> + Display>(
    module_ctx: &e::Module<Str>,
    operand_type: &e::TypeExpression<Str>,
    operand_value: &e::ValueExpression<Str>,
    field: &Str,
) -> Result<(e::Constructor<Str>, usize, e::TypeExpression<Str>), Error> {
    let e::TypeExpression::TypeExpression {
        name: type_name,
        dependencies: type_deps,
    } = operand_type;

    let ty = module_ctx
        .types
        .get(type_name)
        .ok_or_else(|| UnknownType(type_name.to_string()))?;

    let ctor_name = match &ty.constructor_names {
        e::ConstructorNames::OfMessage(ctor_name) => ctor_name.clone(),
        e::ConstructorNames::OfEnum(_) => return Err(UnsupportedSyntax),
    };

    let ctor = module_ctx
        .constructors
        .get(&ctor_name)
        .ok_or_else(|| UnknownConstructor(ctor_name.to_string()))?
        .clone();

    let (field_idx, (_, field_type)) = ctor
        .fields
        .iter()
        .enumerate()
        .find(|(_, (n, _))| n == field)
        .ok_or_else(|| UnknownField(field.to_string()))?;
    let after_implicits = ctor.implicits.iter().zip(type_deps.iter()).fold(
        field_type.clone(),
        |ty, ((implicit_name, _), concrete_val)| subst::subst_type(ty, implicit_name, concrete_val),
    );

    let (concrete_field_type, _) = ctor.fields[..field_idx].iter().fold(
        (
            after_implicits,
            Vec::<(Str, e::ValueExpression<Str>)>::new(),
        ),
        |(ty, mut prev_subs), (field_name, raw_field_type)| {
            let concrete_prev_type =
                subst::apply_bindings_to_type(raw_field_type.clone(), &prev_subs);
            let field_access = e::ValueExpression::OpCall {
                op_call: o::OpCall::Unary(
                    o::UnaryOp::Access(field_name.clone()),
                    e::Rec::new(operand_value.clone()),
                ),
                result_type: concrete_prev_type,
            };
            let new_ty = subst::subst_type(ty, field_name, &field_access);
            prev_subs.push((field_name.clone(), field_access));
            (new_ty, prev_subs)
        },
    );

    Ok((ctor, field_idx, concrete_field_type))
}
