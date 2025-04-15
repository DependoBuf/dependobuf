//! Provides function, that returns Ast sample.
//!

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::common::ast_access::ElaboratedAst;

use dbuf_core::ast::elaborated::*;
use dbuf_core::ast::operators::*;

type Str = String;

fn literal_expr(l: Literal) -> Expression<Str> {
    Expression::OpCall(OpCall::Literal(l))
}

fn var_expr(var: &str) -> Expression<Str> {
    Expression::Variable {
        name: var.to_owned(),
    }
}

fn access_expr(acc: &[&str]) -> Expression<Str> {
    assert!(acc.len() >= 2);

    let mut basic_expr = var_expr(acc[0]);

    for i in 1..acc.len() {
        basic_expr = Expression::OpCall(OpCall::Unary(
            UnaryOp::Access(acc[i].to_string()),
            Rc::new(basic_expr),
        ));
    }

    basic_expr
}

fn type_expr(name: &str, dependenices: &[Expression<Str>]) -> TypeExpression<Str> {
    Expression::Type {
        name: name.to_owned(),
        dependencies: Rc::from(dependenices.to_owned()),
    }
}

fn type_context(
    field: &str,
    t: &str,
    dependenices: &[Expression<Str>],
) -> (Str, TypeExpression<Str>) {
    (field.to_owned(), type_expr(t, dependenices))
}

fn message_type(name: &str, dependencies: Vec<(Str, Expression<Str>)>) -> Type<Str> {
    Type {
        dependencies: dependencies,
        constructor_names: ConstructorNames::OfMessage(name.to_owned()),
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
            result_type: type_expr("M2", &[var_expr("d1")]),
        },
    );

    elaborated
}
