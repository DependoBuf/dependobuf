use std::rc::Rc;

use super::prelude::*;

// TODO: make string here and in ast stored in arena and remove cloning

// TODO: introduce Codegen trait in order to generate helper functions. For example:
// suppose that we have implemented
// TypeDefinitionCodegen::generate(ty: &Type, context: CodegenContext<'a>, scope: &NamingScope<'_>) -> BoxDoc<'a>
// then it can automatically generate
// TypeDefinitionCodegen::generate(iter: impl Iterator<Item = &Type>, context: CodegenContext<'a>, scope: &NamingScope<'_>) -> impl Iterator<Item = BoxDoc<'a>
// saving tedious maps that I must write for now

impl<'a> Module {
    pub(super) fn generate(&self, (ctx, namespace): MutContext<'a, '_, '_>) -> BoxDoc<'a> {
        let alloc = ctx.alloc;

        let types = self
            .types
            .iter()
            .map(|ty| (*ty).clone().generate((ctx, namespace)))
            .collect::<Vec<_>>();

        alloc
            .intersperse(
                [
                    "use dbuf_rust_runtime::{Box, ConstructorError, DeserializeError};",
                    "use std::io::{Write, Read, Error};",
                    "use std::slice;",
                ],
                alloc.hardline(),
            )
            .append(alloc.hardline())
            .append(alloc.intersperse(types, alloc.hardline()))
            .into_doc()
    }
}

impl<'a> Type {
    pub fn generate(&self, (ctx, namespace): MutContext<'a, '_, '_>) -> BoxDoc<'a> {
        let alloc = ctx.alloc;

        let (type_module, mut type_namespace) = namespace
            .insert_object_preserve_name(objects::Module::from_object(
                ObjectId(NodeId::id(self), Tag::String("module")),
                self.name.to_lowercase().clone(),
            ))
            .expect("couldn't generate type module");

        let mut module_parts = Vec::new();
        module_parts.push(self.generate_dependencies_import((ctx, &mut type_namespace)));
        if let Some(descriptor_module) =
            self.generate_enum_descriptor_module((ctx, &mut type_namespace))
        {
            module_parts.push(descriptor_module);
        }
        module_parts.push(self.generate_declaration((ctx, &mut type_namespace)));
        module_parts.push(self.generate_inherent_impl((ctx, &mut type_namespace)));

        let module = alloc.intersperse(module_parts, alloc.hardline());

        drop(type_namespace);

        // That is needed workaround so further written applies could modify this.
        // TODO: In future need to write custom builder that will modify it self in-place instead of returning new.
        // Another solution could be use crate replace_with (https://docs.rs/replace_with).
        let mut message_type_path: Option<DocBuilder<_>> = Some(alloc.text(""));
        namespace
            .cursor()
            .lookup_generated::<objects::Module>(ObjectId(NodeId::id(self), Tag::String("module")))
            .expect("couldn't get generated type module")
            .apply(|_, object| {
                // cast here is unnecessary, make GetGenerated preserve type that is was getting in the returned cursor
                message_type_path = Some(
                    message_type_path.take().unwrap().append(
                        objects::GeneratedModule::try_from(object)
                            .expect("expected module")
                            .to_doc(ctx),
                    ),
                );
            })
            .lookup_generated::<objects::Type>(ObjectId(NodeId::id(self), Tag::String("type")))
            .expect("couldn't get generated message type")
            .apply(|_, object| {
                message_type_path = Some(
                    message_type_path.take().unwrap().append("::").append(
                        objects::GeneratedType::try_from(object)
                            .expect("expected type")
                            .to_doc(ctx),
                    ),
                );
            });

        let (use_alias_name, _) = namespace.insert_object_auto_name(objects::Type::from_object(
            ObjectId(NodeId::id(self), Tag::String("type")),
            self.name.clone(),
        ));

        alloc
            .text("pub mod")
            .append(alloc.space())
            .append(type_module.to_doc(ctx))
            .append(alloc.space())
            .append("{")
            .append(
                alloc
                    .hardline()
                    .append(module)
                    .nest(NEST_UNIT)
                    .append(alloc.hardline()),
            )
            .append("}")
            .append(alloc.hardline())
            .append(alloc.hardline())
            .append("pub use")
            .append(alloc.space())
            .append(message_type_path)
            .append(alloc.space())
            .append("as")
            .append(alloc.space())
            .append(use_alias_name.to_doc(ctx))
            .append(";")
            .append(alloc.hardline())
            .into_doc()
    }
}

mod type_dependencies_import {
    use super::super::prelude::*;

    use std::{collections::HashSet, iter, rc::Rc};

    impl<'a> Type {
        #[allow(clippy::too_many_lines, reason = "??? (144/100)")]
        pub(super) fn generate_dependencies_import(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let (deps_module, mut deps_namespace) = namespace
                .insert_object_preserve_name(objects::Module::from_name("deps".to_owned()))
                .expect("couldn't create 'deps' module");

            let namespace = &mut deps_namespace;
            let alloc = ctx.alloc;

            let mut dependencies = self
                .constructors
                .iter()
                .map(|constructor| {
                    let Constructor {
                        name: _,
                        implicits,
                        fields,
                        result_type,
                    } = constructor.as_ref();
                    let implicits = implicits.iter().map(Self::symbol_dependencies);
                    let fields = fields.iter().map(Self::symbol_dependencies);
                    iter::once(Self::type_expression_dependencies(result_type))
                        .chain(implicits)
                        .chain(fields)
                        .flatten()
                        .collect()
                })
                .chain(self.dependencies.iter().map(Self::symbol_dependencies))
                .flatten()
                .collect::<HashSet<_>>();

            dependencies.remove(&NodeId::id(self)); // don't need to return self name

            let other_type_deps = if dependencies.is_empty() {
                alloc.text("// ")
            } else {
                alloc.nil()
            };

            let other_type_deps = other_type_deps
                .append(alloc.text("pub(super) use"))
                .append(alloc.space())
                .append("super::super::")
                .append("{")
                .append(alloc.intersperse(
                    dependencies.into_iter().map(|node_id| {
                        let global_namespace = namespace
                            .cursor()
                            .lookup_module_root()
                            .go_back()
                            .expect("deps module expected to be non-root")
                            .lookup_module_root()
                            .go_back()
                            .expect("message module expected to be non-root");

                        let message_module = global_namespace
                            .clone()
                            .lookup_generated::<objects::Module>(ObjectId(
                                node_id.clone(),
                                Tag::String("module"),
                            ))
                            .expect("couldn't get message module");

                        let message_type = global_namespace
                            .lookup_generated::<objects::Type>(ObjectId(
                                node_id.clone(),
                                Tag::String("type"),
                            ))
                            .expect("couldn't get message type");

                        let message_module_generated =
                            objects::GeneratedModule::try_from(message_module.value())
                                .expect("expected module");
                        let message_type_generated =
                            objects::GeneratedType::try_from(message_type.value())
                                .expect("expected type");

                        // TODO: those should not be copies. References are enough, because they are non-modifiable parts of tree.
                        let message_module_node = message_module.node().clone();
                        let message_type_node = message_type.node().clone();
                        drop(message_module);
                        drop(message_type);

                        namespace.insert_tree(
                            &ObjectId(node_id.clone(), Tag::String("module")),
                            message_module_node,
                        );
                        namespace.insert_tree(
                            &ObjectId(node_id.clone(), Tag::String("type")),
                            message_type_node,
                        );

                        alloc
                            .text("{")
                            .append(message_module_generated.to_doc(ctx))
                            .append(",")
                            .append(alloc.space())
                            .append(message_type_generated.to_doc(ctx))
                            .append("}")
                    }),
                    alloc.text(",").append(alloc.space()),
                ))
                .append("};");

            alloc
                .text("mod ")
                .append(deps_module.to_doc(ctx))
                .append(" {")
                .append(
                    alloc
                        .hardline()
                        .append(alloc.intersperse([other_type_deps], alloc.hardline()))
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }

        fn type_expression_dependencies(expr: &TypeExpression) -> HashSet<NodeId<'a>> {
            match expr {
                TypeExpression::Type { call, dependencies } => {
                    let ty = call.upgrade().expect("missing type in expression");
                    let name_deps = if Self::is_primitive_type(&ty.name) {
                        iter::empty().collect()
                    } else {
                        iter::once(NodeId::id_weak(call)).collect()
                    };
                    iter::once(name_deps)
                        .chain(dependencies.iter().map(Self::value_expression_dependencies))
                        .flatten()
                        .collect()
                }
            }
        }

