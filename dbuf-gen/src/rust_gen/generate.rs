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
            .text("use dbuf_rust_runtime::{serde, Box, ConstructorError, DeserializeError, Serialize, Deserialize, to_vec, from_slice};")
            .append(alloc.hardline().append(alloc.hardline()))
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

        let module = alloc.intersperse(
            [
                alloc
                    .text("use super::serde::{self, Serialize, Deserialize};")
                    .into_doc(),
                self.generate_dependencies_import((ctx, &mut type_namespace)),
                self.generate_declaration((ctx, &mut type_namespace)),
                self.generate_inherent_impl((ctx, &mut type_namespace)),
                self.generate_serialize_trait_impl((ctx, &mut type_namespace)),
                self.generate_deserialize_trait_impl((ctx, &mut type_namespace)),
            ],
            alloc.hardline().append(alloc.hardline()),
        );

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

            let (box_type, _) = namespace
                .insert_object_preserve_name(objects::Type::from_name("Box".to_owned()))
                .expect("couldn't insert Box");
            let (constructor_error_type, _) = namespace
                .insert_object_preserve_name(objects::Type::from_name(
                    "ConstructorError".to_owned(),
                ))
                .expect("couldn't insert ConstructorError");
            let (deserialize_error_type, _) = namespace
                .insert_object_preserve_name(objects::Type::from_name(
                    "DeserializeError".to_owned(),
                ))
                .expect("couldn't insert DeserializeError");
            let (serialize_trait, _) = namespace
                .insert_object_preserve_name(objects::Type::from_name("Serialize".to_owned()))
                .expect("couldn't insert Serialize");
            let (deserialize_trait, _) = namespace
                .insert_object_preserve_name(objects::Type::from_name("Deserialize".to_owned()))
                .expect("couldn't insert Deserialize");
            let (to_vec_function, _) = namespace
                .insert_object_preserve_name(objects::Function::from_name("to_vec".to_owned()))
                .expect("couldn't insert Serialize");
            let (from_slice_function, _) = namespace
                .insert_object_preserve_name(objects::Function::from_name("from_slice".to_owned()))
                .expect("couldn't insert Deserialize");

            // TODO: get those from module
            let helpers_deps = alloc
                .text("pub(super) use super::super::{")
                .append(alloc.intersperse(
                    [
                        box_type.to_doc(ctx),
                        constructor_error_type.to_doc(ctx),
                        deserialize_error_type.to_doc(ctx),
                        serialize_trait.to_doc(ctx),
                        deserialize_trait.to_doc(ctx),
                        to_vec_function.to_doc(ctx),
                        from_slice_function.to_doc(ctx),
                    ],
                    alloc.text(",").append(alloc.space()),
                ))
                .append("};");

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
                        .append(
                            alloc.intersperse([helpers_deps, other_type_deps], alloc.hardline()),
                        )
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
                .text("#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]")
                .append(alloc.hardline())
                .append("#[serde(crate = \"self::serde\")]")
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
                        alloc.hardline(),
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
                .text("#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]")
                .append(alloc.hardline())
                .append("#[serde(crate = \"self::serde\")]")
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
                .text("#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]")
                .append(alloc.hardline())
                .append("#[serde(crate = \"self::serde\")]")
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
                                .append("Box::new(")
                                .append(value)
                                .append(")")
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
    use super::super::prelude::*;

    impl<'a> Type {
        pub(super) fn generate_inherent_impl(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (_, mut inherent_impl_namespace) =
                namespace.insert_object_auto_name(objects::Type::from_object(
                    ObjectId(NodeId::id(self), Tag::String("inherent_impl")),
                    "inherent_impl".to_owned(),
                ));

            let constructors = self
                .constructors
                .iter()
                .map(|constructor| {
                    constructor
                        .generate_constructor_declaration((ctx, &mut inherent_impl_namespace))
                })
                .collect::<Vec<_>>();

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
                        .append(alloc.intersperse(constructors, alloc.hardline()))
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }
    }

    impl<'a> Constructor {
        #[allow(clippy::too_many_lines, reason = "??? (128/100)")]
        pub fn generate_constructor_declaration(
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
                .map(|expr| expr.generate_as_value((ctx, namespace.cursor())))
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
                .append(") -> Result<Self, deps::ConstructorError> {")
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
                                        alloc.text("&").append(
                                            expr.generate_as_value((ctx, namespace.cursor())),
                                        )
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
            // Dirty, because doesn't check that deps include this
            // TODO: Now it can be implemented
            ctx.alloc
                .text("Err(deps::ConstructorError::MismatchedDependencies)")
                .into_doc()
        }
    }
}

mod type_serde_traits_impl {
    use super::super::prelude::*;

    impl<'a> Type {
        pub(super) fn generate_serialize_trait_impl(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (_, mut inherent_impl_namespace) =
                namespace.insert_object_auto_name(objects::Type::from_object(
                    ObjectId(NodeId::id(self), Tag::String("serialize_impl")),
                    "serialize_impl".to_owned(),
                ));

            let (serialize_func, mut func_namespace) = inherent_impl_namespace
                .insert_object_preserve_name(objects::Function::from_name("serialize".to_owned()))
                .expect("couldn't generate method serialize for Serialize trait");

            let (self_param, _) = func_namespace
                .insert_object_preserve_name(objects::Variable::from_name("self".to_owned()))
                .expect("couldn't generate parameter self");

            let func_body = alloc
                .text("deps::to_vec(&")
                .append(self_param.to_doc(ctx))
                .append(").unwrap().into_boxed_slice()")
                .into_doc();

            drop(func_namespace);

            let method = alloc
                .text("fn")
                .append(alloc.space())
                .append(serialize_func.to_doc(ctx))
                .append("(")
                .append(self_param.to_doc(ctx))
                .append(")")
                .append(alloc.space())
                .append("->")
                .append(alloc.space())
                .append("Box<[u8]>")
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(func_body)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}");

