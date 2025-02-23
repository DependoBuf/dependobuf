use std::iter;

use super::{
    ast::{self, Constructor, Expression, Module, Symbol, Type},
    scope::{Scope, TaggerScope},    
    BoxAllocator, BoxDoc, DocAllocator, NEST_UNIT,
};

// TODO: make string here and in ast stored in arena and remove cloning

// TODO: crate Codegen trait in order to generate helper functions. For example:
// suppose that we have implemented
// TypeDefinitionCodegen::generate(ty: &Type, context: CodegenContext<'a>, scope: &NamingScope<'_>) -> BoxDoc<'a>
// then it can automatically generate
// TypeDefinitionCodegen::generate(iter: impl Iterator<Item = &Type>, context: CodegenContext<'a>, scope: &NamingScope<'_>) -> impl Iterator<Item = BoxDoc<'a>
// saving tedious maps that I must write for now

#[derive(Clone, Copy)]
struct CodegenContext<'a> {
    alloc: &'a BoxAllocator,
}

type NamingScope<'a> = TaggerScope<'a, Identifier>;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Tag(u32);

impl Tag {
    pub fn inc(self) -> Self {
        Tag(self.0 + 1)
    }
}

impl Default for Tag {
    fn default() -> Self {
        Tag(0)
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum IdentifierKind {
    Module,
    Function,
    Variable,
    Type,
}

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct Identifier {
    // String here is not optimal. Something like Vec<Word> seems more appropriate.
    // Were Word insures that string in lowered so it can be easily formatted to camelCase or snake_case.
    name: String,
    kind: IdentifierKind,
}

impl<'a> Identifier {
    pub fn module(name: String) -> Self {
        Identifier {
            name,
            kind: IdentifierKind::Module,
        }
    }

    pub fn function(name: String) -> Self {
        Identifier {
            name,
            kind: IdentifierKind::Function,
        }
    }

    pub fn variable(name: String) -> Self {
        Identifier {
            name,
            kind: IdentifierKind::Variable,
        }
    }

    pub fn ty(name: String) -> Self {
        Identifier {
            name,
            kind: IdentifierKind::Type,
        }
    }

    pub fn try_generate(
        self,
        context: CodegenContext<'a>,
        scope: &mut NamingScope<'_>,
    ) -> Option<BoxDoc<'a>> {
        if scope.try_insert(self.clone()) {
            Some(self.format(Tag::default(), context.alloc))
        } else {
            None
        }
    }

    pub fn generate(self, context: CodegenContext<'a>, scope: &mut NamingScope<'_>) -> BoxDoc<'a> {
        let tag = scope.insert(self.clone());
        self.format(tag, context.alloc)
    }

    // TODO: do kind specific formatting (needs conversion of vec of words into camelCase, snake_case, etc.)
    fn format(self, tag: Tag, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
        if tag.0 == 0 {
            alloc.text(self.name).into_doc()
        } else {
            alloc
                .text(self.name)
                .append("_")
                .append(tag.0.to_string())
                .into_doc()
        }
    }
}

impl<'a> Module {
    pub fn generate(self, context: CodegenContext<'a>, scope: &mut NamingScope<'_>) -> BoxDoc<'a> {
        let alloc = context.alloc;
        let aliases = alloc
            .text("mod aliases {")
            .append(alloc.intersperse(
                [alloc.text("type Box<T> = std::boxed::Box<T>;")],
                alloc.line(),
            ))
            .append("}")
            .append(alloc.line())
            .into_doc();

        let types = self
            .types
            .into_iter()
            .map(|ty| (*ty).clone().generate(context, scope))
            .collect::<Vec<_>>();

        aliases.append(alloc.intersperse(types, alloc.line()))
    }
}

impl<'a> Type {
    pub fn generate(self, context: CodegenContext<'a>, scope: &mut NamingScope<'_>) -> BoxDoc<'a> {
        let prelude =
            BoxDoc::text("use super::{Box, ConstructorError, Message}").append(BoxDoc::line());

        let module = BoxDoc::intersperse(
            [
                prelude,
                self.generate_dependencies_import(context, scope),
                self.generate_declaration(context, scope),
                self.generate_inherent_impl(context, scope),
            ],
            BoxDoc::line(),
        );

        BoxDoc::text(format!("mod {} {{", self.name))
            .append(BoxDoc::line())
            .append(module.nest(NEST_UNIT))
            .append(BoxDoc::text("}"))
    }
}