        fn value_expression_dependencies(expr: &ValueExpression) -> HashSet<NodeId<'a>> {
            let results = match expr {
                ValueExpression::OpCall(op_call) => {
                    match op_call {
                        OpCall::Literal(_) => vec![], // no deps in literal
                        OpCall::Unary(_, expr) => vec![Self::value_expression_dependencies(expr)],
                        OpCall::Binary(_, lhs, rhs) => {
                            vec![
                                Self::value_expression_dependencies(lhs),
                                Self::value_expression_dependencies(rhs),
                            ]
                        }
                    }
                }
                ValueExpression::Constructor {
                    call: _, // dependencies in called constructor should not matter
                    implicits,
                    arguments,
                } => implicits
                    .iter()
                    .map(Self::value_expression_dependencies)
                    .chain(arguments.iter().map(Self::value_expression_dependencies))
                    .collect(),
                ValueExpression::Variable(symbol) => {
                    vec![Self::symbol_dependencies(
                        &symbol.upgrade().expect("missing type in symbol"),
                    )] // this is maybe unnecessary
                }
            };
            results.into_iter().flatten().collect()
        }

        fn symbol_dependencies(symbol: &Rc<Symbol>) -> HashSet<NodeId<'a>> {
            Self::type_expression_dependencies(&symbol.ty)
        }

        // TODO: move this to more appropriate place
        fn is_primitive_type(name: &str) -> bool {
            matches!(name, "Bool" | "Double" | "Int" | "UInt" | "String")
        }
    }
}

mod enum_descriptor_mod {
    use super::super::prelude::*;

    impl<'a> Type {
        pub(super) fn generate_enum_descriptor_module(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> Option<BoxDoc<'a>> {
            let alloc = ctx.alloc;

            if matches!(self.kind, ast::TypeKind::Message) {
                return None;
            }

            let (descriptor_module, mut descriptor_module_namepspace) = namespace
                .insert_object_preserve_name(objects::Module::from_object(
                    objects::ObjectId(
                        ast::NodeId::owned("descriptor".to_owned()),
                        objects::Tag::String("module"),
                    ),
                    "descriptor".to_owned(),
                ))
                .expect("couldn't insert descriptor module");

            Some(
                alloc
                    .text("mod")
                    .append(alloc.space())
                    .append(descriptor_module.to_doc(ctx))
                    .append(alloc.space())
                    .append("{")
                    .append(
                        alloc
                            .hardline()
                            .append(
                                alloc.intersperse(
                                    self.constructors.iter().enumerate().map(
                                        |(index, constructor)| {
                                            let (construct_descriptor_name, _) =
                                                descriptor_module_namepspace
                                                    .insert_object_preserve_name(
                                                        objects::Variable::from_name(
                                                            constructor.name.clone(),
                                                        ),
                                                    )
                                                    .expect("couldn't insert descriptor variable");
                                            alloc
                                                .text("pub(super)")
                                                .append(alloc.space())
                                                .append("const")
                                                .append(alloc.space())
                                                .append(construct_descriptor_name.to_doc(ctx))
                                                .append(":")
                                                .append(alloc.space())
                                                .append("u8")
                                                .append(alloc.space())
                                                .append("=")
                                                .append(alloc.space())
                                                .append(index.to_string())
                                                .append(";")
                                        },
                                    ),
                                    alloc.hardline(),
                                ),
                            )
                            .nest(NEST_UNIT)
                            .append(alloc.hardline()),
                    )
                    .append("}")
                    .into_doc(),
            )
        }
    }

    impl<'a> Constructor {
        pub(super) fn generate_enum_descriptor<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
        ) -> Option<BoxDoc<'a>> {
            let ty = self.result_type.get_type();

            if matches!(ty.kind, ast::TypeKind::Message) {
                return None;
            }

            let (descriptor_module, descriptor_cursor) = namespace
                .get_generated::<objects::Module>(
                    objects::ObjectId::from_name("descriptor".to_owned())
                        .with_tag(objects::Tag::String("module")),
                )
                .expect("couldn't get generated descriptor module");

            let (descriptor_variable, _) = descriptor_cursor
                .get_generated::<objects::Variable>(objects::ObjectId::from_name(self.name.clone()))
                .expect("couldn't get generated descriptor variable");

            Some(
                descriptor_module
                    .to_doc(ctx)
                    .append("::")
                    .append(descriptor_variable.to_doc(ctx)),
            )
        }
    }
}

mod type_declaration {
    use super::super::prelude::*;

    use std::rc::Rc;

    impl<'a> Type {
        pub(super) fn generate_declaration(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let message_type_object = objects::Type::from_object(
                ObjectId(NodeId::id(self), Tag::String("type")),
                self.name.clone(),
            );
            let name = namespace.name_object(&message_type_object);
            let (_, _) = namespace.insert_object(message_type_object.clone(), &name);

            let body = self.generate_body((ctx, namespace));
            let dependencies = self.generate_dependencies((ctx, namespace));

            let _ = namespace.remove_tree(&ObjectId(NodeId::id(self), Tag::String("type")));
            let (message_type_name, mut message_type_namespace) =
                namespace.insert_object(message_type_object, &name);

            let (body_field, _) = message_type_namespace
                .insert_object_preserve_name(objects::Variable::from_name("body".to_owned()))
                .expect("couldn't insert body field");
            let (dependencies_field, _) = message_type_namespace
                .insert_object_preserve_name(objects::Variable::from_name(
                    "dependencies".to_owned(),
                ))
                .expect("couldn't insert dependencies field");

            drop(message_type_namespace);

            let (body_type, _) = namespace
                .get_generated::<objects::Type>(ObjectId::from_name("Body".to_owned()))
                .expect("couldn't get Body type");
            let (dependencies_type, _) = namespace
                .get_generated::<objects::Type>(ObjectId::from_name("Dependencies".to_owned()))
                .expect("couldn't get Dependencies type");

            let message_struct = alloc
                .text("#[derive(Clone, Debug, PartialEq, Eq)]")
                .append(alloc.hardline())
                .append("pub struct")
                .append(alloc.space())
                .append(message_type_name.to_doc(ctx))
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(
                            alloc.intersperse(
                                [
                                    (body_field, body_type),
                                    (dependencies_field, dependencies_type),
                                ]
                                .into_iter()
                                .map(|(field, ty)| {
                                    alloc
                                        .text("pub")
                                        .append(alloc.space())
                                        .append(field.to_doc(ctx))
                                        .append(":")
                                        .append(alloc.space())
                                        .append(ty.to_doc(ctx))
                                }),
                                alloc.text(",").append(alloc.hardline()),
                            ),
                        )
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc();

            alloc
                .intersperse(
                    [body, dependencies, message_struct],
                    alloc.hardline().append(alloc.hardline()),
                )
                .into_doc()
        }

        fn generate_body(&self, (ctx, namespace): MutContext<'a, '_, '_>) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (body_type, mut body_namespace) =
                namespace.insert_object_auto_name(objects::Type::from_name("Body".to_owned()));

            // if I implement Codegen::generate as trait, those helper function can be generated automatically
            let generate_fields =
                |fields: &Vec<Rc<Symbol>>, namespace: &mut context::NamingContext<'a, '_>| {
                    fields
                        .iter()
                        .map(|symbol| {
                            symbol
                                .clone()
                                .generate_as_field_declaration((ctx, namespace))
                        })
                        .collect::<Vec<_>>()
                };

            let fields = match self.kind {
                ast::TypeKind::Message => {
                    debug_assert!(
                        self.constructors.len() == 1,
                        "Message expected to have only one constructor"
                    );
                    let constructor = self.constructors[0].as_ref();
                    alloc.intersperse(
                        generate_fields(&constructor.fields, &mut body_namespace),
                        alloc.text(",").append(alloc.hardline()),
                    )
                }
                ast::TypeKind::Enum => {
                    alloc.intersperse(
                        self.constructors
                            .iter()
                            .map(|constructor| {
                                // branches are not quite types but could be modeled as such
                                let (branch_type, mut branch_namespace) = body_namespace
                                    .insert_object_auto_name(objects::Type::from_object(
                                        ObjectId(
                                            NodeId::id_rc(constructor),
                                            Tag::String("enum_branch"),
                                        ),
                                        constructor.name.clone(),
                                    ));
                                branch_type
                                    .to_doc(ctx)
                                    .append(alloc.space())
                                    .append("{")
                                    .append(
                                        alloc
                                            .hardline()
                                            .append(alloc.intersperse(
                                                generate_fields(
                                                    &constructor.fields,
                                                    &mut branch_namespace,
                                                ),
                                                alloc.text(",").append(alloc.hardline()),
                                            ))
                                            .nest(NEST_UNIT)
                                            .append(alloc.hardline()),
                                    )
                                    .append("}")
                            })
                            .collect::<Vec<_>>(),
                        alloc.text(",").append(alloc.hardline()),
                    )
                }
            };

            let holder = match self.kind {
                ast::TypeKind::Message => "struct",
                ast::TypeKind::Enum => "enum",
            };
            alloc
                .text("#[derive(Clone, Debug, PartialEq, Eq)]")
                .append(alloc.hardline())
                .append(format!("pub {holder}"))
                .append(alloc.space())
                .append(body_type.to_doc(ctx))
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(fields)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }

