use core::panic;
use std::iter;

use crate::ast::elaborated::*;
use pretty::BoxDoc;

type Str = String; // Temporary
const NEST_UNIT: isize = 4;

#[derive(Clone)]
struct NamedConstructor<Name, Constructor> {
    name: Name,
    constructor: Constructor,
}

struct RustType(String);

// everything is needed to know about arbitrary field
struct Field {
    // name of the field in dependobuf
    name: Str,

    // dependobuf type
    ty: TypeExpression<Str>,

    // associated name when is part of some struct
    private_name: Str,

    // associated type when is part of some struct
    private_ty: RustType,
}

// TODO: change panics to regular error handling
// TODO: write adequate doc
// TODO: add rust pretty printer in order to simplify most of the generation here
// TODO: do something with lifetimes, they are shouldn't be needed

// This is sketch implementation.
// Final implementation will use own ast, because elaborated is not quite what codegen needs.
// For example types in elaborated module are ordered in topological order over dependencies
// however when generating, codegen is not enforced to place struct in such order.
// For now however order is now even a concern because everything is passed inside Boxes.
struct ModuleCodegen {
    module: Module<Str>,
}

impl ModuleCodegen {
    pub fn generate<'a>(&'a self) -> BoxDoc<'a> {
        let types = self
            .module
            .types
            .iter()
            .map(|(name, body)| self.generate_type(name, body));
        BoxDoc::text("type Box<T> = std::boxed::Box<T>;")
            .append(BoxDoc::line())
            .append(BoxDoc::intersperse(types, BoxDoc::line()))
    }

    fn generate_type<'a>(&'a self, name: &'a Str, body: &'a Type<Str>) -> BoxDoc<'a> {
        let prelude =
            BoxDoc::text("use super::{Box, ConstructorError, Message}").append(BoxDoc::line());

        let constructors = &self.module.constructors;
        let (constructors, kind) = match &body.constructor_names {
            ConstructorNames::OfMessage(constructor) => (
                vec![NamedConstructor {
                    name,
                    constructor: constructors
                        .get(constructor)
                        .expect("Codegen expects valid ast"),
                }],
                TypeKind::Message,
            ),
            ConstructorNames::OfEnum(branches) => (
                branches
                    .iter()
                    .map(|name| NamedConstructor {
                        name,
                        constructor: constructors.get(name).expect("Codegen expects valid ast"),
                    })
                    .collect(),
                TypeKind::Enum,
            ),
        };
        let type_codegen = TypeCodegen {
            codegen: self,
            name,
            dependencies: &body.dependencies,
            constructors,
            kind,
        };

        let module = BoxDoc::intersperse(
            [
                prelude,
                type_codegen.generate_definition(),
                type_codegen.generate_inherent_impl(),
            ],
            BoxDoc::line(),
        );

        BoxDoc::text(format!("mod {} {{", name))
            .append(BoxDoc::line())
            .append(module.nest(NEST_UNIT))
            .append(BoxDoc::text("}"))
    }

    fn generate_fields<'a>(&'a self, fields: &'a Context<Str>) -> BoxDoc<'a> {
        BoxDoc::intersperse(
            fields.iter().map(|(name, ty)| {
                BoxDoc::text(format!("{}: ", name)).append(self.generate_type_expression(ty))
            }),
            BoxDoc::text(",").append(BoxDoc::line()),
        )
    }

    fn generate_type_expression<'a>(&'a self, expression: &'a TypeExpression<Str>) -> BoxDoc<'a> {
        match expression {
            Expression::OpCall(_) => panic!("OpCall in type expression"),
            Expression::Type {
                name,
                dependencies: _, // dependencies are ignored here because they are runtime
            } => BoxDoc::text(format!("{}", name)),
            Expression::Constructor {
                name: _,
                implicits: _,
                arguments: _,
            } => panic!("Constructor in type expression"),
            Expression::Variable { name: _ } => panic!("Variable in type expression"),
        }
    }

    fn generate_value_expression<'a>(&'a self, _: &'a Expression<Str>) -> BoxDoc<'a> {
        todo!("value expression is not yet implemented")
    }
}

enum TypeKind {
    Message,
    Enum,
}

// Message and enum are both types (see ast::elaborated::Type),
// so they are both have name, dependencies and some (for message its always one) constructors
struct TypeCodegen<'a> {
    pub codegen: &'a ModuleCodegen,
    pub name: &'a Str,
    pub dependencies: &'a Context<Str>,
    pub constructors: Vec<NamedConstructor<&'a Str, &'a Constructor<Str>>>,
    pub kind: TypeKind,
}

impl<'a> TypeCodegen<'a> {
    // generates type definition
    pub fn generate_definition(&self) -> BoxDoc<'a> {
        let body = self.generate_body();
        let dependencies = BoxDoc::text("#[derive(PartialEq, Eq)]")
            .append(BoxDoc::line())
            .append(BoxDoc::text("struct Dependencies {"))
            .append(
                self.codegen
                    .generate_fields(self.dependencies)
                    .nest(NEST_UNIT),
            )
            .append(BoxDoc::text("}"))
            .append(BoxDoc::line());
        let name_alias = BoxDoc::text(format!(
            "pub type {} = Message<Body, Dependencies>;",
            self.name
        ))
        .append(BoxDoc::line());

        BoxDoc::intersperse([body, dependencies, name_alias], BoxDoc::line())
    }

    fn generate_body(&self) -> BoxDoc<'a> {
        let fields = match self.kind {
            TypeKind::Message => {
                debug_assert!(
                    self.constructors.len() == 1,
                    "Message expected to have only one constructor"
                );
                self.codegen
                    .generate_fields(&self.constructors[0].constructor.fields)
                    .nest(NEST_UNIT)
            }
            TypeKind::Enum => BoxDoc::intersperse(
                self.constructors
                    .iter()
                    .map(|NamedConstructor { name, constructor }| {
                        let fields = &constructor.fields;
                        if fields.is_empty() {
                            BoxDoc::nil()
                        } else {
                            BoxDoc::text(format!("{} {{", name))
                                .append(self.codegen.generate_fields(fields).nest(NEST_UNIT))
                                .append(BoxDoc::text("}"))
                        }
                    }),
                BoxDoc::text(",").append(BoxDoc::line()),
            ),
        };

        let holder = match self.kind {
            TypeKind::Message => "enum",
            TypeKind::Enum => "struct",
        };
        BoxDoc::text("#[derive(PartialEq, Eq)]")
            .append(BoxDoc::line())
            .append(BoxDoc::text(format!("pub {} {} {{", holder, self.name)))
            .append(BoxDoc::line())
            .append(fields.nest(NEST_UNIT))
            .append(BoxDoc::text("}"))
    }
}

impl<'a> TypeCodegen<'a> {
    // generates inherent impl that implements constructors for main type and setters and getters
    pub fn generate_inherent_impl(&self) -> BoxDoc<'a> {
        let constructors =
            self.constructors
                .iter()
                .map(|NamedConstructor { name, constructor }| {
                    Self::generate_constructor(name, constructor)
                });

        let setters = iter::once(BoxDoc::nil());

        let getters = iter::once(BoxDoc::nil());

        BoxDoc::intersperse(constructors.chain(setters).chain(getters), BoxDoc::line())
    }

    fn generate_constructor(name: &'a Str, constructor: &'a Constructor<Str>) -> BoxDoc<'a> {
        let _ = name;
        let _ = constructor;
        todo!()
    }

    // fn generate_setter()
}
