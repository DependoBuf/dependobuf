mod lexer;

mod located_token;
mod location;

mod parser;
mod parser_error;
mod parser_utils;
mod parser_whitespace;

mod cst_to_ast;

use chumsky::Parser;
use chumsky::input::{Input, Stream};
use logos::Logos;

use cst_to_ast::convert;
use located_token::LocatedLexer;

use crate::arena::InternedString;
use crate::ast::parsed::Module;
use crate::error::Error;
use crate::location::LocatedName;

use crate::location::Location;
use crate::location::Offset;

pub use lexer::Token;

#[derive(Debug, Clone, PartialEq)]
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
    ExprHole,
}

#[derive(Debug)]
pub struct Tree {
    pub kind: TreeKind,
    pub location: Location<Offset>,
    pub children: Vec<Child>,
}

#[derive(Debug)]
pub enum Child {
    Token(Token, Location<Offset>),
    Tree(Tree),
}

/// Parse text to CST.
#[must_use]
pub fn parse_to_cst(text: &str) -> (Option<Tree>, Vec<Error>) {
    let lexer = LocatedLexer::from_lexer(Token::lexer(text));

    let mut errors = vec![];

    let (output, parsing_errors) = {
        let token_iter = lexer.map(|(tok, loc)| match tok {
            Ok(tok) => (tok, loc),
            Err(err) => {
                errors.push(err.into());
                (Token::Err, loc)
            }
        });

        let lines_number = text.lines().count() + 1;
        let eof_offset = Offset {
            lines: lines_number,
            columns: 0,
        };
        let eof_location = Location::point(eof_offset);

        let token_stream = Stream::from_iter(token_iter).map(eof_location, |(t, l)| (t, l));
        let parser = parser::file_parser();

        parser.parse(token_stream).into_output_errors()
    };
    errors.extend(parsing_errors.into_iter().map(Into::into));

    (output, errors)
}

/// Convert CST to AST.
#[must_use]
pub fn convert_to_ast(
    tree: &Tree,
) -> Module<Location<Offset>, LocatedName<InternedString, Offset>> {
    convert(tree)
}