        fn generate_dependencies(&self, (ctx, namespace): MutContext<'a, '_, '_>) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (dependencies_type, mut dependencies_namespace) = namespace
                .insert_object_auto_name(objects::Type::from_name("Dependencies".to_owned()));

            alloc
                .text("#[derive(Clone, Debug, PartialEq, Eq)]")
                .append(alloc.hardline())
                .append("pub")
                .append(alloc.space())
                .append("struct")
                .append(alloc.space())
                .append(dependencies_type.to_doc(ctx))
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(
                            alloc.intersperse(
                                self.dependencies
                                    .iter()
                                    .map(|symbol| {
                                        alloc.text("pub").append(alloc.space()).append(
                                            symbol.clone().generate_as_field_declaration((
                                                ctx,
                                                &mut dependencies_namespace,
                                            )),
                                        )
                                    })
                                    .collect::<Vec<_>>(),
                                alloc.text(",").append(alloc.hardline()),
                            ),
                        )
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }
    }
}

impl<'a> Type {
    pub fn generate_type_dependencies_struct<'cursor>(
        &self,
        (ctx, namespace): Context<
            'a,
            'cursor,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
        values: Vec<BoxDoc<'a>>,
    ) -> BoxDoc<'a> {
        assert!(
            values.len() == self.dependencies.len(),
            "not enough/to much values to initialize Dependencies"
        );
        let alloc = ctx.alloc;

        let (dependencies_type, dependencies_cursor) = namespace
            .get_generated::<objects::Type>(ObjectId::from_name("Dependencies".to_owned()))
            .expect("couldn't get Dependencies struct");

        dependencies_type
            .to_doc(ctx)
            .append(alloc.space())
            .append("{")
            .append(
                alloc
                    .hardline()
                    .append(alloc.intersperse(
                        self.dependencies.iter().zip(values).map(|(symbol, value)| {
                            let (field, _) = dependencies_cursor
                                .clone()
                                .get_generated::<objects::Variable>(ObjectId(
                                    NodeId::id_rc(symbol),
                                    Tag::None,
                                ))
                                .expect("couldn't get Dependencies field");
                            field
                                .to_doc(ctx)
                                .append(":")
                                .append(alloc.space())
                                .append(value)
                        }),
                        alloc.text(",").append(alloc.hardline()),
                    ))
                    .nest(NEST_UNIT)
                    .append(alloc.hardline()),
            )
            .append("}")
    }
}

mod type_inherent_impl {
    use std::{
        iter,
        rc::{Rc, Weak},
    };

    use crate::rust_gen::generate::value_from_expression::Locator;

    use super::super::prelude::*;

    struct ConstructorObjectsLocator {}

    impl super::value_from_expression::Locator<'_> for ConstructorObjectsLocator {}

