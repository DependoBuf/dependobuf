use crate::ast::{elaborated as e, operators as o, parsed as p};
use crate::error::elaborating::Error;
use crate::error::elaborating::Error::*;
use crate::typecheck::context::Context;
use crate::typecheck::rename::{Rename, add_suffix_context};
use crate::typecheck::subst::apply_bindings;
use crate::typecheck::{apply, subst, unify};

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::zip;

type ElaboratedDeclaration<Str> = (Str, e::Type<Str>, Vec<(Str, e::Constructor<Str>)>);

#[must_use]
pub fn new_global<Str: Eq + Hash + From<String> + Clone>() -> Vec<(Str, e::Type<Str>)> {
    let mut ctx = Vec::new();
    for builtin in ["Bool", "Double", "Int", "UInt", "String", "Type"] {
        ctx.push((
            Str::from(builtin.to_string()),
            e::Type {
                dependencies: vec![],
                constructor_names: e::ConstructorNames::OfEnum(BTreeSet::new()),
            },
        ));
    }
    ctx
}

fn get_builtin<Str: From<String>>(type_name: &str) -> e::TypeExpression<Str> {
    e::TypeExpression::TypeExpression {
        name: type_name.to_string().into(),
        dependencies: e::Rec::new([]),
    }
}

#[must_use]
pub fn elaborate<Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord + Rename>(
    module: &p::Module<Loc, Str>,
) -> e::Module<Str> {
    let elaborated_module = e::Module {
        types: new_global::<Str>(),
        constructors: BTreeMap::new(),
    };
    elaborate_with_module_ctx(elaborated_module, module)
}

#[must_use]
pub fn elaborate_with_module_ctx<
    Loc,
    Str: Debug + From<String> + Clone + Hash + Eq + Ord + Rename,
>(
    mut module_ctx: e::Module<Str>,
    module: &p::Module<Loc, Str>,
) -> e::Module<Str> {
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

fn elaborate_type_decl<Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord + Rename>(
    module_ctx: &e::Module<Str>,
    type_def: &p::definition::Definition<Loc, Str, p::TypeDeclaration<Loc, Str>>,
) -> ElaboratedDeclaration<Str> {
    let local_ctx = Context::new();

    let p::definition::Definition {
        name,
        data: p::TypeDeclaration { dependencies, body },
        ..
    } = type_def;

    let mut binding = local_ctx.new_layer();
    let (local_ctx_with_deps, elaborated_dependencies) =
        check_dependencies(module_ctx, &mut binding, dependencies);
    let deps = add_suffix_context(elaborated_dependencies.clone(), "_dep");

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
                name.clone(),
                e::Type {
                    dependencies: deps,
                    constructor_names: e::ConstructorNames::OfEnum(constructor_names),
                },
                constructors,
            )
        }
    }
}
fn check_dependencies<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a mut Context<'a, Str, e::TypeExpression<Str>>,
    deps: &p::definition::Definitions<Loc, Str, p::TypeExpression<Loc, Str>>,
) -> (Context<'a, Str, e::TypeExpression<Str>>, e::Context<Str>) {
    let mut dependencies = vec![];
    for p::definition::Definition {
        name,
        data: type_expr,
        ..
    } in deps
    {
        let elaborated_type_expr: e::TypeExpression<Str> =
            elaborate_type(module_ctx, local_ctx, type_expr).expect("invalid context");

        local_ctx.insert(name.clone(), elaborated_type_expr.clone());
        dependencies.push((name.clone(), elaborated_type_expr));
    }
    (local_ctx.new_layer(), dependencies)
}

fn elaborate_message<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    name: &Str,
    ctor_body: &p::ConstructorBody<Loc, Str>,
) -> (Str, e::Context<Str>) {
    let mut fields: e::Context<Str> = Vec::new();
    let mut field_ctx = local_ctx.new_layer();

    for p::definition::Definition { name, data, .. } in ctor_body {
        let elaborated_type = elaborate_type(module_ctx, &field_ctx, data).expect("elaborate");
        field_ctx.insert(name.clone(), elaborated_type.clone());
        fields.push((name.clone(), elaborated_type));
    }

    (name.clone(), fields)
}

