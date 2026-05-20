use crate::ast::{elaborated as e, operators as o, parsed as p};
use crate::elaboration::*;
use crate::error::elaborating::Error::{
    self, ArityMismatch, OperatorTypeMismatch, TypeMismatch, UnknownConstructor, UnknownField,
    UnknownType, UnknownVariable, UnsupportedSyntax,
};

use std::collections::{BTreeMap, BTreeSet};
use std::iter::zip;

type DefRef<'a> = &'a p::definition::Definition<Loc, Name, p::TypeDeclaration<Loc, Name>>;

/// # Errors
pub(super) fn elaborate_sorted(module: &[DefRef<'_>]) -> Result<Mod, Error> {
    elaborate_with_module_ctx(builtins::builtins_module::<Str>(), module)
}

fn elaborate_with_module_ctx(mut module_ctx: Mod, module: &[DefRef<'_>]) -> Result<Mod, Error> {
    let mut elaborated_module = e::Module {
        types: vec![],
        constructors: BTreeMap::new(),
    };
    for &type_def in module {
        let (name, elaborated_type, elaborated_constructors) =
            elaborate_type_decl(&module_ctx, type_def)?;

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
    Ok(elaborated_module)
}

fn elaborate_type_decl(
    module_ctx: &Mod,
    type_def: &p::definition::Definition<Loc, Name, p::TypeDeclaration<Loc, Name>>,
) -> Result<ElaboratedDeclaration, Error> {
    let local_ctx = Ctx::new();

    let p::definition::Definition {
        name,
        data: p::TypeDeclaration { dependencies, body },
        ..
    } = type_def;

    let mut binding = local_ctx.new_layer();
    let (local_ctx_with_deps, elaborated_dependencies) =
        elaborate_deps(module_ctx, &mut binding, dependencies)?;
    let deps = rename::add_suffix_context(elaborated_dependencies.clone(), "_dep");

    match body {
        p::TypeDefinition::Message(ctor_body) => {
            let name_str = name.content.clone();
            let fields = elaborate_constructor_body(module_ctx, &local_ctx_with_deps, ctor_body)?;

            Ok((
                name_str.clone(),
                e::Type {
                    dependencies: deps,
                    constructor_names: e::ConstructorNames::OfMessage(name_str.clone()),
                },
                vec![(
                    name_str.clone(),
                    e::Constructor {
                        implicits: elaborated_dependencies.clone(),
                        fields,
                        result_type: TypeExpr::TypeExpression {
                            name: name_str,
                            dependencies: e::Rec::from(ctx_to_deps(&elaborated_dependencies)),
                        },
                    },
                )],
            ))
        }
        p::TypeDefinition::Enum(branches) => {
            let self_type = e::Module {
                types: vec![(
                    name.content.clone(),
                    e::Type {
                        dependencies: deps.clone(),
                        constructor_names: e::ConstructorNames::OfEnum(BTreeSet::new()),
                    },
                )],
                constructors: BTreeMap::new(),
            };
            let inductive_ctx = module_ctx.merge(self_type);
            let (constructor_names, constructors) = elaborate_enum(
                &inductive_ctx,
                &local_ctx_with_deps,
                name,
                branches,
                &elaborated_dependencies,
            )?;
            Ok((
                name.content.clone(),
                e::Type {
                    dependencies: deps,
                    constructor_names: e::ConstructorNames::OfEnum(constructor_names),
                },
                constructors,
            ))
        }
    }
}

fn elaborate_deps<'a>(
    module_ctx: &Mod,
    local_ctx: &'a mut Ctx<'a>,
    deps: &p::definition::Definitions<Loc, Name, p::TypeExpression<Loc, Name>>,
) -> Result<(Ctx<'a>, ElaboratedCtx), Error> {
    let mut dependencies = vec![];
    for p::definition::Definition {
        name,
        data: type_expr,
        ..
    } in deps
    {
        let elaborated_type_expr = elaborate_type(module_ctx, local_ctx, type_expr)?;

        local_ctx.insert(name.content.clone(), elaborated_type_expr.clone());
        dependencies.push((name.content.clone(), elaborated_type_expr));
    }
    Ok((local_ctx.new_layer(), dependencies))
}

fn elaborate_constructor_body<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    ctor_body: &p::ConstructorBody<Loc, Name>,
) -> Result<ElaboratedCtx, Error> {
    let mut fields: ElaboratedCtx = Vec::new();
    let mut field_ctx = local_ctx.new_layer();

    for p::definition::Definition { name, data, .. } in ctor_body {
        let elaborated_type = elaborate_type(module_ctx, &field_ctx, data)?;
        field_ctx.insert(name.content.clone(), elaborated_type.clone());
        fields.push((name.content.clone(), elaborated_type));
    }

    Ok(fields)
}

fn elaborate_type<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    expr: &p::TypeExpression<Loc, Name>,
) -> Result<TypeExpr, Error> {
    match &expr.node {
        p::ExpressionNode::OpCall(_) => Err(UnsupportedSyntax),
        p::ExpressionNode::FunCall { fun, args } => {
            let (_, ty) = module_ctx
                .types
                .iter()
                .find(|(name, _)| *name == fun.content)
                .ok_or_else(|| UnknownType(fun.content.to_string()))?;

            let declared_deps = ty.dependencies.clone();

            if declared_deps.len() != args.len() {
                return Err(ArityMismatch {
                    expected: declared_deps.len(),
                    found: args.len(),
                });
            }

            let mut remaining_deps = declared_deps;
            let mut elaborated_args = Vec::new();

            for arg in args.iter() {
                let (dep_name, dep_type) = remaining_deps.remove(0);

                let (arg_value, bindings) = check(module_ctx, local_ctx, arg, &dep_type)?;
                elaborated_args.push(arg_value.clone());

                for (_, remaining_ty) in &mut remaining_deps {
                    let after_name = subst::subst_type(remaining_ty.clone(), &dep_name, &arg_value);
                    *remaining_ty = subst::apply_bindings_to_type(after_name, &bindings);
                }
            }

            Ok(TypeExpr::TypeExpression {
                name: fun.content.clone(),
                dependencies: e::Rec::from(elaborated_args),
            })
        }
        p::ExpressionNode::ConstructorCall { .. } => Err(UnsupportedSyntax),
        p::ExpressionNode::Variable { name } => local_ctx
            .get(&name.content)
            .cloned()
            .ok_or_else(|| UnknownVariable(name.content.to_string())),
        p::ExpressionNode::TypedHole => Err(UnsupportedSyntax),
    }
}

