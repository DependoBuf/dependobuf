use super::{
    Constructor, Module, OpCall, Str, Symbol, Type, TypeExpression, TypeKind, UnaryOp,
    ValueExpression,
};
use crate::scope::Scope;
use std::rc::{Rc, Weak};

pub use dbuf_core::ast::{elaborated, operators};

// maybe temporary
#[allow(dead_code, reason = "maybe temporary")]
type ElaboratedType = elaborated::Type<Str>;
type ElaboratedValueExpression = elaborated::ValueExpression<Str>;
type ElaboratedTypeExpression = elaborated::TypeExpression<Str>;
type ElaboratedModule = elaborated::Module<Str>;
#[allow(dead_code, reason = "maybe temporary")]
type ElaboratedContext = elaborated::Context<Str>;
type ElaboratedConstructor = elaborated::Constructor<Str>;

// This implementation has one huge downside:
// It uses Rc and Weak in order to have references to associated objects smarter than strings (which also requires
// global hashmap). For example codegen::ast::Expression::Constructor stores weak reference to the corresponding
// codegen::ast::Constructor while ast::elaborated::Expression::Constructor stores just the name of the constructor
// which then must be located in constructor list
//
// Current implementation is not aware that whole ast has one lifetime, and any smart pointer deref should
// return one common lifetime.
//
// TODO: So at some point I want to rewrite this to custom indices wrapper that are aware of lifetime.
// Until that point I must clone in some places in order to please Rust gods.

#[derive(Clone, Copy)]
struct ASTContext<'a> {
    known_types: &'a Scope<'a, Str, Weak<Type>>,
    variables: &'a Scope<'a, Str, Rc<Symbol>>,
    constructors: &'a Scope<'a, Str, Rc<Constructor>>,
}

const BUILTIN_NAMES: &[&str] = &["Bool", "Int", "UInt", "String"];

impl Module {
    pub(crate) fn from_elaborated(module: &ElaboratedModule) -> Self {
        let mut all_constructors = Scope::<Str, Rc<Constructor>>::empty();
        let mut types = Vec::with_capacity(module.types.len());
        let mut known_types = Scope::<Str, Weak<Type>>::empty();

        let builtins: Vec<Rc<Type>> = BUILTIN_NAMES
            .iter()
            .map(|&name| {
                let ty = Rc::new(Type {
                    name: Str::from(name),
                    dependencies: vec![],
                    constructors: vec![],
                    kind: TypeKind::Enum,
                    is_builtin: true,
                });
                assert!(
                    known_types.try_insert(Str::from(name), Rc::downgrade(&ty)),
                    "builtin '{name}' inserted twice"
                );
                ty
            })
            .collect();

        for (name, ty) in &module.types {
            // all this scope constructs Rc<Type> and can be become Type::from_elaborated
            // but I do not really want this, because I do not feel like it will simplify logic
            let mut variables = Scope::<Str, Rc<Symbol>>::empty();

            let dependencies = ty
                .dependencies
                .iter()
                .map(|(name, expr)| {
                    let context = ASTContext {
                        known_types: &known_types,
                        variables: &variables,
                        constructors: &all_constructors,
                    };

                    let symbol = Rc::new(Symbol::from_elaborated(context, name.clone(), expr));
                    assert!(variables.try_insert(name.clone(), symbol.clone()));
                    symbol
                })
                .collect();

            let variables = variables;

            let ty = Rc::new_cyclic(|me| {
                use elaborated::ConstructorNames;
                assert!(
                    known_types.try_insert(name.clone(), me.clone()),
                    "codegen expects valid elaborated ast: two types can not have same name"
                );

                let (constructors, kind) = match &ty.constructor_names {
                    ConstructorNames::OfMessage(name) => (vec![name], TypeKind::Message),
                    ConstructorNames::OfEnum(constructors) => {
                        (constructors.iter().collect(), TypeKind::Enum)
                    }
                };

                let constructors = constructors
                    .into_iter()
                    .map(|constructor_name| {
                        let context = ASTContext {
                            known_types: &known_types,
                            variables: &variables,
                            constructors: &all_constructors,
                        };

                        let elaborated_constructor = module
                            .constructors
                            .get(constructor_name)
                            .expect("codegen expects valid elaborated ast: unknown constructor");

                        let constructor = Constructor::from_elaborated(context, constructor_name.clone(), elaborated_constructor, me.clone());

                        let constructor = Rc::new(constructor);
                        assert!(all_constructors.try_insert(constructor.name.clone(), constructor.clone()), "codegen expects valid elaborated ast: two constructors can not have same name");
                        constructor
                    })
                    .collect();

                Type {
                    name: name.clone(),
                    dependencies,
                    constructors,
                    kind,
                    is_builtin: false,
                }
            });

            types.push(ty);
        }
        Module {
            types,
            _builtins: builtins,
        }
    }
}