    impl<'a> Type {
        pub(super) fn generate_inherent_impl(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (_, mut inherent_impl_namespace) = namespace.insert_object_auto_name(
                objects::Scope::new(ObjectId(NodeId::id(self), Tag::String("inherent_impl"))),
            );

            let constructors = self
                .constructors
                .iter()
                .map(|constructor| {
                    constructor
                        .generate_constructor_declaration((ctx, &mut inherent_impl_namespace))
                })
                .collect::<Vec<_>>();

            let serialize_function =
                self.generate_serialize_function_declaration((ctx, &mut inherent_impl_namespace));
            let deserilize_function =
                self.generate_deserialize_function_declaration((ctx, &mut inherent_impl_namespace));

            drop(inherent_impl_namespace);

            alloc
                .text("impl")
                .append(alloc.space())
                .append(
                    namespace
                        .get_generated::<objects::Type>(ObjectId(
                            NodeId::id(self),
                            Tag::String("type"),
                        ))
                        .expect("couldn't get message type")
                        .0
                        .to_doc(ctx),
                )
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(
                            alloc.intersperse(
                                constructors
                                    .into_iter()
                                    .chain(iter::once(serialize_function))
                                    .chain(iter::once(deserilize_function)),
                                alloc.hardline(),
                            ),
                        )
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }
    }

    impl<'a> Constructor {
        #[allow(clippy::too_many_lines, reason = "??? (128/100)")]
        fn generate_constructor_declaration(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (constructor_func, mut constructor_namespace) =
                namespace.insert_object_auto_name(objects::Function::from_object(
                    ObjectId(NodeId::id(self), Tag::None),
                    self.name.to_lowercase(),
                ));
            let namespace = &mut constructor_namespace;

            let params = self
                .implicits
                .iter()
                .map(|symbol| {
                    symbol
                        .clone()
                        .generate_as_field_declaration((ctx, namespace))
                })
                .collect::<Vec<_>>()
                .into_iter()
                .chain(self.fields.iter().map(|symbol| {
                    symbol
                        .clone()
                        .generate_as_field_declaration((ctx, namespace))
                }))
                .collect::<Vec<_>>();

            let (_, mut constructor_body) = namespace.insert_object_auto_name(objects::Scope::new(
                ObjectId::from_name("constructor_body".to_owned()),
            ));
            let namespace = &mut constructor_body;

            let (ty, dependencies) = match &self.result_type {
                TypeExpression::Type { call, dependencies } => {
                    (call.upgrade().expect("call to unknown type"), dependencies)
                }
            };
            let (body_var, _) =
                namespace.insert_object_auto_name(objects::Variable::from_name("body".to_owned()));
            let (dependencies_var, _) = namespace
                .insert_object_auto_name(objects::Variable::from_name("dependencies".to_owned()));

            let body_initialization = alloc
                .text("let ")
                .append(body_var.to_doc(ctx))
                .append(" = ")
                .append(self.generate_type_checker_if((ctx, namespace)))
                .append(" {")
                .append({
                    let (_, scope) = namespace.insert_object_auto_name(objects::Scope::new(
                        ObjectId::from_name("if_true".to_owned()),
                    ));
                    let fields = self
                        .fields
                        .iter()
                        .map(|field| {
                            scope
                                .get_generated::<objects::Variable>(ObjectId(
                                    NodeId::id_rc(field),
                                    Tag::None,
                                ))
                                .expect("couldn't get constructor param for field")
                                .0
                                .to_doc(ctx)
                        })
                        .collect();
                    drop(scope);

                    let (type_module_prefix, type_module_cursor) = ty
                        // this lookup is not necessary but here for generality
                        .lookup_type_module((ctx, namespace.cursor()))
                        .expect("couldn't get type module");
                    alloc
                        .hardline()
                        .append(type_module_prefix)
                        .append(self.generate_body_construction((ctx, type_module_cursor), fields))
                        .nest(NEST_UNIT)
                        .append(alloc.hardline())
                })
                .append("} else {")
                .append({
                    let (_, scope) = namespace.insert_object_auto_name(objects::Scope::new(
                        ObjectId::from_name("if_false".to_owned()),
                    ));
                    alloc
                        .hardline()
                        .append(Self::generate_constructor_error((ctx, scope.cursor())))
                        .nest(NEST_UNIT)
                        .append(alloc.hardline())
                })
                .append("}")
                .append("?;");

            let dependencies = dependencies
                .iter()
                .map(|expr| match expr {
                    // Variables are already Box<T>, use directly without re-boxing
                    ValueExpression::Variable(_) => expr.generate_as_value(
                        (ctx, namespace.cursor()),
                        &ConstructorObjectsLocator {},
                    ),
                    // Constructor calls and op calls return T, must be wrapped
                    _ => alloc
                        .text("Box")
                        .append("::")
                        .append("new")
                        .append("(")
                        .append(expr.generate_as_value(
                            (ctx, namespace.cursor()),
                            &ConstructorObjectsLocator {},
                        ))
                        .append(")")
                        .into_doc(),
                })
                .collect();

            let dependencies_initialization =
                alloc
                    .text("let ")
                    .append(dependencies_var.to_doc(ctx))
                    .append(" = ")
                    .append({
                        let (type_module_prefix, type_module_cursor) = ty
                            .lookup_type_module((ctx, namespace.cursor()))
                            .expect("couldn't get type module");
                        type_module_prefix.append(ty.generate_type_dependencies_struct(
                            (ctx, type_module_cursor),
                            dependencies,
                        ))
                    })
                    .append(";");

            let constructor_body = body_initialization
                .append(alloc.hardline())
                .append(dependencies_initialization)
                .append(alloc.hardline())
                // TODO
                .append("Ok(Self { body: ")
                .append(body_var.to_doc(ctx))
                .append(", dependencies: ")
                .append(dependencies_var.to_doc(ctx))
                .append(" })");
            alloc
                .text("pub fn".to_string())
                .append(alloc.space())
                .append(constructor_func.to_doc(ctx))
                .append("(")
                .append(alloc.intersperse(params, alloc.text(", ")))
                .append(") -> Result<Self, super::ConstructorError> {")
                .append(
                    alloc
                        .hardline()
                        .append(constructor_body)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }

        fn generate_type_checker_if(&self, (ctx, namespace): MutContext<'a, '_, '_>) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            alloc
                .text("if (")
                .append(alloc.intersperse(
                    self.fields.iter().map(|symbol| {
                        match &symbol.ty {
                            TypeExpression::Type {
                                call: _,
                                dependencies,
                            } => alloc
                                .text("(")
                                .append(alloc.intersperse(
                                    dependencies.iter().map(|expr| {
                                        let val = expr.generate_as_value(
                                            (ctx, namespace.cursor()),
                                            &ConstructorObjectsLocator {},
                                        );
                                        match expr {
                                            ValueExpression::Variable(_) => {
                                                alloc.text("&").append(val)
                                            }
                                            _ => alloc.text("&Box::new(").append(val).append(")"),
                                        }
                                    }),
                                    alloc.text(",").append(alloc.line()),
                                ))
                                .append(")"),
                        }
                    }),
                    alloc.text(",").append(alloc.line()),
                ))
                .append(") == (")
                .append(alloc.intersperse(
                    self.fields.iter().map(|symbol| {
                        let (field_var, _) = namespace
                            .get_generated::<objects::Variable>(ObjectId(
                                NodeId::id_rc(symbol),
                                Tag::None,
                            ))
                            .expect("constructor params must be already in the scope");

                        let symbol_ty = match &symbol.ty {
                            TypeExpression::Type {
                                call,
                                dependencies: _,
                            } => call.upgrade().expect("call to unknown type"),
                        };

                        let (_, dependencies_struct_namespace) = symbol_ty
                            .lookup_type_module((ctx, namespace.cursor()))
                            .expect("couldn't found field type")
                            .1
                            .get_generated::<objects::Type>(ObjectId::from_name(
                                "Dependencies".to_owned(),
                            ))
                            .expect("couldn't get Dependencies struct");

                        alloc
                            .text("(")
                            .append(alloc.intersperse(
                                symbol_ty.dependencies.iter().map(|dep_field| {
                                    alloc
                                        .text("&")
                                        .append(field_var.to_doc(ctx))
                                        .append(".")
                                        .append("dependencies") // because Message was not fully inserted TODO: fix that
                                        .append(".")
                                        .append(
                                            dependencies_struct_namespace
                                                .clone()
                                                .get_generated::<objects::Variable>(ObjectId(
                                                    NodeId::id_rc(dep_field),
                                                    Tag::None,
                                                ))
                                                .expect("couldn't found Dependencies field")
                                                .0
                                                .to_doc(ctx),
                                        )
                                }),
                                alloc.text(",").append(alloc.line()),
                            ))
                            .append(")")
                    }),
                    alloc.text(",").append(alloc.line()),
                ))
                .append(")")
                .into_doc()
        }

        fn generate_constructor_error<'cursor>(
            (ctx, _): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
        ) -> BoxDoc<'a> {
            // Dirty, because doesn't check that super include this
            // TODO: Now it can be implemented
            ctx.alloc
                .text("Err(super::ConstructorError::MismatchedDependencies)")
                .into_doc()
        }

        fn generate_body_construction<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            fields: Vec<BoxDoc<'a>>,
        ) -> BoxDoc<'a> {
            assert!(
                self.fields.len() == fields.len(),
                "unexpected amount of fields passed in"
            );

            let alloc = ctx.alloc;

            let (body_type, body_namespace) = namespace
                .get_generated::<objects::Type>(ObjectId::from_name("Body".to_owned()))
                .expect("couldn't get Body struct");

            let (constructor, constructor_namespace) = match self.result_type.get_type().kind {
                ast::TypeKind::Message => (body_type.to_doc(ctx), body_namespace),
                ast::TypeKind::Enum => {
                    let (enum_branch, enum_branch_namespace) = body_namespace
                        .get_generated::<objects::Type>(ObjectId(
                            NodeId::id(self),
                            Tag::String("enum_branch"),
                        ))
                        .expect("couldn't get enum branch");
                    (
                        body_type
                            .to_doc(ctx)
                            .append("::")
                            .append(enum_branch.to_doc(ctx)),
                        enum_branch_namespace,
                    )
                }
            };

            alloc
                .text("Ok(")
                .append(constructor)
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(alloc.intersperse(
                            self.fields.iter().zip(fields).map(|(field, value)| {
                                constructor_namespace
                                    .clone()
                                    .get_generated::<objects::Variable>(ObjectId(
                                        NodeId::id_rc(field),
                                        Tag::None,
                                    ))
                                    .expect("couldn't find body constructor field")
                                    .0
                                    .to_doc(ctx)
                                    .append(": ")
                                    .append(value)
                            }),
                            alloc.text(",").append(alloc.hardline()),
                        ))
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .append(")")
                .into_doc()
        }
    }

    impl<'a> Type {
        fn generate_serialize_function_declaration(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            // TODO: serialize function could overlap with other methods, in future need to separate serilialize/deserialize into separate trait
            let (serialize_function, mut serialize_function_namespace) = namespace
                .insert_object_preserve_name(objects::Function::from_name("serialize".to_owned()))
                .expect("couldn't generate serialize function");

            let (writer_type_parameter, _) = serialize_function_namespace
                .insert_object_preserve_name(objects::Type::from_name("W".to_owned()))
                .expect("couldn't generate W type parameter");

            let (self_parameter, _) = serialize_function_namespace
                .insert_object_preserve_name(objects::Variable::from_name("self".to_owned()))
                .expect("couldn't generate self function parameter");
            let (writer_parameter, _) = serialize_function_namespace
                .insert_object_preserve_name(objects::Variable::from_name("writer".to_owned()))
                .expect("couldn't generate writer function parameter");

            let function_body = match self.kind {
                ast::TypeKind::Message => self.generate_serialize_function_body_for_message((
                    ctx,
                    &mut serialize_function_namespace,
                )),
                ast::TypeKind::Enum => self.generate_serialize_function_body_for_enum((
                    ctx,
                    &mut serialize_function_namespace,
                )),
            };

            alloc
                .text("pub")
                .append(alloc.space())
                .append("fn")
                .append(alloc.space())
                .append(serialize_function.to_doc(ctx))
                .append("<")
                .append(
                    writer_type_parameter
                        .to_doc(ctx)
                        .append(":")
                        .append(alloc.space())
                        .append("super::Write"),
                )
                .append(">") // TODO
                .append("(")
                .append(
                    alloc.intersperse(
                        [
                            self_parameter.to_doc(ctx),
                            writer_parameter
                                .to_doc(ctx)
                                .append(":")
                                .append(alloc.space())
                                .append("&mut")
                                .append(alloc.space())
                                .append(writer_type_parameter.to_doc(ctx)),
                        ],
                        alloc.text(",").append(alloc.space()),
                    ),
                )
                .append(")")
                .append(alloc.space())
                .append("->")
                .append(alloc.space())
                .append("Result")
                .append("<")
                .append(alloc.intersperse(
                    ["()", "super::Error"],
                    alloc.text(",").append(alloc.space()),
                ))
                .append(">")
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(function_body)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }

        #[allow(clippy::too_many_lines, reason = "??? (104/100)")]
        fn generate_serialize_function_body_for_enum(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (writer_parameter, _) = namespace
                .get_generated::<objects::Variable>(objects::ObjectId::from_name(
                    "writer".to_owned(),
                ))
                .expect("couldn't get generated writer parameter");
            let (self_parameter, _) = namespace
                .get_generated::<objects::Variable>(objects::ObjectId::from_name("self".to_owned()))
                .expect("couldn't get generated self parameter");
            let (message_type_body_field, _) = namespace
                .get_generated::<objects::Type>(objects::ObjectId(
                    ast::NodeId::id(self),
                    objects::Tag::String("type"),
                ))
                .expect("couldn't get generated message type")
                .1
                .get_generated::<objects::Variable>(objects::ObjectId::from_name("body".to_owned()))
                .expect("couldn't get generated message type 'body' field");

            alloc
                .text("match")
                .append(alloc.space())
                .append(self_parameter.to_doc(ctx))
                .append(".")
                .append(message_type_body_field.to_doc(ctx))
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(alloc.concat(self.constructors.iter().map(|constructor| {
                            let (body_type, body_type_cursor) = namespace
                                .get_generated::<objects::Type>(objects::ObjectId::from_name(
                                    "Body".to_owned(),
                                ))
                                .expect("couldn't get generated Body type");

                            let (constructor_variant_type, _) = body_type_cursor
                                .clone()
                                .get_generated::<objects::Type>(objects::ObjectId(
                                    ast::NodeId::id_rc(constructor),
                                    Tag::String("enum_branch"),
                                ))
                                .expect("couldn't get generated enum variant for constructor");

                            drop(body_type_cursor);

                            let (_, mut variant_scope_namespace) = namespace
                                .insert_object_auto_name(objects::Scope::new(
                                    objects::ObjectId::from_name(constructor.name.clone()),
                                ));

                            body_type
                                .to_doc(ctx)
                                .append("::")
                                .append(constructor_variant_type.to_doc(ctx))
                                .append(alloc.space())
                                .append("{")
                                .append(alloc.space())
                                .append(alloc.intersperse(
                                    constructor.fields.iter().map(|field| {
                                        variant_scope_namespace
                                            .insert_object_auto_name(objects::Variable::from_name(
                                                field.name.clone(),
                                            ))
                                            .0
                                            .to_doc(ctx)
                                    }),
                                    alloc.text(",").append(alloc.space()),
                                ))
                                .append(alloc.space())
                                .append("}")
                                .append(alloc.space())
                                .append("=>")
                                .append(alloc.space())
                                .append("{")
                                .append(
                                    alloc
                                        .hardline()
                                        .append(
                                            writer_parameter
                                                .to_doc(ctx)
                                                .append(".")
                                                .append("write_all")
                                                .append("(")
                                                .append("&")
                                                .append("[")
                                                .append(constructor.generate_enum_descriptor((
                                                    ctx,
                                                    variant_scope_namespace.cursor(),
                                                )))
                                                .append("]")
                                                .append(")")
                                                .append("?")
                                                .append(";"),
                                        )
                                        .append(alloc.hardline())
                                        .append(constructor.generate_constructor_serialization((
                                            ctx,
                                            &mut variant_scope_namespace,
                                        )))
                                        .nest(NEST_UNIT),
                                )
                                .append("}")
                                .append(",")
                                .append(alloc.hardline())
                        })))
                        .nest(NEST_UNIT),
                )
                .append("}")
                .append(alloc.hardline())
                .append("Ok(())")
                .into_doc()
        }

        fn generate_serialize_function_body_for_message(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (self_parameter, _) = namespace
                .get_generated::<objects::Variable>(objects::ObjectId::from_name("self".to_owned()))
                .expect("couldn't get generated self parameter");

            let constructor = self.constructors[0].clone();

            let (body_type, _) = namespace
                .get_generated::<objects::Type>(objects::ObjectId::from_name("Body".to_owned()))
                .expect("couldn't get generated Body type");

            let (message_type_body_field, _) = namespace
                .get_generated::<objects::Type>(objects::ObjectId(
                    ast::NodeId::id(self),
                    objects::Tag::String("type"),
                ))
                .expect("couldn't get generated message type")
                .1
                .get_generated::<objects::Variable>(objects::ObjectId::from_name("body".to_owned()))
                .expect("couldn't get generated message type 'body' field");

            alloc
                .text("let")
                .append(alloc.space())
                .append(body_type.to_doc(ctx))
                .append("{")
                .append(alloc.space())
                .append(alloc.intersperse(
                    constructor.fields.iter().map(|field| {
                        namespace
                            .insert_object_auto_name(objects::Variable::from_name(
                                field.name.clone(),
                            ))
                            .0
                            .to_doc(ctx)
                    }),
                    alloc.text(",").append(alloc.space()),
                ))
                .append(alloc.space())
                .append("}")
                .append(alloc.space())
                .append("=")
                .append(alloc.space())
                .append(self_parameter.to_doc(ctx))
                .append(".")
                .append(message_type_body_field.to_doc(ctx))
                .append(";")
                .append(alloc.hardline())
                .append(constructor.generate_constructor_serialization((ctx, namespace)))
                .append("Ok(())")
                .into_doc()
        }
    }

    impl<'a> Constructor {
        fn generate_constructor_serialization(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (writer_parameter, _) = namespace
                .get_generated::<objects::Variable>(objects::ObjectId::from_name(
                    "writer".to_owned(),
                ))
                .expect("couldn't get generated writer parameter");

            alloc
                .concat(self.fields.iter().map(|field| {
                    namespace
                        .get_generated::<objects::Variable>(objects::ObjectId::from_name(
                            field.name.clone(),
                        ))
                        .expect("couldn't get generated constructor field")
                        .0
                        .to_doc(ctx)
                        .append(".")
                        .append("serialize") // TODO
                        .append("(")
                        .append(writer_parameter.to_doc(ctx))
                        .append(")")
                        .append("?")
                        .append(";")
                        .append(alloc.hardline())
                }))
                .into_doc()
        }
    }

    struct EnumConstructorDeserializationObjectsLocator {}

    impl<'a> super::value_from_expression::Locator<'a>
        for EnumConstructorDeserializationObjectsLocator
    {
        fn locate_variable<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            symbol: &Weak<Symbol>,
        ) -> BoxDoc<'a> {
            namespace
                .get_generated::<objects::Variable>(ObjectId(NodeId::id_weak(symbol), Tag::None))
                .expect("couldn't get variable")
                .0
                .to_doc(ctx)
                .append(".")
                .append("clone")
                .append("()")
        }
    }

    struct MessageConstructorDeserializationObjectsLocator {}

    impl<'a> super::value_from_expression::Locator<'a>
        for MessageConstructorDeserializationObjectsLocator
    {
        fn locate_variable<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            symbol: &Weak<Symbol>,
        ) -> BoxDoc<'a> {
            namespace
                .get_generated::<objects::Variable>(objects::ObjectId(
                    ast::NodeId::id_weak(symbol),
                    objects::Tag::None,
                ))
                .expect("couldn't get generated implicit")
                .0
                .to_doc(ctx)
                .append(".")
                .append("clone")
                .append("()")
        }
    }

    impl<'a> Type {
        fn generate_deserialize_function_declaration(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            // TODO: deserialize function could overlap with other methods, in future need to separate serilialize/deserialize into separate trait
            let (deserialize_function, mut deserialize_function_namespace) = namespace
                .insert_object_preserve_name(objects::Function::from_name("deserialize".to_owned()))
                .expect("couldn't generate deserialize function");

            let (reader_type_parameter, _) = deserialize_function_namespace
                .insert_object_preserve_name(objects::Type::from_name("R".to_owned()))
                .expect("couldn't generate R type parameter");

            let (reader_parameter, _) = deserialize_function_namespace
                .insert_object_preserve_name(objects::Variable::from_name("reader".to_owned()))
                .expect("couldn't generate reader function parameter");

            let (dependencies_type, _) = deserialize_function_namespace
                .get_generated::<objects::Type>(objects::ObjectId::from_name(
                    "Dependencies".to_owned(),
                ))
                .expect("coudln't get Dependencies type");

            let (dependencies_parameter, _) = deserialize_function_namespace
                .insert_object_preserve_name(objects::Variable::from_name(
                    "dependencies".to_owned(),
                ))
                .expect("couldn't generate dependencies function parameter");

            let function_body = match self.kind {
                ast::TypeKind::Message => self.generate_deserialize_function_body_for_message((
                    ctx,
                    &mut deserialize_function_namespace,
                )),
                ast::TypeKind::Enum => self.generate_deserialize_function_body_for_enum((
                    ctx,
                    &mut deserialize_function_namespace,
                )),
            };

            alloc
                .text("pub")
                .append(alloc.space())
                .append("fn")
                .append(alloc.space())
                .append(deserialize_function.to_doc(ctx))
                .append("<")
                .append(reader_type_parameter.to_doc(ctx))
                .append(":")
                .append(alloc.space())
                .append("super")
                .append("::")
                .append("Read")
                .append(">")
                .append("(")
                .append(
                    alloc.intersperse(
                        [
                            (
                                dependencies_parameter.to_doc(ctx),
                                dependencies_type.to_doc(ctx),
                            ),
                            (
                                reader_parameter.to_doc(ctx),
                                alloc
                                    .text("&mut")
                                    .append(alloc.space())
                                    .append(reader_type_parameter.to_doc(ctx))
                                    .into_doc(),
                            ),
                        ]
                        .into_iter()
                        .map(|(value, ty)| value.append(":").append(alloc.space()).append(ty)),
                        alloc.text(",").append(alloc.space()),
                    ),
                )
                .append(")")
                .append(alloc.space())
                .append("->")
                .append(alloc.space())
                .append("Result")
                .append("<")
                .append(alloc.intersperse(
                    [
                        alloc.text("Self"),
                        alloc.text("super").append("::").append("DeserializeError"),
                    ],
                    alloc.text(",").append(alloc.space()),
                ))
                .append(">")
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(function_body)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }

        fn generate_deserialize_function_body_for_message(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;
            let constructor = &self.constructors[0];

            let bindings: Vec<(objects::GeneratedVariable, Rc<Symbol>)> = {
                let (_, deps_type_cursor) = namespace
                    .get_generated::<objects::Type>(ObjectId::from_name("Dependencies".to_owned()))
                    .expect("couldn't get Dependencies type for implicit pre-binding");

                constructor
                    .implicits
                    .iter()
                    .zip(self.dependencies.iter())
                    .map(|(implicit, type_dep)| {
                        let (field, _) = deps_type_cursor
                            .clone()
                            .get_generated::<objects::Variable>(ObjectId(
                                NodeId::id_rc(type_dep),
                                Tag::None,
                            ))
                            .expect("couldn't get type dep field in Dependencies");
                        (field, Rc::clone(implicit))
                    })
                    .collect()
            };

            let dependencies_param = namespace
                .get_generated::<objects::Variable>(ObjectId::from_name("dependencies".to_owned()))
                .expect("couldn't get dependencies parameter")
                .0;

            let implicit_bindings =
                alloc.concat(bindings.into_iter().map(|(type_dep_field, implicit)| {
                    let (implicit_var, _) =
                        namespace.insert_object_auto_name(objects::Variable::from_object(
                            ObjectId(NodeId::id_rc(&implicit), Tag::None),
                            implicit.name.clone(),
                        ));

                    alloc
                        .text("let ")
                        .append(implicit_var.to_doc(ctx))
                        .append(" = ")
                        .append(dependencies_param.to_doc(ctx))
                        .append(".")
                        .append(type_dep_field.to_doc(ctx))
                        .append(".clone();")
                        .append(alloc.hardline())
                }));

            alloc
                .nil()
                .append(implicit_bindings)
                .append(constructor.generate_constructor_deserialization((ctx, namespace), false))
                .into_doc()
        }

        #[allow(clippy::too_many_lines, reason = "??? (103/100)")]
        fn generate_deserialize_function_body_for_enum(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (reader_parameter, _) = namespace
                .get_generated::<objects::Variable>(objects::ObjectId::from_name(
                    "reader".to_owned(),
                ))
                .expect("couldn't get generated reader parameter");

            let (descriptor_variable, _) =
                namespace.insert_object_auto_name(objects::Variable::from_object(
                    objects::ObjectId(
                        ast::NodeId::owned("descriptor".to_owned()),
                        objects::Tag::String("variable"),
                    ),
                    "descriptor".to_owned(),
                ));

            alloc
                .text("let")
                .append(alloc.space())
                .append("mut")
                .append(alloc.space())
                .append(descriptor_variable.to_doc(ctx))
                .append(alloc.space())
                .append("=")
                .append(alloc.space())
                .append("0")
                .append(";")
                .append(alloc.hardline())
                .append(reader_parameter.clone().to_doc(ctx))
                .append(".")
                .append("read")
                .append("(")
                .append("super::slice::from_mut(")
                .append("&mut")
                .append(alloc.space())
                .append(descriptor_variable.to_doc(ctx))
                .append(")")
                .append(")")
                .append(".")
                .append("map_err")
                .append("(|e| super::DeserializeError::IoError(e))")
                .append("?")
                .append(";")
                .append(alloc.hardline())
                .append("match")
                .append(alloc.space())
                .append(descriptor_variable.to_doc(ctx))
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(
                            alloc.concat(
                                self.constructors
                                    .iter()
                                    .map(|constructor| {
                                        let (_, mut variant_scope_namespace) = namespace
                                            .insert_object_auto_name(objects::Scope::new(
                                                objects::ObjectId::from_name(
                                                    constructor.name.clone(),
                                                ),
                                            ));

                                        constructor
                                            .generate_enum_descriptor((
                                                ctx,
                                                variant_scope_namespace.cursor(),
                                            ))
                                            .expect("couldn't generate enum descriptor")
                                            .append(alloc.space())
                                            .append("=>")
                                            .append(alloc.space())
                                            .append("{")
                                            .append(
                                                alloc
                                                    .hardline()
                                                    .append(
                                                        constructor
                                                            .generate_constructor_deserialization(
                                                                (ctx, &mut variant_scope_namespace),
                                                                true,
                                                            ),
                                                    )
                                                    .nest(NEST_UNIT),
                                            )
                                            .append("}")
                                    })
                                    .chain(iter::once(
                                        alloc
                                            .text("_")
                                            .append(alloc.space())
                                            .append("=>")
                                            .append(alloc.space())
                                            .append(
                                                "Err(super::DeserializeError::UnknownDescriptor)",
                                            )
                                            .into_doc(),
                                    ))
                                    .map(|variant| variant.append(",").append(alloc.hardline())),
                            ),
                        )
                        .nest(NEST_UNIT),
                )
                .append("}")
                .into_doc()
        }
    }

    impl<'a> Constructor {
        fn generate_constructor_deserialization(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
            is_enum_constructor: bool,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            if is_enum_constructor {
                let (dependencies_param, _) = namespace
                    .get_generated::<objects::Variable>(ObjectId::from_name(
                        "dependencies".to_owned(),
                    ))
                    .expect("couldn't get generated dependencies parameter");

                let (_, dependencies_type_cursor) = namespace
                    .get_generated::<objects::Type>(ObjectId::from_name("Dependencies".to_owned()))
                    .expect("couldn't get generated Dependencies type");

                let ty = self.result_type.get_type();
                let dependencies = ty
                    .dependencies
                    .iter()
                    .map(|dependency| {
                        dependencies_param
                            .to_doc(ctx)
                            .append(".")
                            .append(
                                dependencies_type_cursor
                                    .clone()
                                    .get_generated::<objects::Variable>(objects::ObjectId(
                                        ast::NodeId::id_rc(dependency),
                                        objects::Tag::None,
                                    ))
                                    .expect("couldn't get generated dependency")
                                    .0
                                    .to_doc(ctx),
                            )
                            .append(".")
                            .append("body")
                    })
                    .collect::<Vec<_>>();

                drop(dependencies_type_cursor);

                let (_, mut implicits_extractor_if_scope) =
                    namespace.insert_object_auto_name(objects::Scope::new(
                        objects::ObjectId::from_name("implicits_extractor".to_owned()),
                    ));

                let implicits_extracting_patterns = self
                    .result_type
                    .get_dependencies()
                    .iter()
                    .map(|dependency| {
                        dependency.generate_as_pattern((ctx, &mut implicits_extractor_if_scope))
                    })
                    .collect::<Vec<_>>();

                alloc
                    .text("if")
                    .append(alloc.space())
                    .append("let")
                    .append(alloc.space())
                    .append("(")
                    .append(alloc.intersperse(
                        implicits_extracting_patterns,
                        alloc.text(",").append(alloc.space()),
                    ))
                    .append(")")
                    .append(alloc.space())
                    .append("=")
                    .append(alloc.space())
                    .append("(")
                    .append(alloc.intersperse(dependencies, alloc.text(",").append(alloc.space())))
                    .append(")")
                    .append(alloc.space())
                    .append("{")
                    .append(
                        alloc
                            .hardline()
                            .append(self.generate_constructor_call(
                                (ctx, &mut implicits_extractor_if_scope),
                                true,
                            ))
                            .nest(NEST_UNIT),
                    )
                    .append("}")
                    .append(alloc.space())
                    .append("else")
                    .append(alloc.space())
                    .append("{")
                    .append(
                        alloc
                            .hardline()
                            .append("Err(super::DeserializeError::DependenciesDescriptorMismatch)")
                            .nest(NEST_UNIT)
                            .append(alloc.hardline()),
                    )
                    .append("}")
                    .into_doc()
            } else {
                self.generate_constructor_call((ctx, namespace), false)
            }
        }

        #[allow(clippy::too_many_lines, reason = "??? (114/100)")]
        fn generate_constructor_call(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
            is_enum_constructor: bool,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (reader_parameter, _) = namespace
                .get_generated::<objects::Variable>(objects::ObjectId::from_name(
                    "reader".to_owned(),
                ))
                .expect("couldn't get generated reader parameter");

            // This is dirty, because I can't type erase Locator as its methods take as argument Context<..., impl Cursor>
            // So cleaning this up requires type erased Cursor
            // TODO: cleanup when proper Cursor will be implemented

            // Collect raw pointers of field symbols so we can detect when a type dependency
            // references a previously-deserialized field value (which is T, not Box<T>).
            let field_symbol_ptrs: std::collections::HashSet<usize> =
                self.fields.iter().map(|f| Rc::as_ptr(f) as usize).collect();

            let fields_deserialization = alloc.concat(self.fields.iter().map(|field| {
                let field_ty = field.ty.get_type();

                let (field_type_module_prefix, field_type_module_cursor) = field_ty
                    .lookup_type_module((ctx, namespace.cursor()))
                    .expect("couldn't lookup type module");

                // let namespace_cursor = namespace.cursor();

                let generate_value_expression: Box<dyn Fn(&ValueExpression) -> BoxDoc<'a>> =
                    if is_enum_constructor {
                        Box::new(|value: &ValueExpression| {
                            let val = value.generate_as_value(
                                (ctx, namespace.cursor()),
                                &EnumConstructorDeserializationObjectsLocator {},
                            );
                            match value {
                                ValueExpression::Variable(_) => val,
                                _ => ctx
                                    .alloc
                                    .text("Box::new(")
                                    .append(val)
                                    .append(")")
                                    .into_doc(),
                            }
                        })
                    } else {
                        Box::new(|value: &ValueExpression| {
                            let val = value.generate_as_value(
                                (ctx, namespace.cursor()),
                                &MessageConstructorDeserializationObjectsLocator {},
                            );
                            match value {
                                ValueExpression::Variable(_) => val,
                                _ => ctx
                                    .alloc
                                    .text("Box::new(")
                                    .append(val)
                                    .append(")")
                                    .into_doc(),
                            }
                        })
                    };

                let dependencies_struct = field_type_module_prefix.append(
                    field_ty.generate_type_dependencies_struct(
                        (ctx, field_type_module_cursor),
                        field
                            .ty
                            .get_dependencies()
                            .iter()
                            .map(|dep| {
                                let val = generate_value_expression(dep);
                                // Field variables are plain T (not Box<T>), so they need wrapping
                                // when used as type dependencies (which expect Box<T>).
                                if let ValueExpression::Variable(weak) = dep {
                                    if field_symbol_ptrs
                                        .contains(&(std::rc::Weak::as_ptr(weak) as usize))
                                    {
                                        return alloc
                                            .text("Box::new(")
                                            .append(val)
                                            .append(")")
                                            .into_doc();
                                    }
                                }
                                val
                            })
                            .collect(),
                    ),
                );

                drop(generate_value_expression);

                let (field_type_type_prefix, _) = field_ty
                    .lookup_type_type((ctx, namespace.cursor()))
                    .expect("couldn't lookup type type");

                alloc
                    .text("let")
                    .append(alloc.space())
                    .append(
                        namespace
                            .insert_object_auto_name(objects::Variable::from_object(
                                objects::ObjectId(ast::NodeId::id_rc(field), objects::Tag::None),
                                field.name.clone(),
                            ))
                            .0
                            .to_doc(ctx),
                    )
                    .append(alloc.space())
                    .append("=")
                    .append(alloc.space())
                    .append(field_type_type_prefix)
                    .append("deserialize") // TODO
                    .append("(")
                    .append(alloc.intersperse(
                        [dependencies_struct, reader_parameter.to_doc(ctx)],
                        alloc.text(",").append(alloc.space()),
                    ))
                    .append(")")
                    .append("?")
                    .append(";")
                    .append(alloc.hardline())
            }));

            let constructor_construction = self.generate_constructor_construction(
                (ctx, namespace.cursor()),
                self.implicits
                    .iter()
                    .map(|implicit| {
                        if is_enum_constructor {
                            EnumConstructorDeserializationObjectsLocator {}.locate_variable(
                                (ctx, namespace.cursor()),
                                &Rc::downgrade(implicit),
                            )
                        } else {
                            MessageConstructorDeserializationObjectsLocator {}.locate_variable(
                                (ctx, namespace.cursor()),
                                &Rc::downgrade(implicit),
                            )
                        }
                    })
                    .chain(self.fields.iter().map(|field| {
                        alloc
                            .text("Box")
                            .append("::")
                            .append("new")
                            .append("(")
                            .append(
                                namespace
                                    .get_generated::<objects::Variable>(objects::ObjectId(
                                        ast::NodeId::id_rc(field),
                                        objects::Tag::None,
                                    ))
                                    .expect("couldn't get generated field variable")
                                    .0
                                    .to_doc(ctx),
                            )
                            .append(")")
                            .into_doc()
                    }))
                    .collect(),
            );

            fields_deserialization
                .append(constructor_construction)
                .append(".")
                .append("map_err")
                .append("(|e| super::DeserializeError::ConstructorError(e))")
                .append(alloc.hardline())
                .into_doc()
        }
    }
}