            drop(inherent_impl_namespace);

            alloc
                .text("impl deps::Serialize for ")
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
                        .append(method)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }

        pub(super) fn generate_deserialize_trait_impl(
            &self,
            (ctx, namespace): MutContext<'a, '_, '_>,
        ) -> BoxDoc<'a> {
            let alloc = ctx.alloc;

            let (_, mut inherent_impl_namespace) =
                namespace.insert_object_auto_name(objects::Type::from_object(
                    ObjectId(NodeId::id(self), Tag::String("deserialize_impl")),
                    "deserialize_impl".to_owned(),
                ));

            let (deserialize_func, mut func_namespace) = inherent_impl_namespace
                .insert_object_preserve_name(objects::Function::from_name("deserialize".to_owned()))
                .expect("couldn't generate method serialize for Deserialize trait");

            let (slice_param, _) = func_namespace
                .insert_object_preserve_name(objects::Variable::from_name("slice".to_owned()))
                .expect("couldn't generate parameter slice");

            let func_body = alloc
                .text("deps::from_slice::<Self>(")
                .append(slice_param.to_doc(ctx))
                .append(").map_err(|err| err.into())")
                .into_doc();

            drop(func_namespace);

            let method = alloc
                .text("fn")
                .append(alloc.space())
                .append(deserialize_func.to_doc(ctx))
                .append("<'a>")
                .append("(")
                .append(slice_param.to_doc(ctx))
                .append(":")
                .append(alloc.space())
                .append("&'a [u8]")
                .append(")")
                .append(alloc.space())
                .append("->")
                .append(alloc.space())
                .append("Result<Self, deps::DeserializeError>")
                .append(alloc.space())
                .append("{")
                .append(
                    alloc
                        .hardline()
                        .append(func_body)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}");

            drop(inherent_impl_namespace);

            alloc
                .text("impl deps::Deserialize for ")
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
                        .append(method)
                        .nest(NEST_UNIT)
                        .append(alloc.hardline()),
                )
                .append("}")
                .into_doc()
        }
    }
}

mod value_from_expression {
    use super::super::prelude::*;

    impl<'a> ValueExpression {
        pub(super) fn generate_as_value<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
        ) -> BoxDoc<'a> {
            match self {
                ValueExpression::OpCall(op_call) => generate_op_as_value(op_call, (ctx, namespace)),
                ValueExpression::Constructor {
                    call,
                    implicits,
                    arguments,
                } => call
                    .upgrade()
                    .expect("call to unknown constructor")
                    .generate_call_as_value((ctx, namespace), implicits, arguments),
                ValueExpression::Variable(weak) => namespace
                    .get_generated::<objects::Variable>(ObjectId(NodeId::id_weak(weak), Tag::None))
                    .expect("couldn't get variable")
                    .0
                    .to_doc(ctx),
            }
        }
    }

    fn generate_op_as_value<'a, 'cursor>(
        op_call: &OpCall,
        (ctx, namespace): Context<
            'a,
            'cursor,
            impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
        >,
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
                let operand = operand.generate_as_value((ctx, namespace.clone()));
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
                    .append(lhs.generate_as_value((ctx, namespace.clone())))
                    .append(alloc.space())
                    .append(op)
                    .append(alloc.space())
                    .append(rhs.generate_as_value((ctx, namespace)))
                    .append(")")
                    .into_doc()
            }
        }
    }

    impl<'a> Constructor {
        fn generate_call_as_value<'cursor>(
            &self,
            (ctx, namespace): Context<
                'a,
                'cursor,
                impl Cursor<&'cursor objects::GeneratedRustObject, ObjectId<'a>>,
            >,
            implicits: &[ValueExpression],
            arguments: &[ValueExpression],
        ) -> BoxDoc<'a> {
            assert!(arguments.len() == self.fields.len());

            let alloc = ctx.alloc;

            let ty = self.result_type.get_type();

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
                .lookup_generated::<objects::Type>(ObjectId(
                    NodeId::id_rc(&ty),
                    Tag::String("inherent_impl"),
                ))
                .expect("couldn't lookup inherent impl")
                .lookup_generated::<objects::Function>(ObjectId(NodeId::id(self), Tag::None))
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

            constructor_func
                .unwrap()
                .append("(")
                .append(
                    alloc.intersperse(
                        implicits
                            .iter()
                            .map(|expr| expr.generate_as_value((ctx, namespace.clone())))
                            .collect::<Vec<_>>()
                            .into_iter()
                            .chain(
                                arguments
                                    .iter()
                                    .map(|expr| expr.generate_as_value((ctx, namespace.clone()))),
                            ),
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
    pub fn generate_body_construction<'cursor>(
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

impl<'a> Symbol {
    pub fn generate_as_field_declaration(
        self: Rc<Self>,
        (ctx, namespace): MutContext<'a, '_, '_>,
    ) -> BoxDoc<'a> {
        let ty = ctx
            .alloc
            .text("deps::Box<")
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
}