impl Constructor {
    fn from_elaborated(
        type_context: ASTContext<'_>,
        name: Str,
        ElaboratedConstructor {
            implicits,
            fields,
            result_type,
        }: &ElaboratedConstructor,
        this: Weak<Type>,
    ) -> Constructor {
        let mut all_params = Scope::nested_in(type_context.variables);

        let implicits: Vec<_> = implicits
            .iter()
            .map(|(name, expr)| {
                let partial_context = ASTContext {
                    known_types: type_context.known_types,
                    variables: &all_params,
                    constructors: type_context.constructors,
                };
                let symbol = Rc::new(Symbol::from_elaborated(partial_context, name.clone(), expr));
                assert!(all_params.try_insert(name.clone(), symbol.clone()), "codegen expects valid elaborated ast: two constructor constructor params (among dependencies and implicits) can not have same name");
                symbol
            })
            .collect();

        let fields: Vec<_> = fields
            .iter()
            .map(|(name, expr)| {
                let field_context = ASTContext {
                    known_types: type_context.known_types,
                    variables: &all_params,
                    constructors: type_context.constructors,
                };
                let symbol = Rc::new(Symbol::from_elaborated(field_context, name.clone(), expr));
                all_params.try_insert(name.clone(), symbol.clone());
                symbol
            })
            .collect();

        let constructor_context = ASTContext {
            known_types: type_context.known_types,
            variables: &all_params,
            constructors: type_context.constructors,
        };

        // this is if statement not needed now
        let result_type = match result_type {
            ElaboratedTypeExpression::TypeExpression {
                name: _,
                dependencies,
            } => {
                // unfortunately we can not verify that we constructing correct type here (even tho it's not codegen task)
                // because we operating on still dangling this
                TypeExpression::Type {
                    call: this,
                    dependencies: dependencies
                        .iter()
                        .map(|expr| ValueExpression::from_elaborated(constructor_context, expr))
                        .collect(),
                }
            }
        };

        Constructor {
            name,
            implicits,
            fields,
            result_type,
        }
    }
}

