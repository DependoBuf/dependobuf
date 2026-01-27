use chumsky::extra::Err;
use chumsky::input::ValueInput;
use chumsky::prelude::*;

use super::Location;
use super::{Child, Token, Tree, TreeKind};

use super::parser_error::ParsingError;
use super::parser_utils::{MapToken, MapTree};

pub fn file_parser<'src, I>() -> impl Parser<'src, I, Tree, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let message = message_parser();
    let comment = comment_parser();

    choice((message, comment))
        .repeated()
        .collect::<Vec<_>>()
        .map_with(|v, extra| Tree {
            kind: TreeKind::File,
            location: extra.span(),
            children: v,
        })
}

/// Parses one message
///
/// Pattern:
/// ```dbuf
/// /*one comment*/
/// message /*comments*/ UCIdentifier /*comments*/
///   <body>
/// ```
fn message_parser<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let comment = comment_parser();
    let comment_r = comment.clone().repeated().collect::<Vec<_>>();
    let message_kw = just(Token::Message).map_token();
    let type_ident = parse_type_identifier();
    let body_parser = body_parser();

    comment
        .or_not()
        .then(message_kw)
        .then(comment_r.clone())
        .then(type_ident)
        .then(comment_r)
        .then(body_parser)
        .map_tree(TreeKind::Message)
}

/// Parses body
fn body_parser<'src, I>() -> impl Parser<'src, I, Option<Child>, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    any().or_not().map(|_| None)
}

/// Parses one comment
fn comment_parser<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    let line_comment = select! {
        Token::LineComment(comment) => Token::LineComment(comment)
    }
    .map_token()
    .labelled("Line Comment");

    let block_comment = select! {
        Token::BlockComment(comment) => Token::BlockComment(comment)
    }
    .map_token()
    .labelled("Block Comment");

    choice((line_comment, block_comment))
}

/// Parses type identifier (`UCIdentifier`)
fn parse_type_identifier<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    select! {
        Token::UCIdentifier(name) => Token::UCIdentifier(name)
    }
    .map_token()
    .labelled("Type Identifier")
}

/// Parses var identifier (`LCIdentifier`)
fn parse_var_identifier<'src, I>() -> impl Parser<'src, I, Child, Err<ParsingError>> + Clone
where
    I: ValueInput<'src, Span = Location, Token = Token>,
{
    select! {
        Token::LCIdentifier(name) => Token::LCIdentifier(name)
    }
    .map_token()
    .labelled("Variable Identifier")
}
