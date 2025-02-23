use crate::ast::{elaborated, operators::OpCall};
use super::scope::Scope;
use std::rc::{Rc, Weak};

type Str = String; // Temporary

// maybe temporary
type ElaboratedType = elaborated::Type<Str>;
type ElaboratedExpression = elaborated::Expression<Str>;
type ElaboratedModule = elaborated::Module<Str>;
type ElaboratedContext = elaborated::Context<Str>;
type ElaboratedConstructor = elaborated::Constructor<Str>;

// This implementation has one huge downside: 
// It uses Rc and Weak in order to have references to associated objects smarter then strings (which also requires 
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
    types: &'a Scope<'a, String, Rc<Type>>,
    variables: &'a Scope<'a, String, Rc<Symbol>>,
    constructors: &'a Scope<'a, String, Rc<Constructor>>,
}

pub struct Module {
    pub types: Vec<Rc<Type>>,
}

#[derive(Clone)]
pub enum TypeKind {
    Message,
    Enum,
}

#[derive(Clone)]
pub struct Type {
    pub name: Str,
    pub dependencies: Vec<Rc<Symbol>>,
    pub constructors: Vec<Weak<Constructor>>,
    pub kind: TypeKind,
}

#[derive(Clone)]
pub struct Constructor {
    pub name: Str,
    pub implicits: Vec<Rc<Symbol>>,
    pub fields: Vec<Rc<Symbol>>,
    pub result_type: TypeExpression,
}

#[derive(Clone)]
pub enum Expression {
    OpCall(OpCall<Str, Box<Expression>>),
    Type {
        call: Weak<Type>,
        dependencies: Vec<Expression>,
    },
    Constructor {
        call: Weak<Constructor>,
        implicits: Vec<Expression>,
        arguments: Vec<Expression>,
    },
    Variable(Weak<Symbol>),
}

type TypeExpression = Expression;

#[derive(Clone)]
pub struct Symbol {
    pub name: Str,
    pub ty: TypeExpression,
}

impl Module {
    pub fn from_elaborated(mut module: ElaboratedModule) -> Self {
        let mut all_constructors = Scope::<String, Rc<Constructor>>::empty();
        let mut types = Scope::<String, Rc<Type>>::empty();
        for (name, ty) in module.types.into_iter() {
            // all this scope constructs Rc<Type> and can be become Type::from_elaborated
            // but I do not really want this, because I do not feel like it will simplify logic
            let mut variables = Scope::<String, Rc<Symbol>>::empty();

            let dependencies = ty
                .dependencies
                .into_iter()
                .map(|(name, expr)| {
                    let context = ASTContext {
                        types: &types,
                        variables: &variables,
                        constructors: &all_constructors,
                    };

                    let symbol = Rc::new(Symbol::from_elaborated(context, name.clone(), expr));
                    assert!(variables.try_insert(name, symbol.clone()));
                    symbol
                })
                .collect();

            let variables = variables;

            let ty = Rc::new_cyclic(|me| {
                use elaborated::ConstructorNames;
                let (constructors, kind) = match ty.constructor_names {
                    ConstructorNames::OfMessage(name) => (vec![name], TypeKind::Message),
                    ConstructorNames::OfEnum(constructors) => {
                        (constructors.into_iter().collect(), TypeKind::Enum)
                    }
                };

                let constructors = constructors
                    .into_iter()
                    .map(|constructor_name| {
                        let context = ASTContext {
                            types: &types,
                            variables: &variables,
                            constructors: &all_constructors,
                        };

                        let elaborated_constructor = module
                            .constructors
                            // removing here because each constructor is unique to the type and to the whole module
                            .remove(&constructor_name)
                            .expect("codegen expects valid elaborated ast: unknown constructor");

                        let constructor = Constructor::from_elaborated(context, constructor_name, elaborated_constructor, me.clone());

                        let constructor = Rc::new(constructor);
                        assert!(all_constructors.try_insert(constructor.name.clone(), constructor.clone()), "codegen expects valid elaborated ast: two constructors can not have same name");
                        Rc::downgrade(&constructor)
                    })
                    .collect();

                Type {
                    name,
                    dependencies,
                    constructors,
                    kind,
                }
            });

            assert!(
                types.try_insert(ty.name.clone(), ty),
                "codegen expects valid elaborated ast: two types can not have same name"
            );
        }
        Module {
            types: types.flat_into_iter().map(|(_, value)| value).collect(),
        }
    }
}

