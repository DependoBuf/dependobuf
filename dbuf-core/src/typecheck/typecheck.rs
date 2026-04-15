use crate::ast::{elaborated as e, operators as o, parsed as p};
use std::cmp::Ordering;

use crate::error::elaborating::Error::*;
use crate::typecheck::context::Context;

use std::collections::{BTreeMap, BTreeSet};

use crate::error::elaborating::Error;

use std::fmt::Debug;
use std::hash::Hash;
use std::iter::zip;
use std::ops::Add;
use crate::typecheck::rename::{add_suffix};

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

fn get_type<Str: From<String>>() -> e::TypeExpression<Str> {
    get_builtin("Type")
}

pub fn elaborate<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord + Add<&'a str, Output = Str>>(
    module: &p::Module<Loc, Str>,
) -> e::Module<Str> {
    let elaborated_module = e::Module {
        types: new_global::<Str>(),
        constructors: BTreeMap::new(),
    };
    elaborate_with_module_ctx(elaborated_module, &module)
}

pub fn elaborate_with_module_ctx<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord + Add<&'a str, Output=Str>>(
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

fn elaborate_type_decl<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Add<&'a str, Output = Str> + Ord>(
    module_ctx: &e::Module<Str>,
    type_def: &p::definition::Definition<Loc, Str, p::TypeDeclaration<Loc, Str>>,
) -> (Str, e::Type<Str>, Vec<(Str, e::Constructor<Str>)>) {
    let local_ctx = Context::new();

    let p::definition::Definition {
        name,
        data: p::TypeDeclaration { dependencies, body },
        ..
    } = type_def;

    let mut binding = local_ctx.new_layer();
    let (local_ctx_with_deps, elaborated_dependencies) =
        check_dependencies(module_ctx, &mut binding, dependencies);

    match body {
        p::TypeDefinition::Message(ctor_body) => {
            let (name, fields) = elaborate_message(module_ctx, &local_ctx_with_deps, &name, ctor_body);

            (name.clone(), e::Type {
                dependencies: elaborated_dependencies.clone(),
                constructor_names: e::ConstructorNames::OfMessage(name.clone()),
            }, vec![
                (name.clone(),
                    add_suffix(e::Constructor {
                        implicits: elaborated_dependencies.clone(),
                        fields,
                        result_type: e::TypeExpression::TypeExpression {
                            name: name.clone(),
                            dependencies: e::Rec::from(elaborated_dependencies.iter().map(|(n, v)| {
                                e::ValueExpression::Variable { name: n.clone(), ty: v.clone() }
                            }).collect::<Vec<_>>()),
                        },
                    }, "_dep")
                )
            ])
        }
        p::TypeDefinition::Enum(branches) => {
            elaborate_enum(module_ctx, &local_ctx_with_deps, &name, branches)
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
            elaborate_type(module_ctx, local_ctx, type_expr).expect("invalud context");

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

    for p::definition::Definition { loc, name, data } in ctor_body {
        let elaborated_type = elaborate_type(module_ctx, local_ctx, &data).expect("elaborate");

        fields.push((name.clone(), elaborated_type));
    }

    (
        name.clone(),
        fields
    )
}

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
            let mut arguments = vec![];
            for arg in args.iter() {
                let value = elaborate_value(module_ctx, local_ctx, arg)?;
                arguments.push(value);
            }

            Ok(e::TypeExpression::TypeExpression {
                name: fun.clone(),
                dependencies: e::Rec::from(arguments),
            })
        }
        p::ExpressionNode::ConstructorCall { .. } => {
            todo!()
        }
        p::ExpressionNode::Variable { name } => {
            local_ctx
                .get(name)
                .map(e::TypeExpression::clone)
                // Unknown variable
                .ok_or(ElaboratingError)
        }
        p::ExpressionNode::TypedHole => {
            // Can't infer type for type hole
            Err(ElaboratingError)
        }
    }
}