type EnumSpecification = (BTreeSet<Str>, Vec<(Str, e::Constructor<Str>)>);

fn elaborate_enum<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    name: &Name,
    branches: &[p::EnumBranch<Loc, Name>],
    enum_deps: &ElaboratedCtx,
) -> Result<EnumSpecification, Error> {
    let result_type = TypeExpr::TypeExpression {
        name: name.content.clone(),
        dependencies: e::Rec::from(ctx_to_deps(enum_deps)),
    };

    let mut constructor_names = vec![];
    let mut constructors = vec![];

    for branch in branches {
        let branch_ctors =
            elaborate_branch(module_ctx, local_ctx, branch, enum_deps, &result_type)?;

        for (ctor_name, ctor) in branch_ctors {
            constructor_names.push(ctor_name.clone());
            constructors.push((ctor_name, ctor));
        }
    }

    Ok((constructor_names.into_iter().collect(), constructors))
}

fn elaborate_branch<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    branch: &p::EnumBranch<Loc, Name>,
    enum_deps: &ElaboratedCtx,
    result_type: &TypeExpr,
) -> Result<Vec<(Str, e::Constructor<Str>)>, Error> {
    if branch.patterns.len() != enum_deps.len() {
        return Err(ArityMismatch {
            expected: enum_deps.len(),
            found: branch.patterns.len(),
        });
    }

    let mut branch_ctx = local_ctx.new_layer();
    let mut bindings = Vec::new();

    for (pattern, (dep_name, dep_type)) in zip(&branch.patterns, enum_deps) {
        let concrete_dep_type = subst::apply_bindings_to_type(dep_type.clone(), &bindings);
        let new_bindings = elaborate_pattern(
            module_ctx,
            pattern,
            dep_name,
            &concrete_dep_type,
            &mut branch_ctx,
        )?;
        bindings.extend(new_bindings);
    }

    let result_type = subst::apply_bindings_to_type(result_type.clone(), &bindings);

    let mut implicits = Vec::new();
    for (dep_name, dep_type) in enum_deps {
        if !bindings.iter().any(|(n, _)| n == dep_name) {
            let concrete = subst::apply_bindings_to_type(dep_type.clone(), &bindings);
            implicits.push((dep_name.clone(), concrete));
        }
    }

    let mut extra: Vec<_> = branch_ctx.terms.iter().collect();
    extra.sort_by_key(|&(k, _)| k);
    implicits.extend(extra.into_iter().map(|(k, v)| (k.clone(), v.clone())));

    let mut ctors = Vec::new();
    for p::definition::Definition {
        name: ctor_name,
        data: ctor_body,
        ..
    } in &branch.constructors
    {
        let fields = elaborate_constructor_body(module_ctx, &branch_ctx, ctor_body)?;
        ctors.push((
            ctor_name.content.clone(),
            e::Constructor {
                implicits: implicits.clone(),
                fields,
                result_type: result_type.clone(),
            },
        ));
    }
    Ok(ctors)
}

