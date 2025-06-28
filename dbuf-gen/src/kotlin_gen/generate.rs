use pretty::{BoxAllocator, BoxDoc, Doc, DocAllocator};

use crate::ast::{Module, Type, TypeKind};

mod kotlin {
    use pretty::{BoxAllocator, BoxDoc, DocAllocator, DocBuilder};

    use crate::{ast, format};

    pub struct Param(ast::Symbol);

    /// Kotlin's sealed class resembles enum in rust
    pub struct SealedClass {
        pub name: String,
        pub params: Vec<Param>,
        pub constructors: Vec<InnerClass>,
    }

    /// Inner class of a sealed class resembles enum constructor in rust
    pub struct InnerClass {
        pub name: String,
    }

    impl SealedClass {
        pub fn generate<'a>(&self, alloc: &'a BoxAllocator) -> BoxDoc<'a> {
            let build_param_declarations = |params: &Vec<Param>| {
                alloc.concat(params.iter().map(|param| {
                    alloc
                        .text("val")
                        .append(alloc.space())
                        .append(param.generate(alloc))
                        .append(";")
                        .append(alloc.hardline())
                }))
            };

            let build_constructor = |params: &Vec<Param>| {
                let constructor_params = alloc.intersperse(
                    params.iter().map(|param| param.generate(alloc)),
                    alloc.text(", "),
                );

                alloc
                    .text("private constructor")
                    .append(constructor_params.parens())
                    .append(alloc.space())
                    .append((|| {
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

                        let assignments = alloc.intersperse(
                            params.iter().map(|param| build_assignment(&param.0.name)),
                            alloc.hardline(),
                        );

                        alloc
                            .hardline()
                            .append(assignments)
                            .append(alloc.hardline())
                            .append("// constructor body")
                            .append(alloc.hardline())
                            .nest(format::NEST_UNIT)
                            .braces()
                    })())
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

            let build_class_body = |param_declarations, constructor, inner_classes| {
                alloc.concat(vec![param_declarations, constructor, inner_classes])
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
                    build_param_declarations(&self.params),
                    build_constructor(&self.params),
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

            let build_constructor = || {
                alloc
                    .text("constructor")
                    .append(alloc.text("").parens())
                    .append(":")
                    .append(alloc.space())
                    .append("super")
                    .append(alloc.text("").parens())
                    .append(alloc.space())
                    .append(
                        alloc
                            .hardline()
                            .append("// inner class constructor")
                            .append(alloc.hardline())
                            .braces(),
                    )
            };

            build_class(&self.name, build_constructor()).into_doc()
        }
    }

    impl Param {
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
        params: t
            .dependencies
            .iter()
            .map(|dep| kotlin::Param::new(dep))
            .collect(),
        constructors: t
            .constructors
            .iter()
            .map(|constructor| kotlin::InnerClass {
                name: constructor.name.clone(),
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