impl Constructor {
    pub fn from_elaborated<'a>(
        type_context: ASTContext<'a>,
        name: Str,
        ElaboratedConstructor {
            implicits,
            fields,
            result_type,
        }: ElaboratedConstructor,
        this: Weak<Type>,
    ) -> Constructor {
        let mut all_params = Scope::nested_in(type_context.variables);

        let implicits = implicits
            .into_iter()
            .map(|(name, expr)| {
                let symbol = Rc::new(Symbol::from_elaborated(type_context, name.clone(), expr));
                assert!(all_params.try_insert(name, symbol.clone()), "codegen expects valid elaborated ast: two constructor constructor params (among dependencies and implicits) can not have same name");
                symbol
            })
            .collect();

        let all_params = all_params;

        let constructor_context = ASTContext {
            types: type_context.types,
            variables: &all_params,
            constructors: type_context.constructors,
        };

        let fields = fields
            .into_iter()
            .map(|(name, expr)| Rc::new(Symbol::from_elaborated(constructor_context, name, expr)))
            .collect();

        let result_type = match result_type {
            ElaboratedExpression::Type {
                name: _,
                dependencies,
            } => {
                // unfortunately we can not verify that we constructing correct type here (even tho it's not codegen task)
                // because we operating on still dangling this
                Expression::Type {
                    call: this,
                    dependencies: dependencies
                        .iter()
                        .cloned()
                        .map(|expr| Expression::from_elaborated(constructor_context, expr))
                        .collect(),
                }
            }
            _ => panic!(
                "codegen expects valid elaborated ast: constructor result type is not call to type"
            ),
        };

        Constructor {
            name,
            implicits,
            fields,
            result_type,
        }
    }
}

impl Expression {
    fn from_elaborated<'a>(context: ASTContext<'a>, expr: ElaboratedExpression) -> Self {
        match expr {
            ElaboratedExpression::OpCall(op_call) => {
                let op_call = match op_call {
                    OpCall::Literal(literal) => OpCall::Literal(literal),
                    OpCall::Unary(unary_op, expr) => OpCall::Unary(
                        unary_op,
                        Box::new(Self::from_elaborated(context, (*expr).clone())),
                    ),
                    OpCall::Binary(binary_op, lhs, rhs) => OpCall::Binary(
                        binary_op,
                        Box::new(Self::from_elaborated(context, (*lhs).clone())),
                        Box::new(Self::from_elaborated(context, (*rhs).clone())),
                    ),
                };
                Expression::OpCall(op_call)
            }
            ElaboratedExpression::Type { name, dependencies } => {
                // types in module must be in top sorted order (top sort over types and theirs dependencies)
                // we iterate over them in the same order
                // we can encounter type expression only in dependencies (either when calling or declaring)
                // in both cases top sort ensures following check
                let call = context.types.get(&name).expect("codegen expects valid elaborated ast: expression contains call to unknown type");
                let dependencies = dependencies
                    .iter()
                    .cloned()
                    .map(|expr| Expression::from_elaborated(context, expr))
                    .collect();

                Expression::Type {
                    call: Rc::downgrade(call),
                    dependencies,
                }
            }
            ElaboratedExpression::Constructor {
                name,
                implicits,
                arguments,
            } => {
                // because constructors can be encountered only in dependence substitution and for dependencies we already
                // verified that all types are valid then constructors of those types must also be valid
                let call = context
                    .constructors
                    .get(&name)
                    .expect("codegen expects valid elaborated ast: call to unknown constructor");

                let implicits = implicits
                    .iter()
                    .cloned()
                    .map(|expr| Expression::from_elaborated(context, expr))
                    .collect();

                let arguments = arguments
                    .iter()
                    .cloned()
                    .map(|expr| Expression::from_elaborated(context, expr))
                    .collect();

                Expression::Constructor {
                    call: Rc::downgrade(call),
                    implicits,
                    arguments,
                }
            }
            ElaboratedExpression::Variable { name } => {
                let symbol =
                    Rc::downgrade(context.variables.get(&name).expect(
                        "codegen expects valid elaborated ast: non-introduced variable use",
                    ));
                Expression::Variable(symbol)
            }
        }
    }
}

impl Symbol {
    fn from_elaborated<'a>(
        context: ASTContext<'a>,
        name: Str,
        type_expr: ElaboratedExpression,
    ) -> Self {
        match type_expr {
            ElaboratedExpression::Type {
                name: _,
                dependencies: _,
            } => {
                let ty = Expression::from_elaborated(context, type_expr);
                Symbol { name, ty }
            }
            _ => panic!("tried to construct symbol with type expression that is not type"),
        }
    }
}
