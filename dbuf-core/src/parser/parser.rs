use super::lexer::*;
use crate::ast::parsed::definition::*;
use crate::ast::parsed::location::Offset;
use crate::ast::parsed::*;
use crate::ast::{operators::*, parsed::location::Location};
use chumsky::{input::*, pratt::*, prelude::*};

pub fn create_parser<'src, I>() -> impl Parser<
    'src,
    I,
    Module<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    parser_type_declaration()
        .repeated()
        .collect()
        .then_ignore(end())
}

fn parser_type_declaration<'src, I>() -> impl Parser<
    'src,
    I,
    Definition<Location<Offset>, String, TypeDeclaration<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let message_def = parser_message_def();
    let enum_def = parser_enum_def();

    choice((message_def, enum_def)).labelled("type declaration")
}

pub fn parser_message_def<'src, I>() -> impl Parser<
    'src,
    I,
    Definition<Location<Offset>, String, TypeDeclaration<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let dependencies = parser_depencies(0);
    let constructor_body = parser_constructor_body();

    just(Token::Message)
        .ignore_then(parser_type_identifier())
        .then(dependencies)
        .then(constructor_body)
        .map_with(|((name, deps), body), extra| Definition {
            loc: extra.span(),
            name,
            data: TypeDeclaration {
                dependencies: deps,
                body: TypeDefinition::Message(body),
            },
        })
        .labelled("message definition")
}

pub fn parser_enum_def<'src, I>() -> impl Parser<
    'src,
    I,
    Definition<Location<Offset>, String, TypeDeclaration<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let dependent = parser_dependent_enum_def();
    let independent = parser_independent_enum_def();

    choice((dependent, independent)).labelled("enum definition")
}

pub fn parser_dependent_enum_def<'src, I>() -> impl Parser<
    'src,
    I,
    Definition<Location<Offset>, String, TypeDeclaration<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
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
        .map_with(|((name, deps), branches), extra| Definition {
            loc: extra.span(),
            name,
            data: TypeDeclaration {
                dependencies: deps,
                body: TypeDefinition::Enum(branches),
            },
        })
        .labelled("dependent enum definition")
}

pub fn parser_independent_enum_def<'src, I>() -> impl Parser<
    'src,
    I,
    Definition<Location<Offset>, String, TypeDeclaration<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let constructors_block = parser_constructors_block();
    just(Token::Enum)
        .ignore_then(parser_type_identifier())
        .then(constructors_block)
        .map_with(|(name, vec), extra| Definition {
            loc: extra.span(),
            name,
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Enum(vec![EnumBranch {
                    patterns: vec![],
                    constructors: vec,
                }]),
            },
        })
        .labelled("independent enum definition")
}

pub fn parser_depencies<'src, I>(
    at_least: usize,
) -> impl Parser<
    'src,
    I,
    Definitions<Location<Offset>, String, TypeExpression<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let typed_variable = parser_typed_variable();
    let dependency = typed_variable.delimited_by(just(Token::LParen), just(Token::RParen));

    dependency
        .repeated()
        .at_least(at_least)
        .collect::<Vec<_>>()
        .labelled("dependencies")
}

pub fn parser_enum_branch<'src, I>() -> impl Parser<
    'src,
    I,
    EnumBranch<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
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

pub fn parser_constructors_block<'src, I>() -> impl Parser<
    'src,
    I,
    Definitions<Location<Offset>, String, ConstructorBody<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let constructor_body = parser_constructor_body();
    let constructor_declaration = parser_constructor_identifier()
        .then(constructor_body.or_not())
        .map_with(|(name, body), extra| Definition {
            loc: extra.span(),
            name,
            data: body.unwrap_or_default(),
        });

    constructor_declaration
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("constructors block")
}

pub fn parser_constructor_body<'src, I>() -> impl Parser<
    'src,
    I,
    ConstructorBody<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let typed_variable = parser_typed_variable();
    let field_declaration = typed_variable.then_ignore(just(Token::Semicolon));
    let fields = field_declaration.repeated().collect::<Vec<_>>();
    fields
        .delimited_by(just(Token::LBrace), just(Token::RBrace))
        .labelled("constructor body")
}

pub fn parser_typed_variable<'src, I>() -> impl Parser<
    'src,
    I,
    Definition<Location<Offset>, String, Expression<Location<Offset>, String>>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let type_expr = parser_type_expression();
    parser_var_identifier()
        .then(type_expr)
        .map_with(|(name, expr), extra| Definition {
            loc: extra.span(),
            name,
            data: expr,
        })
        .labelled("typed variable")
}

pub fn parser_pattern<'src, I>() -> impl Parser<
    'src,
    I,
    Pattern<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    recursive(|pattern| {
        let field_init = parser_var_identifier()
            .then_ignore(just(Token::Colon))
            .then(pattern.clone())
            .map_with(|(name, data), extra| Definition {
                loc: extra.span(),
                name,
                data,
            });

        let field_init_list = field_init
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>();

        let constructed_value = parser_constructor_identifier()
            .then(field_init_list.delimited_by(just(Token::LBrace), just(Token::RBrace)))
            .map_with(|(name, fields), extra| Pattern {
                loc: extra.span(),
                node: PatternNode::ConstructorCall { name, fields },
            })
            .labelled("constructed value");

        let literal = parser_literal().map_with(|l, extra| Pattern {
            loc: extra.span(),
            node: PatternNode::Literal(l),
        });

        let underscore = just(Token::Star)
            .ignored()
            .map_with(|_, extra| Pattern {
                loc: extra.span(),
                node: PatternNode::Underscore,
            })
            .labelled("underscore");
        let var_identifier = parser_var_identifier()
            .map_with(|name, extra| Pattern {
                loc: extra.span(),
                node: PatternNode::Variable { name },
            })
            .labelled("var identifier");
        choice((literal, underscore, var_identifier, constructed_value))
    })
    .labelled("pattern")
}

