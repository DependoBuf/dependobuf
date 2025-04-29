//! Provides function, that returns Ast sample.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::rc::Rc;

use crate::core::ast_access::ElaboratedAst;

use dbuf_core::ast::elaborated::*;
use dbuf_core::ast::operators::*;

type Str = String;

fn empty_type() -> TypeExpression<Str> {
    TypeExpression::TypeExpression {
        name: "None".to_owned(),
        dependencies: Rc::new([]),
    }
}

fn simple_type(name: &str) -> TypeExpression<Str> {
    TypeExpression::TypeExpression {
        name: name.to_owned(),
        dependencies: Rc::new([]),
    }
}

fn get_literal_type(l: Literal) -> TypeExpression<Str> {
    match l {
        Literal::Bool(_) => simple_type("Bool"),
        Literal::Double(_) => simple_type("Double"),
        Literal::Int(_) => simple_type("Int"),
        Literal::UInt(_) => simple_type("Unsigned"),
        Literal::Str(_) => simple_type("String"),
    }
}

fn literal_expr(l: Literal) -> ValueExpression<Str> {
    ValueExpression::OpCall {
        op_call: OpCall::Literal(l.to_owned()),
        result_type: get_literal_type(l),
    }
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
            op_call: OpCall::Unary(UnaryOp::Access(access.to_string()), Rc::new(basic_expr)),
            result_type: empty_type(),
        }
    }

    basic_expr
}

fn type_expr(name: &str, dependenices: &[ValueExpression<Str>]) -> TypeExpression<Str> {
    TypeExpression::TypeExpression {
        name: name.to_owned(),
        dependencies: Rc::from(dependenices.to_owned()),
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

#[allow(dead_code)]
fn enum_type(dependencies: Vec<(Str, TypeExpression<Str>)>, constructors: &[&str]) -> Type<Str> {
    let mut ctrs = BTreeSet::new();
    for c in constructors.iter() {
        ctrs.insert(c.to_string());
    }
    Type {
        dependencies,
        constructor_names: ConstructorNames::OfEnum(ctrs),
    }
}

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
                type_context(
                    "d2",
                    "M1",
                    &[var_expr("d1"), literal_expr(Literal::Str("kek".to_owned()))],
                ),
            ],
        ),
    ));

    elaborated.types.push((
        "M3".to_owned(),
        message_type("M3", vec![type_context("d1", "String", &[])]),
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
                type_context(
                    "d2",
                    "M1",
                    &[var_expr("d1"), literal_expr(Literal::Str("kek".to_owned()))],
                ),
            ],
            fields: vec![
                type_context("f1", "Int", &[]),
                type_context("f2", "String", &[]),
                type_context("f3", "M1", &[var_expr("d1"), var_expr("f2")]),
                type_context(
                    "f4",
                    "M1",
                    &[
                        access_expr(&["f3", "f1"]),
                        literal_expr(Literal::Str("funny".to_owned())),
                    ],
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
                type_context(
                    "f3",
                    "M1",
                    &[var_expr("f1"), literal_expr(Literal::Str("kek".to_owned()))],
                ),
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

    elaborated
}
