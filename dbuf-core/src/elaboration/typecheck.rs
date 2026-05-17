use crate::ast::{elaborated as e, operators as o, parsed as p};
use crate::elaboration::context::Context;
use crate::elaboration::{apply, builtins, operators, rename, subst, unify};
use crate::error::elaborating::Error;
use crate::error::elaborating::Error::*;

use crate::arena::InternedString;
use crate::ast::operators::{OpCall, UnaryOp};
use crate::ast::parsed::ExpressionNode;
use crate::elaboration::operators::{binary_accepted_types, unary_accepted_types};
use crate::location::{LocatedName, Location, Offset};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::iter::zip;

pub type Loc = Location<Offset>;
pub type Name = LocatedName<InternedString, Offset>;
type Mod = e::Module<InternedString>;
type Ctx<'a> = Context<'a, InternedString, e::TypeExpression<InternedString>>;
type TypeExpr = e::TypeExpression<InternedString>;
type Value = e::ValueExpression<InternedString>;
type ElaboratedCtx = e::Context<InternedString>;

type ElaboratedDeclaration = (
    InternedString,
    e::Type<InternedString>,
    Vec<(InternedString, e::Constructor<InternedString>)>,
);

pub type Bindings = Vec<(InternedString, Value)>;

#[must_use]
pub fn elaborate(module: &p::Module<Loc, Name>) -> Mod {
    elaborate_with_module_ctx(builtins::builtins_module::<InternedString>(), module)
}

#[must_use]
pub fn elaborate_with_module_ctx(mut module_ctx: Mod, module: &p::Module<Loc, Name>) -> Mod {
    let mut elaborated_module = e::Module {
        types: vec![],
        constructors: BTreeMap::new(),
    };
    for type_def in module {
        let (name, elaborated_type, elaborated_constructors) =
            elaborate_type_decl(&module_ctx, type_def);

        module_ctx
            .types
            .push((name.clone(), elaborated_type.clone()));
        module_ctx
            .constructors
            .extend(elaborated_constructors.clone());

        elaborated_module.types.push((name, elaborated_type));
        elaborated_module
            .constructors
            .extend(elaborated_constructors);
    }
    elaborated_module
}

fn elaborate_type_decl(
    module_ctx: &Mod,
    type_def: &p::definition::Definition<Loc, Name, p::TypeDeclaration<Loc, Name>>,
) -> ElaboratedDeclaration {
    let local_ctx = Context::new();

    let p::definition::Definition {
        name,
        data: p::TypeDeclaration { dependencies, body },
        ..
    } = type_def;

    let mut binding = local_ctx.new_layer();
    let (local_ctx_with_deps, elaborated_dependencies) =
        check_dependencies(module_ctx, &mut binding, dependencies);
    let deps = rename::add_suffix_context(elaborated_dependencies.clone(), "_dep");

    match body {
        p::TypeDefinition::Message(ctor_body) => {
            let (name, fields) =
                elaborate_message(module_ctx, &local_ctx_with_deps, name, ctor_body);

            (
                name.clone(),
                e::Type {
                    dependencies: deps,
                    constructor_names: e::ConstructorNames::OfMessage(name.clone()),
                },
                vec![(
                    name.clone(),
                    e::Constructor {
                        implicits: elaborated_dependencies.clone(),
                        fields,
                        result_type: e::TypeExpression::TypeExpression {
                            name: name.clone(),
                            dependencies: e::Rec::from(
                                elaborated_dependencies
                                    .iter()
                                    .map(|(n, v)| e::ValueExpression::Variable {
                                        name: n.clone(),
                                        ty: v.clone(),
                                    })
                                    .collect::<Vec<_>>(),
                            ),
                        },
                    },
                )],
            )
        }
        p::TypeDefinition::Enum(branches) => {
            let (constructor_names, constructors) =
                elaborate_enum(module_ctx, &local_ctx_with_deps, name, branches);
            (
                name.content.clone(),
                e::Type {
                    dependencies: deps,
                    constructor_names: e::ConstructorNames::OfEnum(constructor_names),
                },
                constructors,
            )
        }
    }
}

