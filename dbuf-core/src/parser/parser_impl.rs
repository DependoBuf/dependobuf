use crate::ast::parsed::definition::*;
use crate::ast::parsed::located_name::LocatedName;
use crate::ast::parsed::location::Offset;
use crate::ast::parsed::*;
use crate::ast::{operators::*, parsed::location::Location};
use crate::parser::Token;
use chumsky::{input::*, pratt::*, prelude::*};

type Name = LocatedName<String, Offset>;

#[must_use]
pub fn create_parser<'src, I>() -> impl Parser<
    'src,
    I,
    Module<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    parser_type_declaration()
        .repeated()
        .collect()
        .then_ignore(end())
}

type ParsedTypeDeclaration =
    Definition<Location<Offset>, Name, TypeDeclaration<Location<Offset>, Name>>;

fn parser_type_declaration<'src, I>(
) -> impl Parser<'src, I, ParsedTypeDeclaration, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let message_def = parser_message_def();
    let enum_def = parser_enum_def();

    choice((message_def, enum_def)).labelled("type declaration")
}

fn parser_message_def<'src, I>(
) -> impl Parser<'src, I, ParsedTypeDeclaration, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let dependencies = parser_depencies(0);
    let constructor_body = parser_constructor_body();

    just(Token::Message)
        .ignore_then(parser_type_identifier())
        .then(dependencies)
        .then(constructor_body)
        .map_with(|((name, deps), body), extra| {
            let span: SimpleSpan<Offset> = extra.span();
            Definition {
                loc: span.into_range().try_into().expect("Always correct range"),
                name,
                data: TypeDeclaration {
                    dependencies: deps,
                    body: TypeDefinition::Message(body),
                },
            }
        })
        .labelled("message definition")
}

fn parser_enum_def<'src, I>(
) -> impl Parser<'src, I, ParsedTypeDeclaration, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let dependent = parser_dependent_enum_def();
    let independent = parser_independent_enum_def();

    choice((dependent, independent)).labelled("enum definition")
}

fn parser_dependent_enum_def<'src, I>(
) -> impl Parser<'src, I, ParsedTypeDeclaration, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let mapping_rule = parser_enum_branch();
    let rules = mapping_rule
        .repeated()
        .at_least(1)
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBrace), just(Token::RBrace));

    let dependencies = parser_depencies(1);
    just(Token::Enum)
        .ignore_then(parser_type_identifier())
        .then(dependencies)
        .then(rules)
        .map_with(|((name, deps), branches), extra| {
            let span: SimpleSpan<Offset> = extra.span();
            Definition {
                loc: span.into_range().try_into().expect("Always correct range"),
                name,
                data: TypeDeclaration {
                    dependencies: deps,
                    body: TypeDefinition::Enum(branches),
                },
            }
        })
        .labelled("dependent enum definition")
}

fn parser_independent_enum_def<'src, I>(
) -> impl Parser<'src, I, ParsedTypeDeclaration, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let constructors_block = parser_constructors_block();
    just(Token::Enum)
        .ignore_then(parser_type_identifier())
        .then(constructors_block)
        .map_with(|(name, vec), extra| {
            let span: SimpleSpan<Offset> = extra.span();
            Definition {
                loc: span.into_range().try_into().expect("Always correct range"),
                name,
                data: TypeDeclaration {
                    dependencies: vec![],
                    body: TypeDefinition::Enum(vec![EnumBranch {
                        patterns: vec![],
                        constructors: vec,
                    }]),
                },
            }
        })
        .labelled("independent enum definition")
}

type ParserDependencies =
    Definitions<Location<Offset>, Name, TypeExpression<Location<Offset>, Name>>;

fn parser_depencies<'src, I>(
    at_least: usize,
) -> impl Parser<'src, I, ParserDependencies, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let typed_variable = parser_typed_variable();
    let dependency = typed_variable.delimited_by(just(Token::LParen), just(Token::RParen));

    dependency
        .repeated()
        .at_least(at_least)
        .collect::<Vec<_>>()
        .labelled("dependencies")
}

fn parser_enum_branch<'src, I>() -> impl Parser<
    'src,
    I,
    EnumBranch<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let pattern = parser_pattern();
    let input_patterns = pattern
        .separated_by(just(Token::Comma))
        .at_least(1)
        .collect::<Vec<_>>();

    let constructor_block = parser_constructors_block();

    input_patterns
        .then_ignore(just(Token::Arrow))
        .then(constructor_block)
        .map(|(patterns, constructors)| EnumBranch {
            patterns,
            constructors,
        })
        .labelled("enum branch")
}

type ParsedConstructedBlock =
    Definitions<Location<Offset>, Name, ConstructorBody<Location<Offset>, Name>>;