fn elaborate_pattern(
    module_ctx: &Mod,
    pattern: &p::Pattern<Loc, Name>,
    dep_name: &Str,
    dep_type: &TypeExpr,
    branch_ctx: &mut Ctx<'_>,
) -> Result<Bindings, Error> {
    match &pattern.node {
        p::PatternNode::ConstructorCall {
            name: ctor_name,
            fields: sub_patterns,
        } => {
            let ctor = module_ctx
                .constructors
                .get(&ctor_name.content)
                .ok_or_else(|| UnknownConstructor(ctor_name.content.to_string()))?
                .clone();

            if sub_patterns.len() != ctor.fields.len() {
                return Err(ArityMismatch {
                    expected: ctor.fields.len(),
                    found: sub_patterns.len(),
                });
            }

            let (implicit_values, argument_values) = elaborate_pattern_constructor(
                module_ctx,
                &ctor,
                sub_patterns,
                dep_type,
                branch_ctx,
            )?;
            let ctor_value = Value::Constructor {
                name: ctor_name.content.clone(),
                implicits: e::Rec::from(implicit_values),
                arguments: e::Rec::from(argument_values),
                result_type: dep_type.clone(),
            };
            branch_ctx.insert_alias(dep_name.clone(), ctor_value.clone());
            Ok(vec![(dep_name.clone(), ctor_value)])
        }
        p::PatternNode::Variable { name } => {
            let value = Value::Variable {
                name: name.content.clone(),
                ty: dep_type.clone(),
            };
            branch_ctx.insert(name.content.clone(), dep_type.clone());
            branch_ctx.insert_alias(dep_name.clone(), value.clone());
            Ok(vec![(dep_name.clone(), value)])
        }
        p::PatternNode::Literal(literal) => {
            operators::check_literal(literal, dep_type)?;
            let value = operators::make_lit(literal.clone(), dep_type.clone());
            branch_ctx.insert_alias(dep_name.clone(), value.clone());
            Ok(vec![(dep_name.clone(), value)])
        }
        p::PatternNode::Underscore => Ok(vec![]),
    }
}