fn check_dependencies<'a>(
    module_ctx: &Mod,
    local_ctx: &'a mut Ctx<'a>,
    deps: &p::definition::Definitions<Loc, Name, p::TypeExpression<Loc, Name>>,
) -> (Ctx<'a>, ElaboratedCtx) {
    let mut dependencies = vec![];
    for p::definition::Definition {
        name,
        data: type_expr,
        ..
    } in deps
    {
        let elaborated_type_expr =
            elaborate_type(module_ctx, local_ctx, type_expr).expect("invalid context");

        local_ctx.insert(name.content.clone(), elaborated_type_expr.clone());
        dependencies.push((name.content.clone(), elaborated_type_expr));
    }
    (local_ctx.new_layer(), dependencies)
}

fn elaborate_message<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    name: &Name,
    ctor_body: &p::ConstructorBody<Loc, Name>,
) -> (InternedString, ElaboratedCtx) {
    let mut fields: ElaboratedCtx = Vec::new();
    let mut field_ctx = local_ctx.new_layer();

    for p::definition::Definition { name, data, .. } in ctor_body {
        let elaborated_type = elaborate_type(module_ctx, &field_ctx, data).expect("elaborate");
        field_ctx.insert(name.content.clone(), elaborated_type.clone());
        fields.push((name.content.clone(), elaborated_type));
    }

    (name.content.clone(), fields)
}

/// # Panics
/// # Errors
pub fn elaborate_type<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    expr: &p::TypeExpression<Loc, Name>,
) -> Result<TypeExpr, Error> {
    match &expr.node {
        p::ExpressionNode::OpCall(_) => {
            // No type-level operator support.
            Err(ElaboratingError)
        }
        p::ExpressionNode::FunCall { fun, args } => {
            let (_, ty) = module_ctx
                .types
                .iter()
                .find(|(name, _)| *name == fun.content)
                .ok_or(ElaboratingError)?;

            let declared_deps = ty.dependencies.clone();

            match declared_deps.len().cmp(&args.len()) {
                Ordering::Less | Ordering::Greater => return Err(ElaboratingError),
                Ordering::Equal => {}
            }

            let mut remaining_deps = declared_deps;
            let mut elaborated_args = Vec::new();

            for arg in args.iter() {
                let (dep_name, dep_type) = remaining_deps.remove(0);

                let (arg_value, bindings) = check(module_ctx, local_ctx, arg, &dep_type)?;
                assert_eq!(bindings.len(), 0);
                elaborated_args.push(arg_value.clone());

                for (_, remaining_ty) in &mut remaining_deps {
                    let after_name = subst::subst_type(remaining_ty.clone(), &dep_name, &arg_value);
                    *remaining_ty = subst::apply_bindings_to_type(after_name, &bindings);
                }
            }

            Ok(e::TypeExpression::TypeExpression {
                name: fun.content.clone(),
                dependencies: e::Rec::from(elaborated_args),
            })
        }
        p::ExpressionNode::ConstructorCall { .. } => Err(ElaboratingError),
        p::ExpressionNode::Variable { name } => {
            local_ctx
                .get(&name.content)
                .cloned()
                // Unknown variable
                .ok_or(ElaboratingError)
        }
        p::ExpressionNode::TypedHole => {
            // Can't infer type for type hole
            Err(ElaboratingError)
        }
    }
}

/// Elaborate parsed expression into value
/// # Panics
/// # Errors
pub fn elaborate_value<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    expr: &p::Expression<Loc, Name>,
) -> Result<Value, Error> {
    match &expr.node {
        p::ExpressionNode::OpCall(op_call) => {
            elaborate_operator_value(module_ctx, local_ctx, op_call)
        }
        p::ExpressionNode::FunCall { .. } => {
            // No generics
            Err(ElaboratingError)
        }
        p::ExpressionNode::ConstructorCall {
            name,
            fields: constructor_args,
        } => elaborate_constructor_call(module_ctx, local_ctx, name, constructor_args),
        p::ExpressionNode::Variable { name } => {
            let ty = local_ctx
                .get(&name.content)
                .cloned()
                .ok_or(ElaboratingError)?;
            Ok(e::ValueExpression::Variable {
                name: name.content.clone(),
                ty,
            })
        }
        p::ExpressionNode::TypedHole => Err(ElaboratingError),
    }
}

