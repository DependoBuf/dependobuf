use chumsky::{
    input::{Input, Stream},
    Parser,
};
use dbuf_core::parser::{lexer::*, parser::*};
use std::io::{self, Read};

use logos::Logos;

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("failed reading");

    let lexer = Token::lexer(&input);
    let token_iter = lexer.spanned().map(|(tok, span)| match tok {
        Ok(tok) => (tok, span.into()),
        Err(()) => (Token::Error, span.into()),
    });

    let token_stream =
        Stream::from_iter(token_iter.clone()).map((0..input.len()).into(), |(t, s): (_, _)| (t, s));

    for (t, s) in token_iter {
        println!("token: {:?}, span: {:?}", t, s);
    }

    let res = parser_expression().parse(token_stream);
    match res.into_result() {
        Ok(expr) => println!("{:#?}", expr),
        Err(err) => println!("error: {:?}", err),
    }
}