mod value_from_expression {
    use std::rc::Weak;

    use super::super::prelude::*;

    pub trait Locator<'a> {
        fn locate_variable<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            symbol: &Weak<Symbol>,
        ) -> BoxDoc<'a> {
            namespace
                .get_generated::<objects::Variable>(ObjectId(NodeId::id_weak(symbol), Tag::None))
                .expect("couldn't get variable")
                .0
                .to_doc(ctx)
        }

        fn locate_constructor<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            constructor: &Constructor,
        ) -> BoxDoc<'a> {
            let ty = constructor.result_type.get_type();

            let (type_module_prefix, type_module_cursor) = ty
                .lookup_type_module((ctx, namespace.clone()))
                .expect("couldn't lookup type module");

            let mut constructor_func = Some(type_module_prefix);

            type_module_cursor
                .clone()
                .lookup_generated::<objects::Type>(ObjectId(
                    NodeId::id_rc(&ty),
                    Tag::String("type"),
                ))
                .expect("couldn't lookup message type")
                .apply(|_, generated| {
                    constructor_func = Some(
                        constructor_func.take().unwrap().append(
                            objects::GeneratedType::try_from(generated)
                                .expect("expected type")
                                .to_doc(ctx),
                        ),
                    );
                });

            type_module_cursor
                .lookup_generated::<objects::Scope>(ObjectId(
                    NodeId::id_rc(&ty),
                    Tag::String("inherent_impl"),
                ))
                .expect("couldn't lookup inherent impl")
                .lookup_generated::<objects::Function>(ObjectId(NodeId::id(constructor), Tag::None))
                .expect("couldn't lookup type constructor")
                .apply(|_, generated| {
                    constructor_func = Some(
                        constructor_func.take().unwrap().append("::").append(
                            objects::GeneratedFunction::try_from(generated)
                                .expect("expected function")
                                .to_doc(ctx),
                        ),
                    );
                });

            constructor_func.unwrap()
        }
    }

    impl<'a> ValueExpression {
        pub(super) fn generate_as_value<'cursor, L: Locator<'a>>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            locator: &L,
        ) -> BoxDoc<'a> {
            match self {
                ValueExpression::OpCall(op_call) => {
                    generate_op_as_value(op_call, (ctx, namespace), locator)
                }
                ValueExpression::Constructor {
                    call,
                    implicits,
                    arguments,
                } => call
                    .upgrade()
                    .expect("call to unknown constructor")
                    .generate_call_as_value((ctx, namespace), locator, implicits, arguments),
                ValueExpression::Variable(weak) => locator
                    .locate_variable((ctx, namespace), weak)
                    .append(".clone()"),
            }
        }
    }

    fn generate_op_as_value<'a, 'cursor, L: Locator<'a>>(
        op_call: &OpCall,
        (ctx, namespace): Context<
            'a,
            'cursor,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
        locator: &L,
    ) -> BoxDoc<'a> {
        let alloc = ctx.alloc;
        match op_call {
            OpCall::Literal(literal) => {
                let string = match literal {
                    Literal::Bool(val) => val.to_string(),
                    Literal::Double(val) => val.to_string(),
                    Literal::Int(val) => val.to_string(),
                    Literal::UInt(val) => val.to_string(),
                    Literal::Str(val) => val.clone(),
                };
                alloc.text(string).into_doc()
            }
            OpCall::Unary(unary_op, operand) => {
                let operand = operand.generate_as_value((ctx, namespace.clone()), locator);
                match unary_op {
                    UnaryOp::Access { to, field } => {
                        let to = to.upgrade().expect("access from unknown type");
                        let (field, _) = to
                            .lookup_type_module((ctx, namespace.clone()))
                            .expect("couldn't find type module")
                            .1
                            .lookup_generated::<objects::Type>(ObjectId::from_name(
                                "Body".to_string(),
                            ))
                            .expect("couldn't find Body struct")
                            .get_generated::<objects::Variable>(ObjectId(
                                NodeId::id_weak(field),
                                Tag::None,
                            ))
                            .expect("couldn't get field request in access");
                        operand
                            .append(".")
                            .append("body")
                            .append(".")
                            .append(field.to_doc(ctx))
                    }
                    UnaryOp::Minus => ctx.alloc.text("-").append(operand).into_doc(),
                    UnaryOp::Bang => ctx.alloc.text("!").append(operand).into_doc(),
                }
            }
            OpCall::Binary(binary_op, lhs, rhs) => {
                let op = match binary_op {
                    BinaryOp::Plus => alloc.text("+"),
                    BinaryOp::Minus => alloc.text("-"),
                    BinaryOp::Star => alloc.text("*"),
                    BinaryOp::Slash => alloc.text("/"),
                    BinaryOp::BinaryAnd => alloc.text("&"),
                    BinaryOp::BinaryOr => alloc.text("|"),
                };
                alloc
                    .text("(")
                    .append(lhs.generate_as_value((ctx, namespace.clone()), locator))
                    .append(alloc.space())
                    .append(op)
                    .append(alloc.space())
                    .append(rhs.generate_as_value((ctx, namespace), locator))
                    .append(")")
                    .into_doc()
            }
        }
    }

    impl<'a> Constructor {
        fn generate_call_as_value<'cursor, L: Locator<'a>>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            locator: &L,
            implicits: &[ValueExpression],
            arguments: &[ValueExpression],
        ) -> BoxDoc<'a> {
            assert!(arguments.len() == self.fields.len());

            let alloc = ctx.alloc;

            let ty = self.result_type.get_type();

            locator
                .locate_constructor((ctx, namespace.clone()), self)
                .append("(")
                .append(
                    alloc.intersperse(
                        implicits
                            .iter()
                            .map(|expr| {
                                let val = expr.generate_as_value((ctx, namespace.clone()), locator);
                                match expr {
                                    ValueExpression::Variable(_) => val,
                                    _ => alloc.text("Box::new(").append(val).append(")").into_doc(),
                                }
                            })
                            .collect::<Vec<_>>()
                            .into_iter()
                            .chain(arguments.iter().map(|expr| {
                                let val = expr.generate_as_value((ctx, namespace.clone()), locator);
                                match expr {
                                    ValueExpression::Variable(_) => val,
                                    _ => alloc.text("Box::new(").append(val).append(")").into_doc(),
                                }
                            })),
                        alloc.text(",").append(alloc.line()),
                    ),
                )
                .append(")")
                .append(".expect(")
                .append(format!(
                    "\"constructor '{}::{}' failed\"",
                    ty.name, self.name
                ))
                .append(")")
        }
    }
}