impl ValueExpression {
    fn from_elaborated(context: ASTContext<'_>, expr: &ElaboratedValueExpression) -> Self {
        match expr {
            ElaboratedValueExpression::OpCall {
                op_call,
                result_type: _,
            } => {
                let op_call = match op_call {
                    operators::OpCall::Literal(literal) => OpCall::Literal(literal.clone()),
                    // TODO: UnaryOp::Access must be Symbol, not string. But in order to locate this symbol I need to traverse
                    // message fields tree and find it. This can be done nicely when proper scope visibility determiner will be implemented
                    // for now tho this is NOT HUGE problem as fields mostly are generated quite trivially.
                    operators::OpCall::Unary(unary_op, expr) => {
                        let unary_op = match unary_op {
                            operators::UnaryOp::Access(name) => {
                                let ty = match expr.as_ref() {
                                    elaborated::ValueExpression::OpCall {
                                        op_call: _,
                                        result_type,
                                    } => result_type,
                                    elaborated::ValueExpression::Constructor {
                                        name: _,
                                        implicits: _,
                                        arguments: _,
                                        result_type,
                                    } => result_type,
                                    elaborated::ValueExpression::Variable { name: _, ty } => ty,
                                };
                                let ty = match ty {
                                    elaborated::TypeExpression::TypeExpression {
                                        name,
                                        dependencies: _,
                                    } => context
                                        .known_types
                                        .get(name)
                                        .expect("access to unknown type")
                                        // Access operator can only be used on message not on enums
                                        // There is no place in message constructor that could produce same message
                                        // So weak must always be safely upgradable
                                        .upgrade()
                                        .expect("access to unknown type"),
                                };

                                assert!(
                                    ty.constructors.len() == 1 && ty.kind == TypeKind::Message,
                                    "access to enum"
                                );
                                // this should be optimized
                                let field = ty.constructors[0]
                                    .fields
                                    .iter()
                                    .find(|field| field.name == *name)
                                    .expect("couldn't find field to access");
                                UnaryOp::Access {
                                    to: Rc::downgrade(&ty),
                                    field: Rc::downgrade(field),
                                }
                            }
                            operators::UnaryOp::Minus => UnaryOp::Minus,
                            operators::UnaryOp::Bang => UnaryOp::Bang,
                        };
                        OpCall::Unary(
                            unary_op,
                            Box::new(ValueExpression::from_elaborated(context, expr)),
                        )
                    }
                    operators::OpCall::Binary(binary_op, lhs, rhs) => OpCall::Binary(
                        *binary_op,
                        Box::new(Self::from_elaborated(context, lhs)),
                        Box::new(Self::from_elaborated(context, rhs)),
                    ),
                };
                ValueExpression::OpCall(op_call)
            }
            ElaboratedValueExpression::Constructor {
                name,
                implicits,
                arguments,
                result_type: _,
            } => {
                // because constructors can be encountered only in dependence substitution and for dependencies we already
                // verified that all types are valid then constructors of those types must also be valid
                let call = context
                    .constructors
                    .get(name)
                    .expect("codegen expects valid elaborated ast: call to unknown constructor");

                let implicits = implicits
                    .iter()
                    .map(|expr| ValueExpression::from_elaborated(context, expr))
                    .collect();

                let arguments = arguments
                    .iter()
                    .map(|expr| ValueExpression::from_elaborated(context, expr))
                    .collect();

                ValueExpression::Constructor {
                    call: Rc::downgrade(call),
                    implicits,
                    arguments,
                }
            }
            ElaboratedValueExpression::Variable { name, ty: _ } => {
                let symbol =
                    Rc::downgrade(context.variables.get(name).expect(
                        "codegen expects valid elaborated ast: non-introduced variable use",
                    ));
                ValueExpression::Variable(symbol)
            }
        }
    }
}

impl TypeExpression {
    fn from_elaborated(context: ASTContext<'_>, expr: &ElaboratedTypeExpression) -> Self {
        match expr {
            ElaboratedTypeExpression::TypeExpression { name, dependencies } => {
                // types in module must be in top sorted order (top sort over types and theirs dependencies)
                // we iterate over them in the same order
                // we can encounter type expression only in dependencies (either when calling or declaring)
                // in both cases top sort ensures following check
                let call = context.known_types.get(name).expect("codegen expects valid elaborated ast: expression contains call to unknown type");
                let dependencies = dependencies
                    .iter()
                    .map(|expr| ValueExpression::from_elaborated(context, expr))
                    .collect();

                TypeExpression::Type {
                    call: call.clone(),
                    dependencies,
                }
            }
        }
    }
}

impl Symbol {
    fn from_elaborated(
        context: ASTContext<'_>,
        name: Str,
        type_expr: &ElaboratedTypeExpression,
    ) -> Self {
        Symbol {
            name,
            ty: TypeExpression::from_elaborated(context, type_expr),
        }
    }
}