mod type_dependencies_import {
    use super::{BoxDoc, CodegenContext, DocAllocator, NamingScope, NEST_UNIT};
    use super::{Constructor, Expression, Identifier, Symbol, Type};
    use std::{collections::hash_set::HashSet, iter, rc::Rc};

    impl<'a> Type {
        pub(super) fn generate_dependencies_import(
            &self,
            context: CodegenContext<'a>,
            scope: &mut NamingScope<'_>,
        ) -> BoxDoc<'a> {
            let deps = Identifier::module("deps".to_owned())
                .try_generate(context, scope)
                .expect("couldn't create 'deps' module");
            let mut deps_scope = NamingScope::nested_in(&scope);
            let scope = &mut deps_scope;
            let alloc = context.alloc;

            let mut dependencies: HashSet<_> = self
                .constructors
                .iter()
                .map(|constructor| {
                    let Constructor {
                        name: _,
                        implicits,
                        fields,
                        result_type,
                    } = (*constructor.upgrade().expect("missing type constructor")).clone();
                    let implicits = implicits.iter().map(Self::symbol_dependencies);
                    let fields = fields.iter().map(Self::symbol_dependencies);
                    iter::once(Self::expression_dependencies(&result_type))
                        .chain(implicits)
                        .chain(fields)
                        .into_iter()
                        .flatten()
                        .collect()
                })
                .chain(self.dependencies.iter().map(Self::symbol_dependencies))
                .into_iter()
                .flatten()
                .collect();

            dependencies.remove(&*self.name); // don't need to return self name

            let helpers_deps = alloc
                .text("use super::{Box, ConstructorError, Message};")
                .into_doc();

            let other_type_deps = alloc
                .text("use super::super::{")
                .append(alloc.intersperse(
                    dependencies.into_iter().map(|name| {
                        let ty = Identifier::ty(name.clone())
                            .try_generate(context, scope)
                            .expect("couldn't import dependence");
                        let module = Identifier::module(name.clone())
                            .try_generate(context, scope)
                            .expect("couldn't import dependence");
                        alloc
                            .text("{")
                            .append(ty)
                            .append(", ")
                            .append(module)
                            .append("}")
                    }),
                    alloc.text(", "),
                ))
                .append("};")
                .append(alloc.line())
                .append(";")
                .into_doc();

            alloc
                .text("mod ")
                .append(deps)
                .append(" {")
                .append(
                    alloc
                        .intersperse([helpers_deps, other_type_deps], alloc.line())
                        .nest(NEST_UNIT),
                )
                .append("}")
                .into_doc()
        }

        fn expression_dependencies(expr: &Expression) -> HashSet<String> {
            let results = match expr {
                Expression::OpCall(op_call) => {
                    use crate::ast::operators::OpCall;
                    match op_call {
                        OpCall::Literal(_) => vec![], // no deps in literal
                        OpCall::Unary(_, expr) => vec![Self::expression_dependencies(expr)],
                        OpCall::Binary(_, lhs, rhs) => {
                            vec![
                                Self::expression_dependencies(lhs),
                                Self::expression_dependencies(rhs),
                            ]
                        }
                    }
                }
                Expression::Type { call, dependencies } => {
                    let ty = (*call.upgrade().expect("missing type in expression")).clone();
                    let name_deps = if Self::is_primitive_type(&ty.name) {
                        iter::empty().collect()
                    } else {
                        iter::once(ty.name).collect()
                    };
                    iter::once(name_deps)
                        .chain(dependencies.into_iter().map(Self::expression_dependencies))
                        .collect()
                }
                Expression::Constructor {
                    call: _, // dependencies of called constructor should not be matter
                    implicits,
                    arguments,
                } => implicits
                    .into_iter()
                    .map(Self::expression_dependencies)
                    .chain(arguments.into_iter().map(Self::expression_dependencies))
                    .collect(),
                Expression::Variable(symbol) => {
                    vec![Self::symbol_dependencies(
                        &symbol.upgrade().expect("missing type in symbol"),
                    )] // this is maybe unnecessary
                }
            };
            results.into_iter().flatten().collect()
        }

        fn symbol_dependencies(symbol: &Rc<Symbol>) -> HashSet<String> {
            Self::expression_dependencies(&(**symbol).clone().ty)
        }

        // TODO: move this to more appropriate place
        fn is_primitive_type(name: &str) -> bool {
            match name {
                "Bool" => true,
                "Double" => true,
                "Int" => true,
                "UInt" => true,
                "String" => true,
                _ => false,
            }
        }
    }
}

