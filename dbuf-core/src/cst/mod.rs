mod lexer;

mod located_token;
mod location;

mod parser;
mod parser_error;
mod parser_utils;

use std::fs;

use chumsky::{
    Parser,
    input::{Input, Stream},
};
use logos::Logos;

use lexer::Token;
use located_token::LocatedLexer;
use location::Location;

use crate::ast::parsed::location::Offset;

#[derive(Debug, Clone)]
pub enum TreeKind {
    ErrorTree,

    File,

    Message,
    Body,
    Definition,

    Enum,
    EnumBody,
    Branch,
    Pattern,
    ConstructedPattern,
    ConstructedPatternField,
    ConstructorEnum,
    Constructor,

    ConstructedValue,
    ConstructedValueField,

    ExprParen,
    ExprLiteral,
    ExprIdentifier,
    ExprBinary,
    ExprUnary,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Tree {
    kind: TreeKind,
    location: Location,
    children: Vec<Child>,
}

#[derive(Debug)]
pub enum Child {
    Token(Token, Location),
    Tree(Tree),
}

#[allow(clippy::missing_panics_doc)]
pub fn cst_main() {
    let str = fs::read_to_string("dbuf_file.dbuf").unwrap();

    println!("===========================");
    println!("|          Lexer          |");
    println!("===========================");

    let lexer = Token::lexer(&str);
    for x in lexer {
        match x {
            Ok(x) => println!("{x:#?}"),
            Err(e) => println!("!! {e}"),
        }
    }

    println!("===========================");
    println!("|         Parsing         |");
    println!("===========================");

    let lexer = LocatedLexer::from_lexer(Token::lexer(&str));
    let token_iter = lexer.map(|(tok, loc)| match tok {
        Ok(tok) => (tok, loc),
        Err(err) => (Token::Err(err), loc),
    });

    let lines_number = str.lines().count() + 1;
    let eof_offset = Offset {
        lines: lines_number,
        columns: 0,
    };
    let eof_location = Location::new(eof_offset, eof_offset).expect("correct location");
    let token_stream = Stream::from_iter(token_iter).map(eof_location, |(t, l)| (t, l));

    let parser = parser::file_parser();
    let result = parser.parse(token_stream);

    println!("{result:#?}");
}