impl<'a> Constructor {
    pub fn generate_constructor_construction<'cursor>(
        &self,
        (ctx, namespace): Context<
            'a,
            'cursor,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
        values: Vec<BoxDoc<'a>>,
    ) -> BoxDoc<'a> {
        let alloc = ctx.alloc;

        // TODO: rewrite from just using "Self" to checking that we are in fact inside impl of Self and using it only then.
        alloc
            .text("Self")
            .append("::")
            .append(
                namespace
                    .get_generated::<objects::Function>(objects::ObjectId(
                        NodeId::id(self),
                        Tag::None,
                    ))
                    .expect("couldn't get generated constructor constructor")
                    .0
                    .to_doc(ctx),
            )
            .append("(")
            .append(alloc.intersperse(values, alloc.text(",").append(alloc.space())))
            .append(")")
            .into_doc()
    }
}

impl<'a> Symbol {
    pub fn generate_as_field_declaration(
        self: Rc<Self>,
        (ctx, namespace): MutContext<'a, '_, '_>,
    ) -> BoxDoc<'a> {
        let ty = ctx
            .alloc
            .text("super::Box<")
            .append({
                let ty = self.ty.get_type();
                let (type_module_prefix, type_module) = ty
                    .lookup_type_module((ctx, namespace.cursor()))
                    .expect("couldn't lookup type module");
                type_module_prefix.append(
                    type_module
                        .get_generated::<objects::Type>(ObjectId(
                            NodeId::id_rc(&ty),
                            Tag::String("type"),
                        ))
                        .expect("couldn't get message type")
                        .0
                        .to_doc(ctx),
                )
            })
            .append(">");
        let name = namespace
            .insert_object_auto_name(objects::Variable::from_object(
                ObjectId(NodeId::id_rc(&self), Tag::None),
                self.name.clone(),
            ))
            .0
            .to_doc(ctx);
        name.append(": ").append(ty)
    }
}

