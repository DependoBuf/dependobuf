use std::rc::Rc;

use super::ast_builder::AstBuilder;
use crate::common::ast_access::{Loc, ParsedAst, Str};

use dbuf_core::ast::operators::*;
use dbuf_core::ast::parsed::*;

fn literal_expr(l: Literal) -> Expression<Loc, Str> {
    Expression {
        loc: Loc::default(),
        node: ExpressionNode::OpCall(OpCall::Literal(l)),
    }
}

fn var_expr(var: &str) -> Expression<Loc, Str> {
    Expression {
        loc: Loc::default(),
        node: ExpressionNode::FunCall {
            fun: Str::new(var),
            args: Rc::new([]),
        },
    }
}

fn access_expr(acc: &[&str]) -> Expression<Loc, Str> {
    assert!(acc.len() >= 2);

    let mut basic_expr = var_expr(acc[0]);

    for i in 1..acc.len() {
        basic_expr = Expression {
            loc: Loc::default(),
            node: ExpressionNode::OpCall(OpCall::Unary(
                UnaryOp::Access(Str::new(acc[i])),
                Rc::new(basic_expr),
            )),
        };
    }

    basic_expr
}

pub fn rename_parsed_ast() -> ParsedAst {
    let mut builder = AstBuilder::new();

    builder
        .with_message("M1")
        .with_dependency("d1", "Int")
        .with_dependency("d2", "String")
        .with_field("f1", "Int")
        .with_field("f2", "Int")
        .with_field("f3", "String");

    builder
        .with_message("M2")
        .with_dependency("d1", "Int")
        .with_huge_dependency(
            "d2",
            "M1",
            Rc::new([var_expr("d1"), literal_expr(Literal::Str("kek".to_owned()))]),
        )
        .with_field("f1", "Int")
        .with_field("f2", "String")
        .with_huge_field("f3", "M1", Rc::new([var_expr("d1"), var_expr("f2")]))
        .with_huge_field(
            "f4",
            "M1",
            Rc::new([
                access_expr(&["f3", "f1"]),
                literal_expr(Literal::Str("funny".to_owned())),
            ]),
        );

    builder
        .with_message("M3")
        .with_dependency("d1", "String")
        .with_field("f1", "Int")
        .with_field("f2", "String")
        .with_huge_field(
            "f3",
            "M1",
            Rc::new([var_expr("f1"), literal_expr(Literal::Str("kek".to_owned()))]),
        )
        .with_huge_field("f4", "M2", Rc::new([var_expr("f1"), var_expr("f3")]))
        .with_huge_field(
            "f5",
            "M1",
            Rc::new([
                access_expr(&["f4", "f4", "f2"]),
                access_expr(&["f4", "f3", "f3"]),
            ]),
        );

    builder.construct()
}
