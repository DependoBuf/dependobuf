use super::lexer::*;
use crate::ast::operators::*;
use crate::ast::parsed::definition::*;
use crate::ast::parsed::*;
use chumsky::{input::*, pratt::*, prelude::*};

pub fn create_parser<'src, I>(
) -> impl Parser<'src, I, Module<Span, String>, extra::Err<Rich<'src, Token>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan>,
{
    parser_type_declaration()
        .repeated()
        .collect()
        .then_ignore(end())
}

fn parser_type_declaration<'src, I>() -> impl Parser<
    'src,
    I,
    Definition<Span, String, TypeDeclaration<Span, String>>,
    extra::Err<Rich<'src, Token>>,
> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan>,
{
    end().map(|_| {
        (Definition {
            loc: todo!(),
            name: todo!(),
            data: todo!(),
        })
    })
}

pub fn parser_expression<'src, I>(
) -> impl Parser<'src, I, Expression<Span, String>, extra::Err<Rich<'src, Token>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan>,
{
    recursive(|expr| {
        let value = parser_literal();

        let field_init = select! {
            Token::LCIdentifier(s) => s,
        }
        .then_ignore(just(Token::Colon))
        .then(expr.clone())
        .map_with(|(var_ident, expr), extra| {
            let span: SimpleSpan = extra.span();
            Expression {
                loc: span.into(),
                node: ExpressionNode::FunCall {
                    fun: var_ident,
                    args: Rec::new([expr]),
                },
            }
        });

        let field_init_list = field_init
            .separated_by(just(Token::Comma))
            .allow_trailing()
            .collect::<Vec<_>>();

        let constructed_value = select! {
            Token::UCIdentifier(s) => s,
        }
        .then(field_init_list.delimited_by(just(Token::LBrace), just(Token::RBrace)))
        .map_with(|(name, vec), extra| {
            let span: SimpleSpan = extra.span();
            Expression {
                loc: span.into(),
                node: ExpressionNode::FunCall {
                    fun: name,
                    args: vec.into_boxed_slice().into(),
                },
            }
        });

        let var_access = select! {
            Token::LCIdentifier(s) => s,
        }
        .map_with(|name, extra| (name, extra.span()))
        .separated_by(just(Token::Dot))
        .at_least(1)
        .collect::<Vec<_>>()
        .map(|vec: Vec<(String, SimpleSpan)>| {
            let start: Expression<Span, String> = {
                let (name, first_span) = &vec[0];
                Expression {
                    loc: (*first_span).into(),
                    node: ExpressionNode::FunCall {
                        fun: name.clone(),
                        args: Rec::new([]),
                    },
                }
            };

            vec.iter()
                .skip(1)
                .fold(start, |prev_expr, (name, cur_span)| Expression {
                    loc: (*cur_span).into(),
                    node: ExpressionNode::OpCall(OpCall::Unary(
                        UnaryOp::Access(name.clone()),
                        Rec::new(prev_expr),
                    )),
                })
        });

        let paren = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let primary = choice((value, var_access, constructed_value, paren));

        let type_expr = select! {
            Token::UCIdentifier(s) => s,
        }
        .then(primary.clone().repeated().collect::<Vec<_>>())
        .map_with(|(name, vec), extra| {
            let span: SimpleSpan = extra.span();
            Expression {
                loc: span.into(),
                node: ExpressionNode::FunCall {
                    fun: name,
                    args: vec.into_boxed_slice().into(),
                },
            }
        });

        let op_expr = primary.pratt((
            prefix(5, just(Token::Minus), |_, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Minus, Rec::new(rhs))),
                }
            }),
            prefix(5, just(Token::Bang), |_, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Bang, Rec::new(rhs))),
                }
            }),
            infix(left(4), just(Token::Star), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Star,
                        Rec::new(lhs),
                        Rec::new(rhs),
                    )),
                }
            }),
            infix(left(4), just(Token::Slash), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Slash,
                        Rec::new(lhs),
                        Rec::new(rhs),
                    )),
                }
            }),
            infix(left(3), just(Token::Plus), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Plus,
                        Rec::new(lhs),
                        Rec::new(rhs),
                    )),
                }
            }),
            infix(left(3), just(Token::Minus), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Minus,
                        Rec::new(lhs),
                        Rec::new(rhs),
                    )),
                }
            }),
            infix(left(2), just(Token::Amp), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::And,
                        Rec::new(lhs),
                        Rec::new(rhs),
                    )),
                }
            }),
            infix(left(1), just(Token::Pipe), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Or,
                        Rec::new(lhs),
                        Rec::new(rhs),
                    )),
                }
            }),
        ));

        choice((op_expr, type_expr))
    })
}

fn parser_literal<'src, I>(
) -> impl Parser<'src, I, Expression<Span, String>, extra::Err<Rich<'src, Token>>> + Clone
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan>,
{
    select! {
        Token::BoolLiteral(b) => Literal::Bool(b),
        Token::IntLiteral(i) => Literal::Int(i),
        Token::UintLiteral(u) => Literal::UInt(u),
        Token::FloatLiteral(f) => Literal::Double(f),
        Token::StringLiteral(s) => Literal::Str(s),
    }
    .map_with(|l, extra| {
        let span: SimpleSpan = extra.span();
        Expression {
            loc: span.into(),
            node: ExpressionNode::OpCall(OpCall::Literal(l)),
        }
    })
}