/// Elaborate parsed operator expression into operator value
/// # Panics
/// # Errors
pub fn elaborate_operator_value<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    op_expr: &o::OpCall<Name, e::Rec<p::Expression<Loc, Name>>>,
) -> Result<Value, Error> {
    match op_expr {
        o::OpCall::Literal(literal) => Ok(e::ValueExpression::OpCall {
            op_call: o::OpCall::Literal(literal.clone()),
            result_type: operators::literal_to_type(literal),
        }),
        o::OpCall::Unary(op, value) => {
            if let o::UnaryOp::Access(field) = op {
                if let p::ExpressionNode::ConstructorCall {
                    name: ctor_name,
                    fields: ctor_args,
                } = &value.node
                {
                    let ctor = module_ctx
                        .constructors
                        .get(&ctor_name.content)
                        .ok_or(ElaboratingError)?;
                    let field_idx = ctor
                        .fields
                        .iter()
                        .position(|(n, _)| *n == field.content)
                        .ok_or(ElaboratingError)?;
                    let field_arg = ctor_args.get(field_idx).ok_or(ElaboratingError)?;
                    return elaborate_value(module_ctx, local_ctx, &field_arg.data);
                }

                let elaborated_expr = elaborate_value(module_ctx, local_ctx, value)?;
                let operand_type = infer(module_ctx, local_ctx, value)?;
                let (_, _, concrete_field_type) =
                    operators::resolve_field_access(module_ctx, &operand_type, &field.content)?;

                Ok(e::ValueExpression::OpCall {
                    op_call: o::OpCall::Unary(
                        o::UnaryOp::Access(field.content.clone()),
                        e::Rec::new(elaborated_expr),
                    ),
                    result_type: concrete_field_type,
                })
            } else {
                let accepted_types = operators::unary_accepted_types(op);
                let Some((elaborated_val, ty)) = accepted_types.iter().find_map(|x| {
                    let builtin = builtins::get_builtin(x);
                    let (v, bindings) = check(module_ctx, local_ctx, value, &builtin).ok()?;
                    assert_eq!(bindings.len(), 0);
                    Some((v, builtin))
                }) else {
                    return Err(ElaboratingError);
                };
                let op_is: o::UnaryOp<InternedString> = match op {
                    o::UnaryOp::Minus => o::UnaryOp::Minus,
                    o::UnaryOp::Bang => o::UnaryOp::Bang,
                    o::UnaryOp::Access(_) => unreachable!(),
                };
                Ok(e::ValueExpression::OpCall {
                    op_call: o::OpCall::Unary(op_is, e::Rec::from(elaborated_val)),
                    result_type: ty,
                })
            }
        }
        o::OpCall::Binary(op, l, r) => {
            let binary_op = |accepted_types: &[&str]| -> Result<Value, Error> {
                let Some((builtin, left, right)) = accepted_types.iter().find_map(|x| {
                    let builtin = builtins::get_builtin(x);
                    let (left, bindings) = check(module_ctx, local_ctx, l, &builtin).ok()?;
                    assert_eq!(bindings.len(), 0);

                    let (right, bindings) = check(module_ctx, local_ctx, r, &builtin).ok()?;
                    assert_eq!(bindings.len(), 0);
                    Some((builtin, left, right))
                }) else {
                    return Err(ElaboratingError);
                };
                Ok(e::ValueExpression::OpCall {
                    op_call: o::OpCall::Binary(*op, e::Rec::from(left), e::Rec::from(right)),
                    result_type: builtin,
                })
            };
            binary_op(operators::binary_accepted_types(op))
        }
    }
}

