use chumsky::{error::Rich, input::*, ParseResult, Parser};
use lexer::{Span, Token};
use logos::Logos;
use parser::create_parser;

use crate::ast::parsed::{definition::Definition, TypeDeclaration};

pub mod lexer;
pub mod parser;

pub fn parse<'src>(
    input: &'src str,
) -> ParseResult<
    Vec<Definition<Span, String, TypeDeclaration<lexer::Span, String>>>,
    Rich<'src, Token>,
> {
    let lexer = Token::lexer(input);
    let token_iter = lexer.spanned().map(|(tok, span)| match tok {
        Ok(tok) => (tok, span.into()),
        Err(()) => (Token::Error, span.into()),
    });

    let token_stream =
        Stream::from_iter(token_iter.clone()).map((0..input.len()).into(), |(t, s): (_, _)| (t, s));

    for (t, s) in token_iter {
        println!("token: {:?}, span: {:?}", t, s);
    }

    create_parser().parse(token_stream)
}