/// # Panics
/// # Errors
pub fn elaborate_type<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    expr: &p::TypeExpression<Loc, Str>,
) -> Result<e::TypeExpression<Str>, Error> {
    match &expr.node {
        p::ExpressionNode::OpCall(_) => {
            // No type-level operator support.
            Err(ElaboratingError)
        }
        p::ExpressionNode::FunCall { fun, args } => {
            let (_, ty) = module_ctx
                .types
                .iter()
                .find(|(name, _)| name == fun)
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
                name: fun.clone(),
                dependencies: e::Rec::from(elaborated_args),
            })
        }
        p::ExpressionNode::ConstructorCall { .. } => Err(ElaboratingError),
        p::ExpressionNode::Variable { name } => {
            local_ctx
                .get(name)
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

/// # Panics
/// # Errors
pub fn elaborate_value<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    expr: &p::Expression<Loc, Str>,
) -> Result<e::ValueExpression<Str>, Error> {
    match &expr.node {
        p::ExpressionNode::OpCall(op_call) => match op_call {
            o::OpCall::Literal(literal) => Ok(e::ValueExpression::OpCall {
                op_call: o::OpCall::Literal(literal.clone()),
                result_type: get_builtin(match literal {
                    o::Literal::Bool(_) => "Bool",
                    o::Literal::Double(_) => "Double",
                    o::Literal::Int(_) => "Int",
                    o::Literal::UInt(_) => "UInt",
                    o::Literal::Str(_) => "Str",
                }),
            }),
            o::OpCall::Unary(op, value) => {
                let unaru_op =
                    |accepted_types: Vec<&str>| -> Result<e::ValueExpression<Str>, Error> {
                        let Some((value, ty)) = accepted_types.iter().find_map(|x| {
                            let builtin = get_builtin(x);
                            let (value, bindings) =
                                check(module_ctx, local_ctx, value, &builtin).ok()?;
                            assert_eq!(bindings.len(), 0);
                            Some((value, builtin))
                        }) else {
                            return Err(ElaboratingError);
                        };
                        Ok(e::ValueExpression::OpCall {
                            op_call: o::OpCall::Unary(op.clone(), e::Rec::from(value)),
                            result_type: ty,
                        })
                    };
                match op {
                    o::UnaryOp::Access(_field) => {
                        todo!("dot access is not yet implemented")
                    }
                    o::UnaryOp::Minus => unaru_op(vec!["Int", "Double"]),
                    o::UnaryOp::Bang => unaru_op(vec!["Bool"]),
                }
            }
            o::OpCall::Binary(op, l, r) => {
                let binary_op =
                    |accepted_types: Vec<&str>| -> Result<e::ValueExpression<Str>, Error> {
                        let Some((builtin, left, right)) = accepted_types.iter().find_map(|x| {
                            let builtin = get_builtin(x);
                            let (left, bindings) =
                                check(module_ctx, local_ctx, l, &builtin).ok()?;
                            assert_eq!(bindings.len(), 0);

                            let (right, bindings) =
                                check(module_ctx, local_ctx, r, &builtin).ok()?;
                            assert_eq!(bindings.len(), 0);
                            Some((builtin, left, right))
                        }) else {
                            return Err(ElaboratingError);
                        };
                        Ok(e::ValueExpression::OpCall {
                            op_call: o::OpCall::Binary(
                                *op,
                                e::Rec::from(left),
                                e::Rec::from(right),
                            ),
                            result_type: builtin,
                        })
                    };
                match op {
                    o::BinaryOp::Plus => binary_op(vec!["Int", "Double", "UInt", "String"]),
                    o::BinaryOp::Minus | o::BinaryOp::Star | o::BinaryOp::Slash => {
                        binary_op(vec!["Int", "Double", "UInt"])
                    }
                    o::BinaryOp::BinaryAnd | o::BinaryOp::BinaryOr => binary_op(vec!["Bool"]),
                }
            }
        },
        p::ExpressionNode::FunCall { .. } => {
            // No generics
            Err(ElaboratingError)
        }
        p::ExpressionNode::ConstructorCall {
            name,
            fields: constructor_args,
        } => {
            elaborate_constructor_call(module_ctx, local_ctx, name, constructor_args)
        }
        p::ExpressionNode::Variable { name } => {
            let ty = local_ctx
                .get(name).cloned()
                .ok_or(ElaboratingError)?;
            Ok(e::ValueExpression::Variable {
                name: name.clone(),
                ty,
            })
        }
        p::ExpressionNode::TypedHole => {
            todo!("elaboration of type hole results in an error")
        }
    }
}