fn elaborate_constructor_call<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    name: &Name,
    constructor_args: &p::definition::Definitions<Loc, Name, p::Expression<Loc, Name>>,
) -> Result<Value, Error> {
    let constructor = module_ctx
        .constructors
        .get(&name.content)
        .ok_or(ElaboratingError)?;

    let e::Constructor { fields, .. } = constructor;

    match fields.len().cmp(&constructor_args.len()) {
        Ordering::Less | Ordering::Greater => Err(ElaboratingError),
        Ordering::Equal => {
            let mut applied_constructor = constructor.clone();
            let mut arguments_expression = Vec::new();
            let mut all_implicit_bindings: Bindings = Vec::new();
            for (p::definition::Definition { data, .. }, (_var_name, ty)) in
                zip(constructor_args, fields.iter())
            {
                let (data_value, check_bindings) = check(module_ctx, local_ctx, data, ty)?;

                all_implicit_bindings.extend(check_bindings);

                let (new_ctor, argument_bindings) =
                    apply::application(&applied_constructor, &data_value, module_ctx)?;

                let applied_data_value = subst::apply_bindings(data_value, &argument_bindings);
                arguments_expression.push(applied_data_value.clone());

                applied_constructor = new_ctor;
            }

            let e::Constructor {
                implicits,
                result_type,
                ..
            } = applied_constructor;

            let resolved_implicits = implicits
                .into_iter()
                .map(|(implicit_name, implicit_ty)| {
                    all_implicit_bindings
                        .iter()
                        .find(|(n, _)| n == &implicit_name)
                        .map(|(_, v)| v.clone())
                        .unwrap_or(e::ValueExpression::Variable {
                            name: implicit_name,
                            ty: implicit_ty,
                        })
                })
                .collect::<Vec<_>>();

            Ok(e::ValueExpression::Constructor {
                name: name.content.clone(),
                implicits: e::Rec::from(resolved_implicits),
                arguments: e::Rec::from(arguments_expression),
                result_type,
            })
        }
    }
}

fn elaborate_enum<'a>(
    _module_ctx: &Mod,
    _local_ctx: &'a Ctx<'a>,
    _name: &Name,
    _branches: &Vec<p::EnumBranch<Loc, Name>>,
) -> (
    BTreeSet<InternedString>,
    Vec<(InternedString, e::Constructor<InternedString>)>,
) {
    todo!("enum elaboration not implemented")
}

/// # Panics
/// # Errors
pub fn infer<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    expression: &p::Expression<Loc, Name>,
) -> Result<TypeExpr, Error> {
    match &expression.node {
        p::ExpressionNode::OpCall(op) => {
            let infer_builtin = |accepted: &[&str], expr| {
                accepted
                    .iter()
                    .find_map(|ty_name| {
                        let ty = builtins::get_builtin(ty_name);
                        check(module_ctx, local_ctx, expr, &ty).ok()?;
                        Some(ty)
                    })
                    .ok_or(ElaboratingError)
            };
            match op {
                o::OpCall::Literal(literal) => Ok(operators::literal_to_type(literal)),
                o::OpCall::Unary(un_op, arg) => match un_op {
                    UnaryOp::Access(field) => {
                        let operand_type = infer(module_ctx, local_ctx, arg)?;
                        let (_, _, concrete_field_type) = operators::resolve_field_access(
                            module_ctx,
                            &operand_type,
                            &field.content,
                        )?;
                        Ok(concrete_field_type)
                    }
                    UnaryOp::Minus | UnaryOp::Bang => {
                        infer_builtin(unary_accepted_types(un_op), arg)
                    }
                },
                OpCall::Binary(bi_op, _l_arg, r_arg) => {
                    infer_builtin(binary_accepted_types(bi_op), r_arg)
                }
            }
        }
        p::ExpressionNode::FunCall { .. } => {
            todo!(
                "It's expected that only Type can be here. Argument types and arity need to be checked"
            )
        }
        p::ExpressionNode::ConstructorCall { .. } => {
            let elaborated = elaborate_value(module_ctx, local_ctx, expression)?;
            match elaborated {
                e::ValueExpression::Constructor { result_type, .. } => Ok(result_type),
                _ => unreachable!(),
            }
        }
        p::ExpressionNode::Variable { name } => {
            local_ctx
                .get(&name.content)
                .cloned()
                // Unknown variable
                .ok_or(ElaboratingError)
        }
        p::ExpressionNode::TypedHole => {
            // Type hole can't be inferred
            Err(ElaboratingError)
        }
    }
}