// Those are quite dirty because there is no path api. So they need to return path prefix in return type as BoxDoc.
// TODO: come up with some convenient path api.
impl<'a> Type {
    pub fn lookup_type_module<'cursor>(
        &self,
        (ctx, namespace): Context<
            'a,
            'cursor,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
    ) -> Option<(
        BoxDoc<'a>,
        context::GeneratedCursor<
            'cursor,
            'a,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
    )> {
        if namespace
            .clone()
            .lookup_generated::<objects::Type>(ObjectId(NodeId::id(self), Tag::String("type")))
            .is_some()
        {
            Some((ctx.alloc.text("").into_doc(), namespace))
        } else {
            // dirty because there is no path api
            let mut path = None;
            let cursor = namespace
                .lookup_generated::<objects::Module>(ObjectId::from_name("deps".to_owned()))
                .expect("couldn't get deps module")
                .apply(|_, generated| {
                    path = Some(
                        objects::GeneratedModule::try_from(generated)
                            .expect("expected module")
                            .to_doc(ctx),
                    );
                })
                .lookup_generated::<objects::Module>(ObjectId(
                    NodeId::id(self),
                    Tag::String("module"),
                ))?
                .apply(|_, generated| {
                    path = Some(
                        path.take().unwrap().append("::").append(
                            objects::GeneratedModule::try_from(generated)
                                .expect("expected module")
                                .to_doc(ctx),
                        ),
                    );
                });
            Some((path.unwrap().append("::"), cursor))
        }
    }

    pub fn lookup_type_type<'cursor>(
        &self,
        (ctx, namespace): Context<
            'a,
            'cursor,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
    ) -> Option<(
        BoxDoc<'a>,
        context::GeneratedCursor<
            'cursor,
            'a,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
    )> {
        if namespace
            .clone()
            .lookup_generated::<objects::Type>(ObjectId(NodeId::id(self), Tag::String("type")))
            .is_some()
        {
            Some((ctx.alloc.text("Self").append("::").into_doc(), namespace))
        } else {
            // dirty because there is no path api
            let mut path = None;
            let cursor = namespace
                .lookup_generated::<objects::Module>(ObjectId::from_name("deps".to_owned()))
                .expect("couldn't get deps module")
                .apply(|_, generated| {
                    path = Some(
                        objects::GeneratedModule::try_from(generated)
                            .expect("expected module")
                            .to_doc(ctx),
                    );
                })
                .lookup_generated::<objects::Type>(ObjectId(NodeId::id(self), Tag::String("type")))?
                .apply(|_, generated| {
                    path = Some(
                        path.take().unwrap().append("::").append(
                            objects::GeneratedType::try_from(generated)
                                .expect("expected type")
                                .to_doc(ctx),
                        ),
                    );
                });
            Some((path.unwrap().append("::"), cursor))
        }
    }
}