fn elaborate_constructor_call<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    name: &Str,
    constructor_args: &p::definition::Definitions<Loc, Str, p::Expression<Loc, Str>>,
) -> Result<e::ValueExpression<Str>, Error> {
    let constructor = module_ctx.constructors.get(name).ok_or(ElaboratingError)?;

    let e::Constructor { fields, .. } = constructor;

    match fields.len().cmp(&constructor_args.len()) {
        // arity < args
        Ordering::Less => Err(ElaboratingError),
        // arity > args
        Ordering::Greater => Err(ElaboratingError),
        Ordering::Equal => {
            let mut applied_constructor = constructor.clone();
            let mut arguments_expression = Vec::new();
            let mut all_implicit_bindings: Bindings<Str> = Vec::new();
            for (p::definition::Definition { data, .. }, (_var_name, ty)) in
                zip(constructor_args, fields.iter())
            {
                let (data_value, check_bindings) = check(module_ctx, local_ctx, data, ty)?;

                all_implicit_bindings.extend(check_bindings);

                let (new_ctor, argument_bindings) =
                    apply::application(&applied_constructor, &data_value, module_ctx)?;

                let applied_data_value = apply_bindings(data_value, &argument_bindings);
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
                name: name.clone(),
                implicits: e::Rec::from(resolved_implicits),
                arguments: e::Rec::from(arguments_expression),
                result_type,
            })
        }
    }
}

fn elaborate_enum<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    _module_ctx: &e::Module<Str>,
    _local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    _name: &Str,
    _branches: &Vec<p::EnumBranch<Loc, Str>>,
) -> (BTreeSet<Str>, Vec<(Str, e::Constructor<Str>)>) {
    todo!("enum elaboration not implemented")
}

/// # Panics
/// # Errors
pub fn infer<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    expression: &p::Expression<Loc, Str>,
) -> Result<e::TypeExpression<Str>, Error> {
    match &expression.node {
        p::ExpressionNode::OpCall(op) => match op {
            o::OpCall::Literal(literal) => Ok(match literal {
                o::Literal::Bool(_) => get_builtin("Bool"),
                o::Literal::Double(_) => get_builtin("Double"),
                o::Literal::Int(_) => get_builtin("Int"),
                o::Literal::UInt(_) => get_builtin("UInt"),
                o::Literal::Str(_) => get_builtin("Str"),
            }),
            o::OpCall::Unary(_op, _value) => {
                todo!("unary operation")
            }
            o::OpCall::Binary(_, _, _) => {
                todo!("binary operation")
            }
        },
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
            // Find in the local context, otherwise error
            local_ctx
                .get(name).cloned()
                // Unknown variable
                .ok_or(ElaboratingError)
        }
        p::ExpressionNode::TypedHole => {
            // Type hole can't be inferred
            Err(ElaboratingError)
        }
    }
}

pub type Bindings<Str> = Vec<(Str, e::ValueExpression<Str>)>;

/// # Panics
/// # Errors
pub fn check<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    expression: &p::Expression<Loc, Str>,
    expected_type: &e::TypeExpression<Str>,
) -> Result<(e::ValueExpression<Str>, Bindings<Str>), Error> {
    let elaborated = elaborate_value(module_ctx, local_ctx, expression)?;
    let inferred_type = infer(module_ctx, local_ctx, expression)?;

    let (left_bindings, right_bindings) =
        unify::unify_type(&inferred_type, expected_type, module_ctx)
            .map_err(|_| ElaboratingError)?;
    assert_eq!(left_bindings.len(), 0);
    if !left_bindings.is_empty() && !right_bindings.is_empty() {
        let elaborated_with_binds = apply_bindings(elaborated, &left_bindings);
        return Ok((elaborated_with_binds, right_bindings));
    }
    Ok((elaborated, right_bindings))
}
