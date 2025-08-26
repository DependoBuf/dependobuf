use super::ast_builder::AstBuilder;
use crate::core::ast_access::LocNameHelper;
use crate::core::ast_access::{Loc, ParsedAst, Str};

use dbuf_core::ast::operators::*;
use dbuf_core::ast::parsed::definition::Definition;
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

fn expr_definition(
    name: &str,
    expr: Expression<Loc, Str>,
) -> Definition<Loc, Str, Expression<Loc, Str>> {
    Definition {
        loc: Loc::default(),
        name: Str::unsafe_new(name),
        data: expr,
    }
}

fn constructor_expr(
    name: &str,
    args: &[&str],
    values: &[Expression<Loc, Str>],
) -> Expression<Loc, Str> {
    let node = ExpressionNode::ConstructorCall {
        name: Str::unsafe_new(name),
        fields: args
            .iter()
            .zip(values)
            .map(|(arg, value)| expr_definition(arg, value.clone()))
            .collect(),
    };

    Expression {
        loc: Loc::default(),
        node,
    }
}

fn pattern_star() -> PatternNode<Loc, Str, Pattern<Loc, Str>> {
    PatternNode::Underscore
}

fn pattern_literal(l: Literal) -> PatternNode<Loc, Str, Pattern<Loc, Str>> {
    PatternNode::Literal(l)
}

fn pattern_alias(name: &str) -> PatternNode<Loc, Str, Pattern<Loc, Str>> {
    PatternNode::Variable {
        name: Str::unsafe_new(name),
    }
}

fn pattern_definition(
    name: &str,
    p: PatternNode<Loc, Str, Pattern<Loc, Str>>,
) -> Definition<Loc, Str, Pattern<Loc, Str>> {
    let p = Pattern {
        loc: Loc::default(),
        node: p,
    };
    Definition {
        loc: Loc::default(),
        name: Str::unsafe_new(name),
        data: p,
    }
}

#[expect(clippy::too_many_lines, reason = "temporary code, no need to refactor")]
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

    builder.with_message("Constructed").with_huge_field(
        "f1",
        "M2",
        Rec::new([
            literal_expr(Literal::Int(0)),
            constructor_expr(
                "M1",
                &["f1", "f2", "f3"],
                &[
                    literal_expr(Literal::Int(0)),
                    literal_expr(Literal::Int(1)),
                    literal_expr(Literal::Str("kek".to_owned())),
                ],
            ),
        ]),
    );

    let e = builder.with_enum("Simple").with_dependency("d1", "Int");
    e.with_branch()
        .with_pattern(pattern_alias("alias"))
        .with_constructor("Aliased")
        .with_huge_field(
            "f1",
            "M1",
            Rec::new([
                var_expr("alias"),
                literal_expr(Literal::Str("kek".to_owned())),
            ]),
        );
    e.with_branch()
        .with_pattern(pattern_literal(Literal::Int(0)))
        .with_constructor("Literaled")
        .with_huge_field(
            "f1",
            "M1",
            Rec::new([
                literal_expr(Literal::Int(1)),
                literal_expr(Literal::Str("kek".to_owned())),
            ]),
        );

    e.with_branch()
        .with_pattern(pattern_star())
        .with_constructor("Wild");

    builder.with_message("SimpleDepended").with_huge_dependency(
        "d1",
        "Simple",
        Rec::new([literal_expr(Literal::Int(0))]),
    );

    builder
        .with_message("EnumConstructed")
        .with_huge_field(
            "f1",
            "M1",
            Rec::new([
                literal_expr(Literal::Int(0)),
                literal_expr(Literal::Str("kek".to_owned())),
            ]),
        )
        .with_huge_field(
            "f2",
            "SimpleDepended",
            Rec::new([constructor_expr("Aliased", &["f1"], &[var_expr("f1")])]),
        )
        .with_huge_field(
            "f3",
            "SimpleDepended",
            Rec::new([constructor_expr(
                "Literaled",
                &["f1"],
                &[constructor_expr(
                    "M1",
                    &["f1", "f2", "f3"],
                    &[
                        literal_expr(Literal::Int(0)),
                        literal_expr(Literal::Int(1)),
                        literal_expr(Literal::Str("kek".to_owned())),
                    ],
                )],
            )]),
        )
        .with_huge_field(
            "f4",
            "SimpleDepended",
            Rec::new([constructor_expr("Wild", &[], &[])]),
        );

    let pattern_constructor = PatternNode::ConstructorCall {
        name: Str::unsafe_new("M1"),
        fields: vec![
            pattern_definition("f1", pattern_alias("alias")),
            pattern_definition("f2", pattern_literal(Literal::Int(1))),
            pattern_definition("f3", pattern_literal(Literal::Str("literal".to_owned()))),
        ],
    };

    builder
        .with_enum("PatternMatch")
        .with_huge_dependency(
            "d1",
            "M1",
            Rec::new([
                literal_expr(Literal::Int(0)),
                literal_expr(Literal::Str("kek".to_owned())),
            ]),
        )
        .with_branch()
        .with_pattern(pattern_constructor)
        .with_constructor("PatternMatched")
        .with_huge_field(
            "f1",
            "M1",
            Rec::new([
                var_expr("alias"),
                literal_expr(Literal::Str("kek".to_owned())),
            ]),
        );

    builder.construct()
}
