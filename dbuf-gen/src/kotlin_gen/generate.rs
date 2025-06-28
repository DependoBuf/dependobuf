use pretty::{BoxAllocator, BoxDoc, Doc, DocAllocator};

use crate::ast::{Module, Symbol, Type, TypeKind};

mod kotlin {
    use pretty::{BoxAllocator, BoxDoc, DocAllocator, DocBuilder};

    use crate::{ast, format};

    pub struct Field(ast::Symbol);

    /// Kotlin's sealed class resembles enum in rust
    pub struct SealedClass {
        pub name: String,
        pub fields: Vec<Field>,
        pub constructors: Vec<InnerClass>,
    }

    /// Inner class of a sealed class resembles enum constructor in rust
    pub struct InnerClass {
        pub name: String,
        pub fields: Vec<Field>,
        pub result_type: ast::TypeExpression,
    }

    impl SealedClass {
        pub fn generate<'a>(&self, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
            let build_field_declarations = |fields: &Vec<Field>| {
                alloc.concat(fields.iter().map(|field| {
                    alloc
                        .text("val")
                        .append(alloc.space())
                        .append(field.generate(alloc))
                        .append(";")
                        .append(alloc.hardline())
                }))
            };

            let build_constructor = |fields: &Vec<Field>| {
                let constructor_params = alloc.intersperse(
                    fields.iter().map(|field| field.generate(alloc)),
                    alloc.text(", "),
                );

                let assignments = (|| {
                    let build_assignment = |name: &String| {
                        alloc
                            .text("this.")
                            .append(name.clone())
                            .append(alloc.space())
                            .append("=")
                            .append(alloc.space())
                            .append(name.clone())
                            .append(";")
                    };
                    alloc.intersperse(
                        fields.iter().map(|field| build_assignment(&field.0.name)),
                        alloc.hardline(),
                    )
                })();

                alloc
                    .text("private constructor")
                    .append(constructor_params.parens())
                    .append(alloc.space())
                    .append(
                        alloc
                            .hardline()
                            .append("// constructor asserts")
                            .append(alloc.hardline())
                            .append(assignments)
                            .append(alloc.hardline())
                            .nest(format::NEST_UNIT)
                            .braces(),
                    )
                    .append(alloc.hardline())
            };

            let build_inner_classes = |constructors: &Vec<InnerClass>| {
                alloc.intersperse(
                    constructors
                        .iter()
                        .map(|inner_class| inner_class.generate(&self.name, alloc)),
                    alloc.hardline(),
                )
            };

            let build_class_body = |field_declarations, constructor, inner_classes| {
                alloc.concat(vec![field_declarations, constructor, inner_classes])
            };

            let build_class = |name: &String, body| {
                alloc
                    .text("sealed class")
                    .append(alloc.space())
                    .append(name.clone())
                    .append(alloc.space())
                    .append(
                        alloc
                            .hardline()
                            .append(body)
                            .nest(format::NEST_UNIT)
                            .append(alloc.hardline())
                            .braces(),
                    )
            };

            build_class(
                &self.name,
                build_class_body(
                    build_field_declarations(&self.fields),
                    build_constructor(&self.fields),
                    build_inner_classes(&self.constructors),
                ),
            )
            .into_doc()
        }
    }

    impl InnerClass {
        pub fn generate<'a>(&self, parent_name: &String, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
            let build_class = |name: &String, body| {
                alloc
                    .text("class")
                    .append(alloc.space())
                    .append(name.clone())
                    .append(":")
                    .append(alloc.space())
                    .append(parent_name.clone())
                    .append(alloc.space())
                    .append(
                        alloc
                            .hardline()
                            .append(body)
                            .nest(format::NEST_UNIT)
                            .append(alloc.hardline())
                            .braces(),
                    )
            };

            let build_constructor = |fields: &Vec<Field>, result_type: &ast::TypeExpression| {
                let constructor_params = alloc.intersperse(
                    fields.iter().map(|field| field.generate(alloc)),
                    alloc.text(", "),
                );

                let parent_params = alloc.intersperse(vec![alloc.space()], alloc.text(", "));

                let assignments = (|| {
                    let build_assignment = |name: &String| {
                        alloc
                            .text("this.")
                            .append(name.clone())
                            .append(alloc.space())
                            .append("=")
                            .append(alloc.space())
                            .append(name.clone())
                            .append(";")
                    };

                    alloc.intersperse(
                        fields.iter().map(|field| build_assignment(&field.0.name)),
                        alloc.hardline(),
                    )
                })();

                alloc
                    .text("constructor")
                    .append(constructor_params.parens())
                    .append(":")
                    .append(alloc.space())
                    .append("super")
                    .append(parent_params.parens())
                    .append(alloc.space())
                    .append(
                        alloc
                            .hardline()
                            .append("// inner class asserts")
                            .append(alloc.hardline())
                            .append(assignments)
                            .append(alloc.hardline())
                            .nest(format::NEST_UNIT)
                            .braces(),
                    )
            };

            build_class(
                &self.name,
                build_constructor(&self.fields, &self.result_type),
            )
            .into_doc()
        }
    }

    impl Field {
        pub fn new(symbol: &ast::Symbol) -> Self {
            Self(symbol.clone())
        }
        pub fn generate<'a>(&self, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
            alloc
                .text(self.0.name.clone())
                .append(":")
                .append(alloc.space())
                .append(self.0.ty.get_type().name.clone())
                .into_doc()
        }
    }
}

fn generate_class<'a>(t: &Type, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
    if t.kind == TypeKind::Message {
        assert!(t.constructors.len() == 1);
    }
    let class = kotlin::SealedClass {
        name: t.name.clone(),
        fields: t
            .dependencies
            .iter()
            .map(|dep| kotlin::Field::new(dep))
            .collect(),
        constructors: t
            .constructors
            .iter()
            .map(|constructor| kotlin::InnerClass {
                name: constructor.name.clone(),
                fields: constructor
                    .fields
                    .iter()
                    .map(|field| kotlin::Field::new(field))
                    .collect(),
                result_type: constructor.result_type.clone(),
            })
            .collect(),
    };

    class.generate(alloc)
}

pub fn generate_module<'a>(module: Module) -> String {
    let alloc = &BoxAllocator;
    let mut writer = Vec::new();
    for t in &module.types {
        generate_class(t, alloc)
            .append(alloc.hardline())
            .render(40, &mut writer)
            .expect("To be ok");
    }

    String::from_utf8(writer).expect("generated code must be correct utf8")
}
