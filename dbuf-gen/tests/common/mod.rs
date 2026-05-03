use dbuf_core::arena::InternedString;
use dbuf_core::ast::elaborated as e;

pub fn to_string_module(module: e::Module<InternedString>) -> e::Module<String> {
    dbuf_core::typecheck::rename::map_module(module, &|s: InternedString| s.as_ref().to_owned())
}

pub fn empty() -> e::Module<InternedString> {
    e::Module {
        types: vec![],
        constructors: vec![].into_iter().collect(),
    }
}
#[must_use]
pub fn nat() -> e::Module<InternedString> {
    e::Module {
        types: vec![(
            "Nat".to_owned().into(),
            e::Type {
                dependencies: Vec::new(),
                constructor_names: e::ConstructorNames::OfEnum(
                    ["Zero", "Suc"]
                        .into_iter()
                        .map(std::borrow::ToOwned::to_owned)
                        .map(InternedString::from)
                        .collect(),
                ),
            },
        )],
        constructors: vec![
            (
                "Zero".to_owned().into(),
                e::Constructor {
                    implicits: Vec::new(),
                    fields: Vec::new(),
                    result_type: e::TypeExpression::TypeExpression {
                        name: "Nat".to_owned().into(),
                        dependencies: e::Rec::new([]),
                    },
                },
            ),
            (
                "Suc".to_owned().into(),
                e::Constructor {
                    implicits: Vec::new(),
                    fields: vec![(
                        "pred".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    )],
                    result_type: e::TypeExpression::TypeExpression {
                        name: "Nat".to_owned().into(),
                        dependencies: e::Rec::new([]),
                    },
                },
            ),
        ]
        .into_iter()
        .collect(),
    }
}