/// # Panics
/// # Errors
pub fn check<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    expression: &p::Expression<Loc, Name>,
    expected_type: &TypeExpr,
) -> Result<(Value, Bindings), Error> {
    match &expression.node {
        p::ExpressionNode::OpCall(op) => match op {
            OpCall::Literal(literal) => {
                operators::check_literal(literal, expected_type)?;
                return Ok((
                    e::ValueExpression::OpCall {
                        op_call: o::OpCall::Literal(literal.clone()),
                        result_type: expected_type.clone(),
                    },
                    vec![],
                ));
            }
            OpCall::Unary(op, arg) => {
                if let UnaryOp::Access(_) = op {
                } else {
                    let accepted = operators::unary_accepted_types(op);
                    if !accepted
                        .iter()
                        .any(|t| *expected_type == builtins::get_builtin(t))
                    {
                        return Err(ElaboratingError);
                    }
                    let (checked_arg, bindings) = check(module_ctx, local_ctx, arg, expected_type)?;
                    assert_eq!(bindings.len(), 0);
                    let op_is: o::UnaryOp<InternedString> = match op {
                        UnaryOp::Minus => UnaryOp::Minus,
                        UnaryOp::Bang => UnaryOp::Bang,
                        UnaryOp::Access(_) => unreachable!(),
                    };
                    return Ok((
                        e::ValueExpression::OpCall {
                            op_call: o::OpCall::Unary(op_is, e::Rec::from(checked_arg)),
                            result_type: expected_type.clone(),
                        },
                        vec![],
                    ));
                }
            }
            OpCall::Binary(op, l, r) => {
                let accepted = operators::binary_accepted_types(op);
                if !accepted
                    .iter()
                    .any(|t| *expected_type == builtins::get_builtin(t))
                {
                    return Err(ElaboratingError);
                }
                let (checked_l, bindings_l) = check(module_ctx, local_ctx, l, expected_type)?;
                assert_eq!(bindings_l.len(), 0);
                let (checked_r, bindings_r) = check(module_ctx, local_ctx, r, expected_type)?;
                assert_eq!(bindings_r.len(), 0);
                return Ok((
                    e::ValueExpression::OpCall {
                        op_call: o::OpCall::Binary(
                            *op,
                            e::Rec::from(checked_l),
                            e::Rec::from(checked_r),
                        ),
                        result_type: expected_type.clone(),
                    },
                    vec![],
                ));
            }
        },

        ExpressionNode::ConstructorCall { name, fields } => {
            return check_constructor_call(module_ctx, local_ctx, name, fields, expected_type);
        }
        _ => {}
    }
    let elaborated = elaborate_value(module_ctx, local_ctx, expression)?;
    let inferred_type = infer(module_ctx, local_ctx, expression)?;

    let (left_bindings, right_bindings) =
        unify::unify_type(&inferred_type, expected_type, module_ctx)
            .map_err(|_| ElaboratingError)?;
    assert_eq!(left_bindings.len(), 0);
    if !left_bindings.is_empty() && !right_bindings.is_empty() {
        let elaborated_with_binds = subst::apply_bindings(elaborated, &left_bindings);
        return Ok((elaborated_with_binds, right_bindings));
    }
    Ok((elaborated, right_bindings))
}

/// Check a constructor call against an expected type.
fn check_constructor_call<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    name: &Name,
    constructor_args: &p::definition::Definitions<Loc, Name, p::Expression<Loc, Name>>,
    expected_type: &TypeExpr,
) -> Result<(Value, Bindings), Error> {
    let elaborated = elaborate_constructor_call(module_ctx, local_ctx, name, constructor_args)?;

    let result_type = match &elaborated {
        e::ValueExpression::Constructor { result_type, .. } => result_type.clone(),
        _ => unreachable!(),
    };

    let e::TypeExpression::TypeExpression {
        name: ref inferred_name,
        dependencies: ref inferred_deps,
    } = result_type;
    let e::TypeExpression::TypeExpression {
        name: ref expected_name,
        dependencies: ref expected_deps,
    } = *expected_type;

    if inferred_name != expected_name || inferred_deps.len() != expected_deps.len() {
        return Err(ElaboratingError);
    }

    let mut left_bindings: Bindings = vec![];
    let mut right_bindings: Bindings = vec![];

    for (inferred_dep, expected_dep) in zip(inferred_deps.iter(), expected_deps.iter()) {
        match inferred_dep {
            e::ValueExpression::Variable { name, .. } => {
                left_bindings.push((name.clone(), expected_dep.clone()));
            }
            concrete => {
                let (lb, rb) =
                    unify::unify_value(concrete, expected_dep).map_err(|_| ElaboratingError)?;
                left_bindings.extend(lb);
                right_bindings.extend(rb);
            }
        }
    }

    Ok((
        subst::apply_bindings(elaborated, &left_bindings),
        right_bindings,
    ))
}