mod type_declaration {
    use super::{ast, Identifier, Symbol, Type};
    use super::{BoxDoc, CodegenContext, DocAllocator, NamingScope, NEST_UNIT};
    use std::rc::Rc;

    impl<'a> Type {
        pub(super) fn generate_declaration(
            &self,
            context: CodegenContext<'a>,
            scope: &mut NamingScope<'_>,
        ) -> BoxDoc<'a> {
            let alloc = context.alloc;

            let body = self.generate_body(context, scope);
            let dependencies = self.generate_body(context, scope);

            let name_alias = alloc
                .text("pub type ")
                .append(Identifier::ty(self.name.clone()).generate(context, scope))
                // deps module provided by type_dependencies_import
                .append(" = deps::Message<Body, Dependencies>;")
                .append(BoxDoc::line())
                .into_doc();

            alloc
                .intersperse([body, dependencies, name_alias], alloc.line())
                .into_doc()
        }

        fn generate_body(
            &self,
            context: CodegenContext<'a>,
            scope: &mut NamingScope<'_>,
        ) -> BoxDoc<'a> {
            let alloc = context.alloc;

            let body = Identifier::ty("body".to_owned())
                .try_generate(context, scope)
                .expect("failed to declare Body");

            // in future, as I implement Codegen::generate as trait, those helper function can be generated automatically
            let generate_fields = |fields: &Vec<Rc<Symbol>>, scope: &mut NamingScope<'_>| {
                fields
                    .iter()
                    .map(|symbol| {
                        let symbol = (**symbol).clone();
                        symbol.generate_declaration(context, scope)
                    })
                    .collect::<Vec<_>>()
            };

            let fields =
                match self.kind {
                    ast::TypeKind::Message => {
                        debug_assert!(
                            self.constructors.len() == 1,
                            "Message expected to have only one constructor"
                        );
                        let constructor = self.constructors[0]
                            .upgrade()
                            .expect("missing type constructor");
                        alloc.intersperse(generate_fields(&constructor.fields, scope), alloc.line())
                    }
                    ast::TypeKind::Enum => {
                        let mut branch_scope = NamingScope::nested_in(&scope);
                        let scope = &mut branch_scope;
                        alloc.intersperse(
                            self.constructors
                                .iter()
                                .map(|constructor| {
                                    let constructor =
                                        &*constructor.upgrade().expect("missing type constructor");
                                    let fields = generate_fields(&constructor.fields, scope);
                                    // TODO: that's not really accurate, enum variant do not match type behavior in
                                    // scopes, because it will not shadow names in the scope where enum is declared
                                    let name = Identifier::ty(constructor.name.clone())
                                        .try_generate(context, scope)
                                        .expect("failed to declare constructor (enum variant)");
                                    name.append(" {")
                                        .append(alloc.intersperse(
                                            fields,
                                            alloc.text(",").append(alloc.line()),
                                        ))
                                        .append("}")
                                })
                                .collect::<Vec<_>>(),
                            alloc.text(",").append(alloc.line()),
                        )
                    }
                };

            let holder = match self.kind {
                ast::TypeKind::Message => "enum",
                ast::TypeKind::Enum => "struct",
            };
            alloc
                .text("#[derive(PartialEq, Eq)]")
                .append(alloc.line())
                .append(format!("pub {} ", holder))
                .append(body)
                .append(" {")
                .append(alloc.line())
                .append(fields.nest(NEST_UNIT))
                .append(alloc.text("}"))
                .append(alloc.line())
                .into_doc()
        }

        fn generate_dependencies(
            &self,
            context: CodegenContext<'a>,
            scope: &mut NamingScope<'_>,
        ) -> BoxDoc<'a> {
            let alloc = context.alloc;

            let dependencies = Identifier::ty("Dependencies".to_owned())
                .try_generate(context, scope)
                .expect("failed to define Dependence");

            alloc
                .text("#[derive(PartialEq, Eq)]")
                .append(alloc.line())
                .append("struct ")
                .append(dependencies)
                .append(" {")
                .append(
                    alloc.intersperse(
                        self.dependencies
                            .iter()
                            .map(|symbol| {
                                let symbol = (**symbol).clone();
                                symbol.generate_declaration(context, scope)
                            })
                            .collect::<Vec<_>>(),
                        alloc.line(),
                    ),
                )
                .nest(NEST_UNIT)
                .append("}")
                .append(alloc.line())
                .into_doc()
        }
    }
}

