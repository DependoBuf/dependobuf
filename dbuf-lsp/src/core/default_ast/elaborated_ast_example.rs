//! Provides function, that returns Elaborated ast sample.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::core::ast_access::ElaboratedAst;

use dbuf_core::ast::elaborated::*;
use dbuf_core::ast::operators::*;

type Str = String;

/// using in places, where type deducing is unused by LSP.
fn empty_type() -> TypeExpression<Str> {
    TypeExpression::TypeExpression {
        name: "None".to_owned(),
        dependencies: Rec::new([]),
    }
}

fn simple_type(name: &str) -> TypeExpression<Str> {
    TypeExpression::TypeExpression {
        name: name.to_owned(),
        dependencies: Rec::new([]),
    }
}

fn get_literal_type(l: &Literal) -> TypeExpression<Str> {
    match l {
        Literal::Bool(_) => simple_type("Bool"),
        Literal::Double(_) => simple_type("Double"),
        Literal::Int(_) => simple_type("Int"),
        Literal::UInt(_) => simple_type("Unsigned"),
        Literal::Str(_) => simple_type("String"),
    }
}

fn literal_expr(l: Literal) -> ValueExpression<Str> {
    let result_type = get_literal_type(&l);
    ValueExpression::OpCall {
        op_call: OpCall::Literal(l),
        result_type,
    }
}

fn int_literal_expr(i: i64) -> ValueExpression<Str> {
    literal_expr(Literal::Int(i))
}

fn str_literal_expr(s: &str) -> ValueExpression<Str> {
    literal_expr(Literal::Str(s.to_owned()))
}

fn var_expr(var: &str) -> ValueExpression<Str> {
    ValueExpression::Variable {
        name: var.to_owned(),
        ty: empty_type(),
    }
}

fn access_expr(acc: &[&str]) -> ValueExpression<Str> {
    assert!(acc.len() >= 2);

    let mut basic_expr = var_expr(acc[0]);

    for access in acc.iter().skip(1) {
        basic_expr = ValueExpression::OpCall {
            op_call: OpCall::Unary(UnaryOp::Access((*access).to_string()), Rec::new(basic_expr)),
            result_type: empty_type(),
        }
    }

    basic_expr
}

fn type_expr(name: &str, dependenices: &[ValueExpression<Str>]) -> TypeExpression<Str> {
    TypeExpression::TypeExpression {
        name: name.to_owned(),
        dependencies: Rec::from(dependenices.to_owned()),
    }
}

fn type_context(
    field: &str,
    t: &str,
    dependenices: &[ValueExpression<Str>],
) -> (Str, TypeExpression<Str>) {
    (field.to_owned(), type_expr(t, dependenices))
}

fn message_type(name: &str, dependencies: Vec<(Str, TypeExpression<Str>)>) -> Type<Str> {
    Type {
        dependencies,
        constructor_names: ConstructorNames::OfMessage(name.to_owned()),
    }
}

