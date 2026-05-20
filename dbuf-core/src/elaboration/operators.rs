use crate::ast::elaborated as e;
use crate::ast::operators as o;
use crate::elaboration::{builtins, subst};
use crate::error::elaborating::Error;
use crate::error::elaborating::Error::ElaboratingError;
use std::fmt::Debug;
use std::hash::Hash;

/// Returns the builtin type names that a unary operator accepts.
#[must_use]
pub fn unary_accepted_types<S>(op: &o::UnaryOp<S>) -> &'static [&'static str] {
    match op {
        o::UnaryOp::Access(_) => &[],
        o::UnaryOp::Minus => &["Int"],
        o::UnaryOp::Bang => &["Bool"],
    }
}

/// Returns the builtin type names that a binary operator accepts.
#[must_use]
pub fn binary_accepted_types(op: &o::BinaryOp) -> &'static [&'static str] {
    match op {
        o::BinaryOp::Plus => &["UInt", "Int", "String"],
        o::BinaryOp::Minus => &["Int"],
        o::BinaryOp::Star => &["UInt", "Int"],
        o::BinaryOp::BinaryAnd | o::BinaryOp::BinaryOr => &["Bool"],
    }
}

#[must_use]
pub fn literal_to_type<Str: From<String>>(literal: &o::Literal) -> e::TypeExpression<Str> {
    builtins::get_builtin(match literal {
        o::Literal::Bool(_) => "Bool",
        o::Literal::Int(_) => "Int",
        o::Literal::UInt(_) => "UInt",
        o::Literal::Str(_) => "String",
    })
}

/// Checks that a literal is of the expected type
///
/// # Arguments
///
/// * `literal`: literal to be checked
/// * `expected_type`: `TypeExpression` that the `literal` must satisfy
///
/// returns: Result<(), Error>
///
/// # Errors
/// * `ElaborationError`: when literal didn't match expected type
pub fn check_literal<Str: Eq + From<String>>(
    literal: &o::Literal,
    expected_type: &e::TypeExpression<Str>,
) -> Result<(), Error> {
    match literal {
        literal if *expected_type == literal_to_type(literal) => Ok(()),
        o::Literal::UInt(_) if *expected_type == builtins::get_builtin("Int") => Ok(()),
        o::Literal::Int(value)
            if *expected_type == builtins::get_builtin("UInt") && *value >= 0 =>
        {
            Ok(())
        }
        // Literal didn't match
        _ => Err(ElaboratingError),
    }
}

/// Gets the operand type and the name of the field being accessed.
/// Returns the constructor, the field's position within it, and the field's
/// *concrete* type — that is, the declared field type with the message's dependency
/// variables replaced by the concrete values carried in `operand_type`.
///
/// # Errors
/// Returns `Err` if the type is not a message, or if
/// no field with the given name exists.
pub fn resolve_field_access<Str: Debug + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    operand_type: &e::TypeExpression<Str>,
    field: &Str,
) -> Result<(e::Constructor<Str>, usize, e::TypeExpression<Str>), Error> {
    let e::TypeExpression::TypeExpression {
        name: type_name,
        dependencies: type_deps,
    } = operand_type;

    let (_, ty) = module_ctx
        .types
        .iter()
        .find(|(n, _)| n == type_name)
        .ok_or(ElaboratingError)?;

    let ctor_name = match &ty.constructor_names {
        e::ConstructorNames::OfMessage(ctor_name) => ctor_name.clone(),
        e::ConstructorNames::OfEnum(_) => return Err(ElaboratingError),
    };

    let ctor = module_ctx
        .constructors
        .get(&ctor_name)
        .ok_or(ElaboratingError)?
        .clone();

    let (field_idx, (_, field_type)) = ctor
        .fields
        .iter()
        .enumerate()
        .find(|(_, (n, _))| n == field)
        .ok_or(ElaboratingError)?;

    let concrete_field_type = ctor.implicits.iter().zip(type_deps.iter()).fold(
        field_type.clone(),
        |ty, ((implicit_name, _), concrete_val)| subst::subst_type(ty, implicit_name, concrete_val),
    );

    Ok((ctor, field_idx, concrete_field_type))
}