pub fn vec() -> e::Module<InternedString> {
    e::Module {
        types: vec![(
            "Vec".to_owned().into(),
            e::Type {
                dependencies: vec![(
                    "n".to_owned().into(),
                    e::TypeExpression::TypeExpression {
                        name: "Nat".to_owned().into(),
                        dependencies: e::Rec::new([]),
                    },
                )],
                constructor_names: e::ConstructorNames::OfEnum(
                    ["Nil", "Cons"]
                        .into_iter()
                        .map(std::borrow::ToOwned::to_owned)
                        .map(InternedString::from)
                        .collect(),
                ),
            },
        )],
        constructors: vec![
            (
                "Nil".to_owned().into(),
                e::Constructor {
                    implicits: Vec::new(),
                    fields: Vec::new(),
                    result_type: e::TypeExpression::TypeExpression {
                        name: "Vec".to_owned().into(),
                        dependencies: e::Rec::new([e::ValueExpression::Constructor {
                            name: "Zero".to_owned().into(),
                            implicits: e::Rec::new([]),
                            arguments: e::Rec::new([]),
                            result_type: e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned().into(),
                                dependencies: e::Rec::new([]),
                            },
                        }]),
                    },
                },
            ),
            (
                "Cons".to_owned().into(),
                e::Constructor {
                    implicits: vec![(
                        "p".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    )],
                    fields: vec![
                        (
                            "value".to_owned().into(),
                            e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned().into(),
                                dependencies: e::Rec::new([]),
                            },
                        ),
                        (
                            "tail".to_owned().into(),
                            e::TypeExpression::TypeExpression {
                                name: "Vec".to_owned().into(),
                                dependencies: e::Rec::new([e::ValueExpression::Variable {
                                    name: "p".to_owned().into(),
                                    ty: e::TypeExpression::TypeExpression {
                                        name: "Nat".to_owned().into(),
                                        dependencies: e::Rec::new([]),
                                    },
                                }]),
                            },
                        ),
                    ],
                    result_type: e::TypeExpression::TypeExpression {
                        name: "Vec".to_owned().into(),
                        dependencies: e::Rec::new([e::ValueExpression::Constructor {
                            name: "Suc".to_owned().into(),
                            implicits: e::Rec::new([]),
                            arguments: e::Rec::new([e::ValueExpression::Variable {
                                name: "p".to_owned().into(),
                                ty: e::TypeExpression::TypeExpression {
                                    name: "Nat".to_owned().into(),
                                    dependencies: e::Rec::new([]),
                                },
                            }]),
                            result_type: e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned().into(),
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

pub fn user() -> e::Module<InternedString> {
    e::Module {
        types: vec![(
            "User".to_owned().into(),
            e::Type {
                dependencies: vec![(
                    "n".to_owned().into(),
                    e::TypeExpression::TypeExpression {
                        name: "Nat".to_owned().into(),
                        dependencies: e::Rec::new([]),
                    },
                )],
                constructor_names: e::ConstructorNames::OfMessage("User".to_owned().into()),
            },
        )],
        constructors: vec![(
            "User".to_owned().into(),
            e::Constructor {
                implicits: vec![(
                    "p".to_owned().into(),
                    e::TypeExpression::TypeExpression {
                        name: "Nat".to_owned().into(),
                        dependencies: e::Rec::new([]),
                    },
                )],
                fields: vec![
                    (
                        "id".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    ),
                    (
                        "age".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    ),
                ],
                result_type: e::TypeExpression::TypeExpression {
                    name: "User".to_owned().into(),
                    dependencies: e::Rec::new([e::ValueExpression::Variable {
                        name: "p".to_owned().into(),
                        ty: e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    }]),
                },
            },
        )]
        .into_iter()
        .collect(),
    }
}

pub fn inventory() -> e::Module<InternedString> {
    e::Module {
        types: vec![(
            "Inventory".to_owned().into(),
            e::Type {
                dependencies: vec![
                    (
                        "n".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    ),
                ],
                constructor_names: e::ConstructorNames::OfMessage("Inventory".to_owned().into()),
            },
        )],
        constructors: vec![(
            "Inventory".to_owned().into(),
            e::Constructor {
                implicits: vec![
                    (
                        "p".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    ),
                ],
                fields: vec![
                    (
                        "items".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Vec".to_owned().into(),
                            dependencies: e::Rec::new([e::ValueExpression::Variable {
                                name: "p".to_owned().into(),
                                ty: e::TypeExpression::TypeExpression {
                                    name: "Nat".to_owned().into(),
                                    dependencies: e::Rec::new([]),
                                },
                            }]),
                        },
                    ),
                    (
                        "count".to_owned().into(),
                        e::TypeExpression::TypeExpression {
                            name: "Nat".to_owned().into(),
                            dependencies: e::Rec::new([]),
                        },
                    ),
                ],
                result_type: e::TypeExpression::TypeExpression {
                    name: "Inventory".to_owned().into(),
                    dependencies: e::Rec::new([
                        e::ValueExpression::Variable {
                            name: "p".to_owned().into(),
                            ty: e::TypeExpression::TypeExpression {
                                name: "Nat".to_owned().into(),
                                dependencies: e::Rec::new([]),
                            },
                        },
                    ]),
                },
            },
        )]
        .into_iter()
        .collect(),
    }
}

#[must_use]
pub fn get_basic_module() -> e::Module<InternedString> {
    create_module(vec![nat()])
}

#[allow(clippy::too_many_lines, reason = "??? (131/100)")]
#[must_use]
pub fn get_nat_vec_module() -> e::Module<InternedString> {
    create_module(vec![nat(), vec()])
}

pub fn get_simple_message_module() -> e::Module<InternedString> {
    create_module(vec![nat(), user()])
}

#[must_use]
pub fn get_inventory_module() -> e::Module<InternedString> {
    create_module(vec![nat(), vec(), inventory()])
}

fn create_module(list: Vec<e::Module<InternedString>>) -> e::Module<InternedString> {
    list.into_iter().fold(empty(), |a, b| a.merge(b))
}