#[allow(dead_code, reason = "simple example")]
fn enum_type(dependencies: Vec<(Str, TypeExpression<Str>)>, constructors: &[&str]) -> Type<Str> {
    let mut ctrs = BTreeSet::new();
    for c in constructors {
        ctrs.insert((*c).to_string());
    }
    Type {
        dependencies,
        constructor_names: ConstructorNames::OfEnum(ctrs),
    }
}
#[expect(clippy::too_many_lines, reason = "temporary code, no need to refactor")]
pub fn rename_elaborated_ast() -> ElaboratedAst {
    let mut elaborated = ElaboratedAst {
        types: vec![],
        constructors: BTreeMap::new(),
    };

    elaborated.types.push((
        "M1".to_owned(),
        message_type(
            "M1",
            vec![
                type_context("d1", "Int", &[]),
                type_context("d2", "String", &[]),
            ],
        ),
    ));

    elaborated.types.push((
        "M2".to_owned(),
        message_type(
            "M2",
            vec![
                type_context("d1", "Int", &[]),
                type_context("d2", "M1", &[var_expr("d1"), str_literal_expr("kek")]),
            ],
        ),
    ));

    elaborated.types.push((
        "M3".to_owned(),
        message_type("M3", vec![type_context("d1", "String", &[])]),
    ));

    elaborated.types.push((
        "Constructed".to_owned(),
        message_type("Constructed", vec![]),
    ));

    elaborated.types.push((
        "Simple".to_owned(),
        enum_type(
            vec![type_context("d1", "Int", &[])],
            &["Aliased", "Literaled", "Wild"],
        ),
    ));

    elaborated.types.push((
        "SimpleDepended".to_owned(),
        message_type(
            "SimpleDepended",
            vec![type_context("d1", "Simple", &[int_literal_expr(0)])],
        ),
    ));

    elaborated.types.push((
        "EnumConstructed".to_owned(),
        message_type("EnumConstructed", vec![]),
    ));

    elaborated.types.push((
        "PatternMatch".to_owned(),
        enum_type(
            vec![type_context(
                "d1",
                "M1",
                &[int_literal_expr(0), str_literal_expr("kek")],
            )],
            &["PatternMatched"],
        ),
    ));

    elaborated.constructors.insert(
        "M1".to_owned(),
        Constructor {
            implicits: vec![
                type_context("d1", "Int", &[]),
                type_context("d2", "String", &[]),
            ],
            fields: vec![
                type_context("f1", "Int", &[]),
                type_context("f2", "Int", &[]),
                type_context("f3", "String", &[]),
            ],
            result_type: type_expr("M1", &[var_expr("d1"), var_expr("d2")]),
        },
    );

    elaborated.constructors.insert(
        "M2".to_owned(),
        Constructor {
            implicits: vec![
                type_context("d1", "Int", &[]),
                type_context("d2", "M1", &[var_expr("d1"), str_literal_expr("kek")]),
            ],
            fields: vec![
                type_context("f1", "Int", &[]),
                type_context("f2", "String", &[]),
                type_context("f3", "M1", &[var_expr("d1"), var_expr("f2")]),
                type_context(
                    "f4",
                    "M1",
                    &[access_expr(&["f3", "f1"]), str_literal_expr("funny")],
                ),
            ],
            result_type: type_expr("M2", &[var_expr("d1"), var_expr("d2")]),
        },
    );

    elaborated.constructors.insert(
        "M3".to_owned(),
        Constructor {
            implicits: vec![type_context("d1", "String", &[])],
            fields: vec![
                type_context("f1", "Int", &[]),
                type_context("f2", "String", &[]),
                type_context("f3", "M1", &[var_expr("f1"), str_literal_expr("kek")]),
                type_context("f4", "M2", &[var_expr("f1"), var_expr("f3")]),
                type_context(
                    "f5",
                    "M1",
                    &[
                        access_expr(&["f4", "f4", "f2"]),
                        access_expr(&["f4", "f3", "f3"]),
                    ],
                ),
            ],
            result_type: type_expr("M3", &[var_expr("d1")]),
        },
    );

    let constructed = ValueExpression::Constructor {
        name: "M1".to_owned(),
        implicits: Rec::new([int_literal_expr(0), str_literal_expr("kek")]),
        arguments: Rec::new([
            int_literal_expr(0),
            int_literal_expr(1),
            str_literal_expr("kek"),
        ]),
        result_type: empty_type(),
    };

    elaborated.constructors.insert(
        "Constructed".to_owned(),
        Constructor {
            implicits: vec![],
            fields: vec![type_context(
                "f1",
                "M2",
                &[literal_expr(Literal::Int(0)), constructed],
            )],
            result_type: simple_type("Constructed"),
        },
    );

    elaborated.constructors.insert(
        "Aliased".to_owned(),
        Constructor {
            implicits: vec![type_context("alias", "Int", &[])],
            fields: vec![type_context(
                "f1",
                "M1",
                &[var_expr("alias"), str_literal_expr("kek")],
            )],
            result_type: type_expr("Simple", &[var_expr("alias")]),
        },
    );

    elaborated.constructors.insert(
        "Literaled".to_owned(),
        Constructor {
            implicits: vec![],
            fields: vec![type_context(
                "f1",
                "M1",
                &[int_literal_expr(1), str_literal_expr("kek")],
            )],
            result_type: type_expr("Simple", &[int_literal_expr(0)]),
        },
    );

    elaborated.constructors.insert(
        "Wild".to_owned(),
        Constructor {
            implicits: vec![type_context("d1", "Int", &[])],
            fields: vec![],
            result_type: type_expr("Simple", &[var_expr("d1")]),
        },
    );

    elaborated.constructors.insert(
        "SimpleDepended".to_owned(),
        Constructor {
            implicits: vec![type_context("d1", "Simple", &[int_literal_expr(0)])],
            fields: vec![],
            result_type: type_expr("SimpleDepended", &[var_expr("d1")]),
        },
    );

    let c1 = ValueExpression::Constructor {
        name: "Aliased".to_owned(),
        implicits: Rec::new([int_literal_expr(0)]),
        arguments: Rec::new([var_expr("f1")]),
        result_type: empty_type(),
    };

    let c2 = ValueExpression::Constructor {
        name: "M1".to_owned(),
        implicits: Rec::new([int_literal_expr(1), var_expr("kek")]),
        arguments: Rec::new([
            int_literal_expr(0),
            int_literal_expr(1),
            str_literal_expr("kek"),
        ]),
        result_type: empty_type(),
    };
    let c3 = ValueExpression::Constructor {
        name: "Literaled".to_owned(),
        implicits: Rec::new([]),
        arguments: Rec::new([c2]),
        result_type: empty_type(),
    };

    let c4 = ValueExpression::Constructor {
        name: "Wild".to_owned(),
        implicits: Rec::new([]),
        arguments: Rec::new([]),
        result_type: empty_type(),
    };

    elaborated.constructors.insert(
        "EnumConstructed".to_owned(),
        Constructor {
            implicits: vec![],
            fields: vec![
                type_context("f1", "M1", &[int_literal_expr(0), str_literal_expr("kek")]),
                type_context("f2", "SimpleDepended", &[c1]),
                type_context("f3", "SimpleDepended", &[c3]),
                type_context("f4", "SimpleDepended", &[c4]),
            ],
            result_type: simple_type("EnumConstructed"),
        },
    );

    let constructed = ValueExpression::Constructor {
        name: "M1".to_owned(),
        implicits: Rec::new([int_literal_expr(0), str_literal_expr("kek")]),
        arguments: Rec::new([
            var_expr("alias"),
            int_literal_expr(1),
            str_literal_expr("literal"),
        ]),
        result_type: empty_type(),
    };

    elaborated.constructors.insert(
        "PatternMatched".to_owned(),
        Constructor {
            implicits: vec![type_context("alias", "Int", &[])],
            fields: vec![type_context(
                "f1",
                "M1",
                &[var_expr("alias"), str_literal_expr("Kek")],
            )],
            result_type: type_expr("PatternMatch", &[constructed]),
        },
    );

    elaborated
}
