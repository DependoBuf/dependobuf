mod lexer;

mod located_token;
mod location;

mod parser;
mod parser_error;
mod parser_utils;

mod cst_to_ast;

use chumsky::Parser;
use chumsky::input::{Input, Stream};
use logos::Logos;

use located_token::LocatedLexer;

use crate::ast::parsed::location::Offset;
use cst_to_ast::convert;

pub use chumsky::ParseResult;
pub use cst_to_ast::ParsedModule;
pub use lexer::{LexingError, LexingErrorData, LexingErrorKind, Token};
pub use location::Location;
pub use parser_error::{ExpectedPattern, ParsingError, ParsingErrorExtra};

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
    pub location: Location,
    pub children: Vec<Child>,
}

#[derive(Debug)]
pub enum Child {
    Token(Token, Location),
    Tree(Tree),
}

/// Parse text to CST.
#[must_use]
pub fn parse_to_cst(text: &str) -> ParseResult<Tree, ParsingError> {
    let lexer = LocatedLexer::from_lexer(Token::lexer(text));
    let token_iter = lexer.map(|(tok, loc)| match tok {
        Ok(tok) => (tok, loc),
        Err(err) => (Token::Err(err), loc),
    });

    let lines_number = text.lines().count() + 1;
    let eof_offset = Offset {
        lines: lines_number,
        columns: 0,
    };
    let eof_location = Location::point(eof_offset);

    let token_stream = Stream::from_iter(token_iter).map(eof_location, |(t, l)| (t, l));
    let parser = parser::file_parser();
    parser.parse(token_stream)
}

/// Convert CST to AST.
#[must_use]
pub fn convert_to_ast(tree: &Tree) -> ParsedModule {
    convert(tree)
}