// WIP
// Current problem: NamingScope has no way of returning tag that he give to some Value afterwards.
// This really limits its usability in the face of many variables. For example with enum branching in dependobuf it is really useful.
// 
// Also, current implementation of tags and identifiers seems to miss its goals, because when constructing identifier from Symbol,
// you nearly lose track of what was that symbol. So this maybe needs to be rethinked.
//
// Possible solution could be: add variant Symbol (which is special variable) to the Identifier enum 

mod type_inherent_impl {
    use super::{ast, Expression, Constructor, Identifier, Symbol, Type};
    use super::{BoxDoc, CodegenContext, DocAllocator, NamingScope, NEST_UNIT};
    use std::{iter, rc::Rc};

    impl<'a> Type {
        pub(super) fn generate_inherent_impl(
            &self,
            context: CodegenContext<'a>,
            scope: &mut NamingScope<'_>,
        ) -> BoxDoc<'a> {
            let constructors = self.constructors.iter().map(|constructor| {
                let constructor = &*constructor.upgrade().expect("missing type constructor");
                constructor.generate_constructor(context, scope)    
            });

            let setters = iter::once(BoxDoc::nil());

            let getters = iter::once(BoxDoc::nil());

            BoxDoc::intersperse(constructors.chain(setters).chain(getters), BoxDoc::line())
        }
    }

    impl<'a> Constructor {
        pub fn generate_constructor(
            &self,
            context: CodegenContext<'a>,
            scope: &mut NamingScope<'_>,
        ) -> BoxDoc<'a> {
            let constructor = Identifier::function(self.name.clone())
                .try_generate(context, scope)
                .expect("failed to name constructor");

            let constructor_scope = NamingScope::nested_in(scope);
            let scope = &mut constructor_scope;

            // order matters because there can be field name "dependencies" and I want to preserve params names
            let params = self.fields.iter().map(|symbol| {
                Identifier::variable(symbol.name)
                    .try_generate(context, scope)
                    .expect("failed to name constructor param")
            });
            let dependencies_param =
                Identifier::variable("dependencies".to_owned()).generate(context, scope);
            let params = iter::once(dependencies_param)
                .chain(params)
                .collect::<Vec<_>>();

            let body = self.generate_constructor_body(context, scope);

            BoxDoc::text(format!("pub fn {}(", constructor.name))
                .append(BoxDoc::intersperse(
                    params,
                        BoxDoc::text(name).append(BoxDoc::text(": ")).append(ty)),
                    BoxDoc::text(", "),
                ))
                .append(BoxDoc::text(") -> Result<Self, ConstructorError> {"))
                .append(BoxDoc::line())
                .append(body)
                .append(BoxDoc::text("}"))
        }

        fn generate_constructor_body(
            &self,
            context: CodegenContext<'a>,
            scope: &mut NamingScope<'_>,
        ) -> BoxDoc<'a> {
            let alloc = context.alloc;

            let body_var = Identifier::variable("body".to_owned()).generate(context, scope);

            alloc.text("let ").append(body_var).into_doc()
        }
    }

    impl<'a> Expression {
        pub fn generate_as_pattern(&self, context: CodegenContext<'a>, scope: &mut NamingScope<'_>) -> BoxDoc<'a> {
            todo!()
        }
    }
}

impl<'a> Symbol {
    pub fn generate_declaration(
        &self,
        context: CodegenContext<'a>,
        scope: &mut NamingScope<'_>,
    ) -> BoxDoc<'a> {
        todo!()
    }
}

impl<'a> Expression {
    pub fn generate_type_expression(
        &self,
        context: CodegenContext<'a>,
        scope: &mut NamingScope<'_>,
    ) -> BoxDoc<'a> {
        todo!()
    }
}
