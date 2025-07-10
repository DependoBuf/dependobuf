use dbuf_core::ast::elaborated as e;

#[must_use]
pub fn get_basic_module() -> e::Module<String> {
    e::Module {
        types: vec![(
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
    }
}

#[allow(clippy::too_many_lines, reason = "??? (131/100)")]
#[must_use]
pub fn get_nat_vec_module() -> e::Module<String> {
    e::Module {
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
                            "value".to_owned(),
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
    }
}