pub fn elaborate_value<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq + Ord>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    expr: &p::Expression<Loc, Str>,
) -> Result<e::ValueExpression<Str>, Error> {
    match &expr.node {
        p::ExpressionNode::OpCall(_) => {
            todo!("evaluate algebraic expression")
        }
        p::ExpressionNode::FunCall { .. } => {
            // No generics
            Err(ElaboratingError)
        }
        p::ExpressionNode::ConstructorCall { name, fields } => {
            let constructor = module_ctx.constructors.get(name).ok_or(ElaboratingError)?;

            todo!("apply constructor to elaborated arguments")
        }
        p::ExpressionNode::Variable { name } => {
            let ty= local_ctx.get(name).map(e::TypeExpression::clone).ok_or(ElaboratingError)?;
            Ok(e::ValueExpression::Variable { name: name.clone(), ty  })
        }
        p::ExpressionNode::TypedHole => {
            todo!()
        }
    }
}

fn elaborate_enum<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    name: &Str,
    branches: &Vec<p::EnumBranch<Loc, Str>>,
) -> (Str, e::Type<Str>, Vec<(Str, e::Constructor<Str>)>) {
    todo!("enum elaboration");
}

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
            o::OpCall::Unary(op, value) => {
                todo!("unary operation")
            }
            o::OpCall::Binary(_, _, _) => {
                todo!("binary operation")
            }
        },
        p::ExpressionNode::FunCall { fun, args } => {
            todo!(
                "It's expected that only Type can be here. Argument types and arity need to be checked"
            )
        }
        p::ExpressionNode::ConstructorCall {
            name: constructor_name,
            fields: constructor_args,
        } => {
            let (name, constructor) = module_ctx
                .constructors
                .iter()
                .find(|(name, _)| constructor_name == *name)
                // Unknown constructor
                .ok_or(ElaboratingError)?;

            assert_eq!(name, constructor_name);

            let e::Constructor {
                implicits,
                fields,
                result_type,
            } = constructor;

            let e::TypeExpression::TypeExpression {
                name: result_type_name,
                dependencies: result_type_deps,
            } = result_type;

            match fields.len().cmp(&constructor_args.len()) {
                // arity < args
                Ordering::Less => Err(ElaboratingError),
                // arity > args
                Ordering::Greater => Err(ElaboratingError),
                Ordering::Equal => {
                    let mut applied_constructor = constructor.clone();

                    for (p::definition::Definition { data, .. }, (var_name, ty)) in
                        zip(constructor_args, fields.iter())
                    {
                        let bindings = check(module_ctx, local_ctx, data, ty)?;

                        let data_value = elaborate_value(module_ctx, local_ctx, &data)?;
                        applied_constructor = application(&applied_constructor, data_value)?;
                        for (bind_name, bind_data) in bindings {
                            todo!("substitution of bindings")
                        }
                    }

                    let e::Constructor { result_type, .. } = constructor;
                    Ok(result_type.clone())
                }
            }
        }
        p::ExpressionNode::Variable { name } => {
            // Find in the local context, otherwise error
            local_ctx
                .get(name)
                .map(e::TypeExpression::clone)
                // Unknown variable
                .ok_or(ElaboratingError)
        }
        p::ExpressionNode::TypedHole => {
            // Type hole can't be inferred
            Err(ElaboratingError)
        }
    }
}
pub fn application<Str>(
    constructor: &e::Constructor<Str>,
    arg: e::ValueExpression<Str>,
) -> Result<e::Constructor<Str>, Error> {
    let e::Constructor {
        implicits,
        fields,
        result_type,
    } = constructor;

    todo!("Application is substitution + dependency reduction");
}

pub type Bindings<Str> = Vec<(Str, e::ValueExpression<Str>)>;

pub fn check<'a, Loc, Str: Debug + From<String> + Clone + Hash + Eq>(
    module_ctx: &e::Module<Str>,
    local_ctx: &'a Context<'a, Str, e::TypeExpression<Str>>,
    expression: &p::Expression<Loc, Str>,
    expected_type: &e::TypeExpression<Str>,
) -> Result<Bindings<Str>, Error> {
    match &expression.node {
        p::ExpressionNode::OpCall(_) => {
            todo!()
        }
        p::ExpressionNode::FunCall { .. } => {
            todo!()
        }
        p::ExpressionNode::ConstructorCall { .. } => {
            todo!()
        }
        p::ExpressionNode::Variable { .. } => {
            todo!()
        }
        p::ExpressionNode::TypedHole => {
            // Type hole is checked by any type without any bindings
            Ok(vec![])
        }
    }
}