fn elaborate_pattern_constructor(
    module_ctx: &Mod,
    ctor: &e::Constructor<Str>,
    sub_patterns: &p::definition::Definitions<Loc, Name, p::Pattern<Loc, Name>>,
    dep_type: &TypeExpr,
    branch_ctx: &mut Ctx<'_>,
) -> Result<(Vec<Value>, Vec<Value>), Error> {
    let TypeExpr::TypeExpression {
        dependencies: dep_deps,
        ..
    } = dep_type;
    let TypeExpr::TypeExpression {
        dependencies: ctor_deps,
        ..
    } = &ctor.result_type;

    let mut implicit_bindings = Vec::new();
    for (dep_td, ctor_rd) in zip(dep_deps.iter(), ctor_deps.iter()) {
        let bindings = unify::unify_value(dep_td, ctor_rd)?;
        implicit_bindings.extend(bindings);
    }

    let mut argument_values = Vec::new();
    let mut field_bindings = implicit_bindings.clone();

    for (sub_def, (field_name, field_type)) in zip(sub_patterns, &ctor.fields) {
        let concrete_field_type =
            subst::apply_bindings_to_type(field_type.clone(), &field_bindings);

        let (arg, new_bindings) = if let p::PatternNode::Underscore = &sub_def.data.node {
            let fresh = Str::from(format!("_{}", field_name.as_ref()));
            branch_ctx.insert(fresh.clone(), concrete_field_type.clone());
            (
                Value::Variable {
                    name: fresh,
                    ty: concrete_field_type,
                },
                vec![],
            )
        } else {
            let subterm_bindings = elaborate_pattern(
                module_ctx,
                &sub_def.data,
                field_name,
                &concrete_field_type,
                branch_ctx,
            )?;
            let arg = subterm_bindings
                .iter()
                .find(|(n, _)| n == field_name)
                .map(|(_, v)| v.clone())
                .ok_or_else(|| UnknownField(field_name.as_ref().to_string()))?;
            (arg, subterm_bindings)
        };

        field_bindings.extend(new_bindings);
        argument_values.push(arg);
    }

    let implicit_values: Vec<Value> = ctor
        .implicits
        .iter()
        .map(|(name_impl, ty_impl)| {
            implicit_bindings
                .iter()
                .find(|(n, _)| n == name_impl)
                .map(|(_, v)| v.clone())
                .unwrap_or(Value::Variable {
                    name: name_impl.clone(),
                    ty: ty_impl.clone(),
                })
        })
        .collect();
    Ok((implicit_values, argument_values))
}

fn infer<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    expr: &p::Expression<Loc, Name>,
) -> Result<Value, Error> {
    match &expr.node {
        p::ExpressionNode::OpCall(op_call) => infer_operator(module_ctx, local_ctx, op_call),
        p::ExpressionNode::FunCall { .. } => Err(UnsupportedSyntax),
        p::ExpressionNode::ConstructorCall {
            name,
            fields: constructor_args,
        } => infer_constructor_call(module_ctx, local_ctx, name, constructor_args),
        p::ExpressionNode::Variable { name } => {
            if let Some(alias) = local_ctx.get_alias(&name.content) {
                return Ok(alias.clone());
            }
            let ty = local_ctx
                .get(&name.content)
                .cloned()
                .ok_or_else(|| UnknownVariable(name.content.to_string()))?;
            Ok(Value::Variable {
                name: name.content.clone(),
                ty,
            })
        }
        p::ExpressionNode::TypedHole => Err(UnsupportedSyntax),
    }
}

