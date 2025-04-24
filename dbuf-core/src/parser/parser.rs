use std::rc::Rc;

use super::lexer::*;
use crate::ast::operators::*;
use crate::ast::parsed::definition::*;
use crate::ast::parsed::*;
use chumsky::{input::*, pratt::*, prelude::*};

pub fn create_parser<'src>(
) -> impl Parser<'src, &'src [Token], Module<Span, String>, extra::Err<Rich<'src, Token>>> {
    parser_type_declaration()
        .repeated()
        .collect()
        .then_ignore(end())
}

fn parser_type_declaration<'src>() -> impl Parser<
    'src,
    &'src [Token],
    Definition<Span, String, TypeDeclaration<Span, String>>,
    extra::Err<Rich<'src, Token>>,
> {
    end().map(|_| {
        (Definition {
            loc: todo!(),
            name: todo!(),
            data: todo!(),
        })
    })
}

pub fn parser_expression<'src, I>(
) -> impl Parser<'src, I, Expression<Span, String>, extra::Err<Rich<'src, Token>>>
where
    I: ValueInput<'src, Token = Token, Span = SimpleSpan>,
{
    recursive(|expr| {
        let value = select! {
            Token::BoolLiteral(b) => Literal::Bool(b),
            Token::IntLiteral(i) => Literal::Int(i),
            Token::UintLiteral(u) => Literal::UInt(u),
            Token::FloatLiteral(f) => Literal::Double(f),
            Token::StringLiteral(s) => Literal::Str(s.clone()),
        }
        .map_with(|l, extra| {
            let span: SimpleSpan = extra.span();
            Expression {
                loc: span.into(),
                node: ExpressionNode::OpCall(OpCall::Literal(l)),
            }
        });

        let paren = expr
            .clone()
            .delimited_by(just(Token::LParen), just(Token::RParen));

        let atom = choice((value, paren));
        atom.pratt((
            prefix(5, just(Token::Minus), |_, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Minus, Rc::new(rhs))),
                }
            }),
            prefix(5, just(Token::Bang), |_, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Unary(UnaryOp::Bang, Rc::new(rhs))),
                }
            }),
            infix(left(4), just(Token::Star), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Star,
                        Rc::new(lhs),
                        Rc::new(rhs),
                    )),
                }
            }),
            infix(left(4), just(Token::Slash), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Slash,
                        Rc::new(lhs),
                        Rc::new(rhs),
                    )),
                }
            }),
            infix(left(3), just(Token::Plus), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Plus,
                        Rc::new(lhs),
                        Rc::new(rhs),
                    )),
                }
            }),
            infix(left(3), just(Token::Minus), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Minus,
                        Rc::new(lhs),
                        Rc::new(rhs),
                    )),
                }
            }),
            infix(left(2), just(Token::Amp), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::And,
                        Rc::new(lhs),
                        Rc::new(rhs),
                    )),
                }
            }),
            infix(left(1), just(Token::Pipe), |lhs, _, rhs, extra| {
                let span: SimpleSpan = extra.span();
                Expression {
                    loc: span.into(),
                    node: ExpressionNode::OpCall(OpCall::Binary(
                        BinaryOp::Or,
                        Rc::new(lhs),
                        Rc::new(rhs),
                    )),
                }
            }),
        ))
    })
}
