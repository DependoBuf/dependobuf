use super::ast_builder::AstBuilder;
use crate::core::ast_access::LocStringHelper;
use crate::core::ast_access::{Loc, ParsedAst, Str};

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
        node: ExpressionNode::Variable {
            name: Str::unsafe_new(var),
        },
    }
}

fn access_expr(acc: &[&str]) -> Expression<Loc, Str> {
    assert!(acc.len() >= 2);

    let mut basic_expr = var_expr(acc[0]);

    for access in acc.iter().skip(1) {
        basic_expr = Expression {
            loc: Loc::default(),
            node: ExpressionNode::OpCall(OpCall::Unary(
                UnaryOp::Access(Str::unsafe_new(access)),
                Rec::new(basic_expr),
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
            Rec::new([var_expr("d1"), literal_expr(Literal::Str("kek".to_owned()))]),
        )
        .with_field("f1", "Int")
        .with_field("f2", "String")
        .with_huge_field("f3", "M1", Rec::new([var_expr("d1"), var_expr("f2")]))
        .with_huge_field(
            "f4",
            "M1",
            Rec::new([
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
            Rec::new([var_expr("f1"), literal_expr(Literal::Str("kek".to_owned()))]),
        )
        .with_huge_field("f4", "M2", Rec::new([var_expr("f1"), var_expr("f3")]))
        .with_huge_field(
            "f5",
            "M1",
            Rec::new([
                access_expr(&["f4", "f4", "f2"]),
                access_expr(&["f4", "f3", "f3"]),
            ]),
        );

    builder.construct()
}