fn infer_operator<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    op_expr: &o::OpCall<Name, e::Rec<p::Expression<Loc, Name>>>,
) -> Result<Value, Error> {
    let value = match op_expr {
        o::OpCall::Literal(literal) => {
            operators::make_lit(literal.clone(), operators::literal_to_type(literal))
        }
        o::OpCall::Unary(o::UnaryOp::Access(field), value) => {
            if let p::ExpressionNode::ConstructorCall {
                name: ctor_name,
                fields: ctor_args,
            } = &value.node
            {
                let ctor = module_ctx
                    .constructors
                    .get(&ctor_name.content)
                    .ok_or_else(|| UnknownConstructor(ctor_name.content.to_string()))?;
                let field_idx = ctor
                    .fields
                    .iter()
                    .position(|(n, _)| *n == field.content)
                    .ok_or_else(|| UnknownField(field.content.to_string()))?;
                let field_arg = ctor_args
                    .get(field_idx)
                    .ok_or_else(|| UnknownField(field.content.to_string()))?;
                return infer(module_ctx, local_ctx, &field_arg.data);
            }

            let elaborated_expr = infer(module_ctx, local_ctx, value)?;
            let operand_type = type_of(&elaborated_expr);
            let (_, _, concrete_field_type) =
                operators::resolve_field_access(module_ctx, &operand_type, &field.content)?;

            operators::make_unary(
                o::UnaryOp::Access(field.content.clone()),
                elaborated_expr,
                concrete_field_type,
            )
        }
        o::OpCall::Unary(op @ (o::UnaryOp::Minus | o::UnaryOp::Bang), value) => {
            let accepted_types = operators::unary_accepted_types(op);
            let Some((elaborated_val, ty)) = accepted_types.iter().find_map(|&x| {
                let builtin = builtins::get_builtin(&x);
                let (v, bindings) = check(module_ctx, local_ctx, value, &builtin).ok()?;
                debug_assert_eq!(bindings.len(), 0);
                Some((v, builtin))
            }) else {
                return Err(OperatorTypeMismatch);
            };
            operators::make_unary(op.into(), elaborated_val, ty)
        }
        o::OpCall::Binary(op, l, r) => {
            let binary_op = |accepted_types: &[builtins::BuiltinType]| -> Result<Value, Error> {
                let Some((builtin, left, right)) = accepted_types.iter().find_map(|x| {
                    let builtin = builtins::get_builtin(x);
                    let (left, bindings) = check(module_ctx, local_ctx, l, &builtin).ok()?;
                    debug_assert_eq!(bindings.len(), 0);

                    let (right, bindings) = check(module_ctx, local_ctx, r, &builtin).ok()?;
                    debug_assert_eq!(bindings.len(), 0);
                    Some((builtin, left, right))
                }) else {
                    return Err(OperatorTypeMismatch);
                };
                Ok(operators::make_binary(*op, left, right, builtin))
            };
            binary_op(operators::binary_accepted_types(op))?
        }
    };
    Ok(normalize::simplify(&value))
}

fn infer_constructor_call<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    name: &Name,
    constructor_args: &p::definition::Definitions<Loc, Name, p::Expression<Loc, Name>>,
) -> Result<Value, Error> {
    let constructor = module_ctx
        .constructors
        .get(&name.content)
        .ok_or_else(|| UnknownConstructor(name.content.to_string()))?;

    let e::Constructor { fields, .. } = constructor;

    if fields.len() != constructor_args.len() {
        return Err(ArityMismatch {
            expected: fields.len(),
            found: constructor_args.len(),
        });
    }
    let mut applied_constructor = constructor.clone();
    let mut arguments_expression = Vec::new();
    let mut all_implicit_bindings = Vec::new();
    for (p::definition::Definition { data, .. }, (_var_name, ty)) in
        zip(constructor_args, fields.iter())
    {
        let (data_value, check_bindings) = check(module_ctx, local_ctx, data, ty)?;

        all_implicit_bindings.extend(check_bindings);

        let new_ctor = apply::application(&applied_constructor, &data_value, module_ctx)?;
        arguments_expression.push(data_value.clone());

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
                .unwrap_or(Value::Variable {
                    name: implicit_name,
                    ty: implicit_ty,
                })
        })
        .collect::<Vec<_>>();

    Ok(Value::Constructor {
        name: name.content.clone(),
        implicits: e::Rec::from(resolved_implicits),
        arguments: e::Rec::from(arguments_expression),
        result_type,
    })
}