fn parser_constructors_block<'src, I>(
) -> impl Parser<'src, I, ParsedConstructedBlock, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>>
       + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let constructor_body = parser_constructor_body();
    let constructor_declaration = parser_constructor_identifier()
        .then(constructor_body.or_not())
        .map_with(|(name, body), extra| {
            let span: SimpleSpan<Offset> = extra.span();
            Definition {
                loc: span.into_range().try_into().expect("Always correct range"),
                name,
                data: body.unwrap_or_default(),
            }
        });

    constructor_declaration
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("constructors block")
}

fn parser_constructor_body<'src, I>() -> impl Parser<
    'src,
    I,
    ConstructorBody<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let typed_variable = parser_typed_variable();
    let field_declaration = typed_variable.then_ignore(just(Token::Semicolon));
    let fields = field_declaration.repeated().collect::<Vec<_>>();
    fields
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("constructor body")
}

type ParsedTypedVariable = Definition<Location<Offset>, Name, Expression<Location<Offset>, Name>>;

fn parser_typed_variable<'src, I>(
) -> impl Parser<'src, I, ParsedTypedVariable, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let type_expr = parser_type_expression();
    parser_var_identifier()
        .then(type_expr)
        .map_with(|(name, expr), extra| {
            let span: SimpleSpan<Offset> = extra.span();
            Definition {
                loc: span.into_range().try_into().expect("Always correct range"),
                name,
                data: expr,
            }
        })
        .labelled("typed variable")
}

fn parser_pattern<'src, I>() -> impl Parser<
    'src,
    I,
    Pattern<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    recursive(|pattern| {
        let field_init = parser_var_identifier()
            .then_ignore(just(Token::Colon))
            .then(pattern.clone())
            .map_with(|(name, data), extra| {
                let span: SimpleSpan<Offset> = extra.span();
                Definition {
                    loc: span.into_range().try_into().expect("Always correct range"),
                    name,
                    data,
                }
            });

        let field_init_list = field_init
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>();

        let constructed_value = parser_constructor_identifier()
            .then(field_init_list.delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map_with(|(name, fields), extra| {
                let span: SimpleSpan<Offset> = extra.span();
                Pattern {
                    loc: span.into_range().try_into().expect("Always correct range"),
                    node: PatternNode::ConstructorCall { name, fields },
                }
            })
            .labelled("constructed value");

        let literal = parser_literal().map_with(|l, extra| {
            let span: SimpleSpan<Offset> = extra.span();
            Pattern {
                loc: span.into_range().try_into().expect("Always correct range"),
                node: PatternNode::Literal(l),
            }
        });

        let underscore = just(Token::Star)
            .ignored()
            .map_with(|(), extra| {
                let span: SimpleSpan<Offset> = extra.span();
                Pattern {
                    loc: span.into_range().try_into().expect("Always correct range"),
                    node: PatternNode::Underscore,
                }
            })
            .labelled("underscore");
        let var_identifier = parser_var_identifier()
            .map_with(|name, extra| {
                let span: SimpleSpan<Offset> = extra.span();
                Pattern {
                    loc: span.into_range().try_into().expect("Always correct range"),
                    node: PatternNode::Variable { name },
                }
            })
            .labelled("var identifier");
        choice((literal, underscore, var_identifier, constructed_value))
    })
    .labelled("pattern")
}

fn parser_type_expression<'src, I>() -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let expr = parser_expression();
    let primary = parser_primary_with_expression(expr);
    parser_type_expression_with_primary(primary)
}

fn parser_expression<'src, I>() -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    recursive(|expr| {
        let primary = parser_primary_with_expression(expr);
        let type_expr = parser_type_expression_with_primary(primary.clone());

        let op_expr = primary
            .pratt((
                prefix(5, just(Token::Plus), |_, rhs: Expression<_, _>, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: rhs.node,
                    }
                }),
                prefix(5, just(Token::Minus), |_, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Minus, Rec::new(rhs))),
                    }
                }),
                prefix(5, just(Token::Bang), |_, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Bang, Rec::new(rhs))),
                    }
                }),
                infix(left(4), just(Token::Star), |lhs, _, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Star,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(4), just(Token::Slash), |lhs, _, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Slash,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(3), just(Token::Plus), |lhs, _, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Plus,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(3), just(Token::Minus), |lhs, _, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Minus,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(2), just(Token::Amp), |lhs, _, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::BinaryAnd,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(1), just(Token::Pipe), |lhs, _, rhs, extra| {
                    let span: SimpleSpan<Offset> = extra.span();
                    Expression {
                        loc: span.into_range().try_into().expect("Always correct range"),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::BinaryOr,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
            ))
            .labelled("opearator expression");

        choice((type_expr, op_expr))
    })
}

