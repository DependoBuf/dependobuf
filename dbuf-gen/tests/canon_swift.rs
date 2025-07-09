mod tests {
    use dbuf_core::ast::elaborated as e;
    use dbuf_gen::swift_gen;
    use pretty_assertions::assert_eq;

    #[test]
    fn basic() {
        let module = e::Module {
            types: vec![(
                "Nat".to_owned(),
                e::Type {
                    dependencies: Vec::new(),
                    constructor_names: e::ConstructorNames::OfEnum(
                        ["Zero", "Suc"].into_iter().map(|s| s.to_owned()).collect(),
                    ),
                },
            )],
            constructors: vec![
                (
                    "Zero".to_owned(),
                    e::Constructor {
                        implicits: Vec::new(),
                        fields: Vec::new(),
                        result_type: e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned(),
                            dependencies: e::Rec::new([]),
                        },
                    },
                ),
                (
                    "Suc".to_owned(),
                    e::Constructor {
                        implicits: Vec::new(),
                        fields: vec![(
                            "pred".to_owned(),
                            e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned(),
                                dependencies: e::Rec::new([]),
                            },
                        )],
                        result_type: e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned(),
                            dependencies: e::Rec::new([]),
                        },
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        let mut writer = Vec::new();
        assert!(swift_gen::generate_module(module, &mut writer).is_ok());

        let code = String::from_utf8(writer).expect("generated code must be correct utf8");

        let expected = include_str!("./canon_swift/basic.swift");

        assert_eq!(code, expected);
    }

    #[test]
    fn nat_vec() {
        let module = e::Module {
            types: vec![
                (
                    "Nat".to_owned(),
                    e::Type {
                        dependencies: Vec::new(),
                        constructor_names: e::ConstructorNames::OfEnum(
                            ["Zero", "Suc"]
                                .into_iter()
                                .map(std::borrow::ToOwned::to_owned)
                                .collect(),
                        ),
                    },
                ),
                (
                    "Vec".to_owned(),
                    e::Type {
                        dependencies: vec![(
                            "n".to_owned(),
                            e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned(),
                                dependencies: e::Rec::new([]),
                            },
                        )],
                        constructor_names: e::ConstructorNames::OfEnum(
                            ["Nil", "Cons"]
                                .into_iter()
                                .map(std::borrow::ToOwned::to_owned)
                                .collect(),
                        ),
                    },
                ),
            ],
            constructors: vec![
                (
                    "Zero".to_owned(),
                    e::Constructor {
                        implicits: Vec::new(),
                        fields: Vec::new(),
                        result_type: e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned(),
                            dependencies: e::Rec::new([]),
                        },
                    },
                ),
                (
                    "Suc".to_owned(),
                    e::Constructor {
                        implicits: Vec::new(),
                        fields: vec![(
                            "pred".to_owned(),
                            e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned(),
                                dependencies: e::Rec::new([]),
                            },
                        )],
                        result_type: e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned(),
                            dependencies: e::Rec::new([]),
                        },
                    },
                ),
                (
                    "Nil".to_owned(),
                    e::Constructor {
                        implicits: Vec::new(),
                        fields: Vec::new(),
                        result_type: e::TypeExpression::TypeExpression {
                            name: "Vec".to_owned(),
                            dependencies: e::Rec::new([e::ValueExpression::Constructor {
                                name: "Zero".to_owned(),
                                implicits: e::Rec::new([]),
                                arguments: e::Rec::new([]),
                                result_type: e::TypeExpression::TypeExpression {
                                    name: "Nat".to_owned(),
                                    dependencies: e::Rec::new([]),
                                },
                            }]),
                        },
                    },
                ),
                (
                    "Cons".to_owned(),
                    e::Constructor {
                        implicits: vec![(
                            "p".to_owned(),
                            e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned(),
                                dependencies: e::Rec::new([]),
                            },
                        )],
                        fields: vec![
                            (
                                "val".to_owned(),
                                e::TypeExpression::TypeExpression {
                                    name: "Nat".to_owned(),
                                    dependencies: e::Rec::new([]),
                                },
                            ),
                            (
                                "tail".to_owned(),
                                e::TypeExpression::TypeExpression {
                                    name: "Vec".to_owned(),
                                    dependencies: e::Rec::new([e::ValueExpression::Variable {
                                        name: "p".to_owned(),
                                        ty: e::TypeExpression::TypeExpression {
                                            name: "Nat".to_owned(),
                                            dependencies: e::Rec::new([]),
                                        },
                                    }]),
                                },
                            ),
                        ],
                        result_type: e::TypeExpression::TypeExpression {
                            name: "Vec".to_owned(),
                            dependencies: e::Rec::new([e::ValueExpression::Constructor {
                                name: "Suc".to_owned(),
                                implicits: e::Rec::new([]),
                                arguments: e::Rec::new([e::ValueExpression::Variable {
                                    name: "p".to_owned(),
                                    ty: e::TypeExpression::TypeExpression {
                                        name: "Nat".to_owned(),
                                        dependencies: e::Rec::new([]),
                                    },
                                }]),
                                result_type: e::TypeExpression::TypeExpression {
                                    name: "Nat".to_owned(),
                                    dependencies: e::Rec::new([]),
                                },
                            }]),
                        },
                    },
                ),
            ]
            .into_iter()
            .collect(),
        };

        let mut writer = Vec::new();
        assert!(swift_gen::generate_module(module, &mut writer).is_ok());

        let code = String::from_utf8(writer).expect("generated code must be correct utf8");

        let expected = include_str!("./canon_swift/nat_vec.swift");

        assert_eq!(code, expected);
    }
}