pub fn parser_type_expression<'src, I>() -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let expr = parser_expression();
    let primary = parser_primary_with_expression(expr);
    parser_type_expression_with_primary(primary)
}

pub fn parser_expression<'src, I>() -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    recursive(|expr| {
        let primary = parser_primary_with_expression(expr);
        let type_expr = parser_type_expression_with_primary(primary.clone());

        let op_expr = primary
            .pratt((
                prefix(5, just(Token::Plus), |_, rhs: Expression<_, _>, extra| {
                    Expression {
                        loc: extra.span(),
                        node: rhs.node,
                    }
                }),
                prefix(5, just(Token::Minus), |_, rhs, extra| Expression {
                    loc: extra.span(),
                    node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Minus, Rec::new(rhs))),
                }),
                prefix(5, just(Token::Bang), |_, rhs, extra| Expression {
                    loc: extra.span(),
                    node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Bang, Rec::new(rhs))),
                }),
                infix(left(4), just(Token::Star), |lhs, _, rhs, extra| {
                    Expression {
                        loc: extra.span(),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Star,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(4), just(Token::Slash), |lhs, _, rhs, extra| {
                    Expression {
                        loc: extra.span(),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Slash,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(3), just(Token::Plus), |lhs, _, rhs, extra| {
                    Expression {
                        loc: extra.span(),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Plus,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(3), just(Token::Minus), |lhs, _, rhs, extra| {
                    Expression {
                        loc: extra.span(),
                        node: ExpressionNode::OpCall(OpCall::Binary(
                            BinaryOp::Minus,
                            Rec::new(lhs),
                            Rec::new(rhs),
                        )),
                    }
                }),
                infix(left(2), just(Token::Amp), |lhs, _, rhs, extra| Expression {
                    loc: extra.span(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::BinaryAnd,
                        Rec::new(lhs),
                        Rec::new(rhs),
                    )),
                }),
                infix(left(1), just(Token::Pipe), |lhs, _, rhs, extra| {
                    Expression {
                        loc: extra.span(),
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
            Expression<Location<Offset>, String>,
            extra::Err<Rich<'src, Token, Location<Offset>>>,
        > + Clone,
) -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    parser_type_identifier()
        .then(primary.clone().repeated().collect::<Vec<_>>())
        .map_with(|(fun, args), extra| Expression {
            loc: extra.span(),
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
            Expression<Location<Offset>, String>,
            extra::Err<Rich<'src, Token, Location<Offset>>>,
        > + Clone,
) -> impl Parser<
    'src,
    I,
    Expression<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    let field_init = parser_var_identifier()
        .then_ignore(just(Token::Colon))
        .then(expr.clone())
        .map_with(|(name, data), extra| Definition {
            loc: extra.span(),
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
            loc: extra.span(),
            node: ExpressionNode::ConstructorCall { name, fields },
        })
        .labelled("constructed value");

    let value = parser_literal().map_with(|l, extra| Expression {
        loc: extra.span(),
        node: ExpressionNode::OpCall(OpCall::Literal(l)),
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
    Expression<Location<Offset>, String>,
    extra::Err<Rich<'src, Token, Location<Offset>>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    parser_var_identifier()
        .map_with(|name, extra| (name, extra.span()))
        .separated_by(just(Token::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|vec: Vec<(String, Location<Offset>)>| {
            let start: Expression<Location<Offset>, String> = {
                let (name, first_span) = &vec[0];
                Expression {
                    loc: *first_span,
                    node: ExpressionNode::Variable { name: name.clone() },
                }
            };

            vec.iter()
                .skip(1)
                .fold(start, |prev_expr, (name, cur_span)| Expression {
                    loc: *cur_span,
                    node: ExpressionNode::OpCall(OpCall::Unary(
                        UnaryOp::Access(name.clone()),
                        Rec::new(prev_expr),
                    )),
                })
        })
        .labelled("var access")
}

fn parser_type_identifier<'src, I>(
) -> impl Parser<'src, I, String, extra::Err<Rich<'src, Token, Location<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    select! {
        Token::UCIdentifier(s) => s,
    }
    .labelled("type identifier")
}

fn parser_constructor_identifier<'src, I>(
) -> impl Parser<'src, I, String, extra::Err<Rich<'src, Token, Location<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    select! {
        Token::UCIdentifier(s) => s,
    }
    .labelled("constructor identifier")
}

fn parser_var_identifier<'src, I>(
) -> impl Parser<'src, I, String, extra::Err<Rich<'src, Token, Location<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
{
    select! {
        Token::LCIdentifier(s) => s,
    }
    .labelled("var identifier")
}

fn parser_literal<'src, I>(
) -> impl Parser<'src, I, Literal, extra::Err<Rich<'src, Token, Location<Offset>>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = Location<Offset>>,
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