fn parser_type_expression_with_primary<'src, I>(
    primary: impl Parser<
            'src,
            I,
            Expression<Location<Offset>, Name>,
            extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
        > + Clone,
) -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    parser_type_identifier()
        .then(primary.clone().repeated().collect::<Vec<_>>())
        .map_with(|(fun, args), extra| Expression {
            loc: extra
                .span()
                .into_range()
                .try_into()
                .expect("Always correct range"),
            node: ExpressionNode::FunCall {
                fun,
                args: args.into_boxed_slice().into(),
            },
        })
        .labelled("type expression")
}

fn parser_primary_with_expression<'src, I>(
    expr: impl Parser<
            'src,
            I,
            Expression<Location<Offset>, Name>,
            extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
        > + Clone,
) -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    let field_init = parser_var_identifier()
        .then_ignore(just(Token::Colon))
        .then(expr.clone())
        .map_with(|(name, data), extra| Definition {
            loc: extra
                .span()
                .into_range()
                .try_into()
                .expect("Always correct range"),
            name,
            data,
        });

    let field_init_list = field_init
        .separated_by(just(Token::Comma))
        .allow_trailing()
        .collect::<Vec<_>>();

    let constructed_value = parser_constructor_identifier()
        .then(field_init_list.delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map_with(|(name, fields), extra| Expression {
            loc: extra
                .span()
                .into_range()
                .try_into()
                .expect("Always correct range"),
            node: ExpressionNode::ConstructorCall { name, fields },
        })
        .labelled("constructed value");

    let value = parser_literal().map_with(|l, extra| {
        let span: SimpleSpan<Offset> = extra.span();
        Expression {
            loc: span.into_range().try_into().expect("Always correct range"),
            node: ExpressionNode::OpCall(OpCall::Literal(l)),
        }
    });

    let var_access = parser_var_access();

    let paren = expr
        .clone()
        .delimited_by(just(Token::LParen), just(Token::RParen))
        .labelled("paren");

    choice((paren, value, var_access, constructed_value)).labelled("primary")
}

fn parser_var_access<'src, I>() -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, Name>,
    extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    parser_var_identifier()
        .map_with(|name, extra| (name, extra.span()))
        .separated_by(just(Token::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|vec: Vec<(Name, SimpleSpan<Offset>)>| {
            let start: Expression<Location<Offset>, Name> = {
                let (name, first_span) = &vec[0];
                Expression {
                    loc: first_span
                        .into_range()
                        .try_into()
                        .expect("Always correct range"),
                    node: ExpressionNode::Variable { name: name.clone() },
                }
            };

            vec.iter()
                .skip(1)
                .fold(start, |prev_expr, (name, cur_span)| Expression {
                    loc: cur_span
                        .into_range()
                        .try_into()
                        .expect("Always correct range"),
                    node: ExpressionNode::OpCall(OpCall::Unary(
                        UnaryOp::Access(name.clone()),
                        Rec::new(prev_expr),
                    )),
                })
        })
        .labelled("var access")
}

fn parser_type_identifier<'src, I>(
) -> impl Parser<'src, I, Name, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    select! {
        Token::UCIdentifier(s) => s,
    }
    .map_with(|s, extra| {
        let span: SimpleSpan<Offset> = extra.span();
        let loc: Location<Offset> = span.into_range().try_into().expect("Always correct range");
        Name {
            content: s,
            start: loc.start,
        }
    })
    .labelled("type identifier")
}

fn parser_constructor_identifier<'src, I>(
) -> impl Parser<'src, I, Name, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    select! {
        Token::UCIdentifier(s) => s,
    }
    .map_with(|s, extra| {
        let span: SimpleSpan<Offset> = extra.span();
        let loc: Location<Offset> = span.into_range().try_into().expect("Always correct range");
        Name {
            content: s,
            start: loc.start,
        }
    })
    .labelled("constructor identifier")
}

fn parser_var_identifier<'src, I>(
) -> impl Parser<'src, I, Name, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    select! {
        Token::LCIdentifier(s) => s,
    }
    .map_with(|s, extra| {
        let span: SimpleSpan<Offset> = extra.span();
        let loc: Location<Offset> = span.into_range().try_into().expect("Always correct range");
        Name {
            content: s,
            start: loc.start,
        }
    })
    .labelled("var identifier")
}

fn parser_literal<'src, I>(
) -> impl Parser<'src, I, Literal, extra::Err<Rich<'src, Token, SimpleSpan<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan<Offset>>,
{
    select! {
        Token::BoolLiteral(b) => Literal::Bool(b),
        Token::IntLiteral(i) => Literal::Int(i),
        Token::UintLiteral(u) => Literal::UInt(u),
        Token::FloatLiteral(f) => Literal::Double(f),
        Token::StringLiteral(s) => Literal::Str(s),
    }
    .labelled("literal")
}