fn check<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    expression: &p::Expression<Loc, Name>,
    expected_type: &TypeExpr,
) -> Result<(Value, Bindings), Error> {
    match &expression.node {
        p::ExpressionNode::OpCall(op) => match op {
            o::OpCall::Literal(literal) => {
                operators::check_literal(literal, expected_type)?;
                return Ok((
                    operators::make_lit(literal.clone(), expected_type.clone()),
                    vec![],
                ));
            }
            o::OpCall::Unary(op @ (o::UnaryOp::Minus | o::UnaryOp::Bang), arg) => {
                let accepted = operators::unary_accepted_types(op);
                if !accepted
                    .iter()
                    .any(|t| *expected_type == builtins::get_builtin(t))
                {
                    return Err(OperatorTypeMismatch);
                }
                let (checked_arg, bindings) = check(module_ctx, local_ctx, arg, expected_type)?;
                debug_assert_eq!(bindings.len(), 0);
                let value = operators::make_unary(op.into(), checked_arg, expected_type.clone());
                return Ok((normalize::simplify(&value), vec![]));
            }
            o::OpCall::Binary(op, l, r) => {
                let accepted = operators::binary_accepted_types(op);
                if !accepted
                    .iter()
                    .any(|t| *expected_type == builtins::get_builtin(t))
                {
                    return Err(OperatorTypeMismatch);
                }
                let (checked_l, bindings_l) = check(module_ctx, local_ctx, l, expected_type)?;
                debug_assert_eq!(bindings_l.len(), 0);
                let (checked_r, bindings_r) = check(module_ctx, local_ctx, r, expected_type)?;
                debug_assert_eq!(bindings_r.len(), 0);
                let value =
                    operators::make_binary(*op, checked_l, checked_r, expected_type.clone());
                return Ok((normalize::simplify(&value), vec![]));
            }
            o::OpCall::Unary(..) => {}
        },

        p::ExpressionNode::ConstructorCall { name, fields } => {
            return check_constructor_call(module_ctx, local_ctx, name, fields, expected_type);
        }
        p::ExpressionNode::TypedHole => return Err(Error::TypeHole(expected_type.clone())),
        _ => {}
    }
    let elaborated = infer(module_ctx, local_ctx, expression)?;
    let inferred_type = type_of(&elaborated);

    let right_bindings = unify::unify_type(&inferred_type, expected_type, module_ctx)?;
    Ok((elaborated, right_bindings))
}

fn check_constructor_call<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    name: &Name,
    constructor_args: &p::definition::Definitions<Loc, Name, p::Expression<Loc, Name>>,
    expected_type: &TypeExpr,
) -> Result<(Value, Bindings), Error> {
    let elaborated = infer_constructor_call(module_ctx, local_ctx, name, constructor_args)?;

    let TypeExpr::TypeExpression {
        name: inferred_name,
        dependencies: inferred_deps,
    } = &type_of(&elaborated);

    let TypeExpr::TypeExpression {
        name: expected_name,
        dependencies: expected_deps,
    } = expected_type;

    if inferred_name != expected_name || inferred_deps.len() != expected_deps.len() {
        return Err(TypeMismatch);
    }

    let mut left_bindings = vec![];
    let mut right_bindings = vec![];

    for (inferred_dep, expected_dep) in zip(inferred_deps.iter(), expected_deps.iter()) {
        match inferred_dep {
            Value::Variable { name, .. } => {
                left_bindings.push((name.clone(), expected_dep.clone()));
            }
            concrete => {
                let rb = unify::unify_value(concrete, expected_dep)?;
                right_bindings.extend(rb);
            }
        }
    }

    Ok((
        subst::apply_bindings(elaborated, &left_bindings),
        right_bindings,
    ))
}

fn ctx_to_deps(elaborated_ctx: &ElaboratedCtx) -> Vec<Value> {
    elaborated_ctx
        .iter()
        .cloned()
        .map(|(name, ty)| Value::Variable { name, ty })
        .collect()
}

/// # Errors
fn check_constructor_call<'a>(
    module_ctx: &Mod,
    local_ctx: &'a Ctx<'a>,
    name: &Name,
    constructor_args: &p::definition::Definitions<Loc, Name, p::Expression<Loc, Name>>,
    expected_type: &TypeExpr,
) -> Result<(Value, Bindings), Error> {
    let elaborated = infer_constructor_call(module_ctx, local_ctx, name, constructor_args)?;

    let TypeExpr::TypeExpression {
        name: inferred_name,
        dependencies: inferred_deps,
    } = &type_of(&elaborated);

    let TypeExpr::TypeExpression {
        name: expected_name,
        dependencies: expected_deps,
    } = expected_type;

    if inferred_name != expected_name || inferred_deps.len() != expected_deps.len() {
        return Err(ElaboratingError);
    }

    let mut left_bindings: Bindings = vec![];
    let mut right_bindings: Bindings = vec![];

    for (inferred_dep, expected_dep) in zip(inferred_deps.iter(), expected_deps.iter()) {
        match inferred_dep {
            Value::Variable { name, .. } => {
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

fn ctx_to_deps(elaborated_ctx: &ElaboratedCtx) -> Vec<Value> {
    elaborated_ctx
        .iter()
        .cloned()
        .map(|(name, ty)| Value::Variable { name, ty })
        .collect::<Vec<_>>()
}