impl<'a> ValueExpression {
    pub fn generate_as_pattern(&self, (ctx, namespace): MutContext<'a, '_, '_>) -> BoxDoc<'a> {
        let alloc = ctx.alloc;

        match self {
            ValueExpression::OpCall(_) => {
                panic!("OpCall's are currently not supported in patterns")
            }
            ValueExpression::Constructor {
                call,
                implicits: _,
                arguments,
            } => {
                let call = call.upgrade().expect("call to unknown constructor");
                let ty = match &call.result_type {
                    TypeExpression::Type {
                        call,
                        dependencies: _,
                    } => call.upgrade().expect("call to unknown type constructor"),
                };

                let (type_module_prefix, type_module_cursor) = ty
                    .lookup_type_module((ctx, namespace.cursor()))
                    .expect("coudln't lookup type module");

                let (body_type, body_type_cursor) = type_module_cursor
                    .get_generated::<objects::Type>(ObjectId::from_name("Body".to_owned()))
                    .expect("coudln't get generated Body type");

                let (constructor_enum_branch, constructor_enum_branch_cursor) = body_type_cursor
                    .get_generated::<objects::Type>(ObjectId(
                        ast::NodeId::id_rc(&call),
                        objects::Tag::String("enum_branch"),
                    ))
                    .expect("couldn't get constructor enum branch");

                // TODO: Do something with the following

                // Problem with current approach is following:
                // enum T (n Nat) {
                //     Zero => ...
                //     Suc ( p ) => ...
                //     Suc ( Suc ( p ) ) => ...
                // }
                // To third constructor be expressible (to something like `if let Suc { pred: Suc { pred } } = ...` be compiled)
                // in current api at least one of the following statements must be true:
                // 1. box, deref etc pattern becomes stable
                // 2. constructor fields are stored by reference

                // I see 2 approaces:
                // Either extract needed implicits from dependencies by hand without any matching.
                // Or implement reference type for the messages, so 2. option could be available.
                // Both requires non small amount of code.

                let fields = call
                    .fields
                    .iter()
                    .map(|field| {
                        constructor_enum_branch_cursor
                            .clone()
                            .get_generated::<objects::Variable>(objects::ObjectId(
                                ast::NodeId::id_rc(field),
                                objects::Tag::None,
                            ))
                            .expect("couldn't get constructor field")
                            .0
                            .to_doc(ctx)
                    })
                    .collect::<Vec<_>>();

                drop(constructor_enum_branch_cursor);

                let arguments = arguments
                    .iter()
                    .map(|argument| argument.generate_as_pattern((ctx, namespace)))
                    .collect::<Vec<_>>();

                type_module_prefix
                    .append(body_type.to_doc(ctx))
                    .append("::")
                    .append(constructor_enum_branch.to_doc(ctx))
                    .append(alloc.space())
                    .append("{")
                    .append(alloc.space())
                    .append(alloc.intersperse(
                        fields.into_iter().zip(arguments).map(|(field, argument)| {
                            field.append(":").append(alloc.space()).append(argument)
                        }),
                        alloc.text(",").append(alloc.space()),
                    ))
                    .append(alloc.space())
                    .append("}")
            }
            ValueExpression::Variable(symbol) => {
                let symbol = symbol.upgrade().expect("use of unknown variable");
                let (variable, _) =
                    namespace.insert_object_auto_name(objects::Variable::from_object(
                        objects::ObjectId(ast::NodeId::id_rc(&symbol), objects::Tag::None),
                        symbol.name.clone(),
                    ));
                variable.to_doc(ctx)
            }
        }
    }
}
