mod lexer;

mod located_token;
mod location;

mod label;

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
use crate::error::parsing::ParsingStage;
use crate::location::LocatedName;

use crate::location::Location;
use crate::location::Offset;

pub use label::Label;
pub use lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum TreeKind {
    /// Contains unparsed tokens as child.
    ErrorTree,

    /// Contains whole file.
    File,

    /// Contains message definition.
    Message,
    /// Contains body of message / constructor.
    Body,
    /// Contains definion of field / dependency.
    Definition,

    /// Contains message definition.
    Enum,
    /// Contains body of enum defintion.
    EnumBody,
    /// Contains one branch definition.
    Branch,
    /// Contains one pattern.
    Pattern,
    /// Contains negative integer as pattern.
    NegativePattern,
    /// Contains constructed pattern.
    ConstructedPattern,
    /// Contains one field of constructed pattern.
    ConstructedPatternField,
    /// Contains all constructor definitions for current branch.
    ConstructorEnum,
    /// Contains one constructor definition.
    Constructor,

    /// Contains constructed value with dot accesses.
    ConstructedValueChain,
    /// Contains constructed value for expressions.
    ConstructedValue,
    /// Contains field of constructed value.
    ConstructedValueField,

    /// Contains parened expression.
    ExprParen,
    /// Contains literal as expression.
    ExprLiteral,
    /// Contains indentifier as expression.
    ExprIdentifier,
    /// Contains binary operation as expression.
    ExprBinary,
    /// Contains unary operation as expression.
    ExprUnary,
    /// Contains type hole as expression.
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
pub fn parse_to_cst(text: &str) -> (Option<Tree>, Vec<Error<ParsingStage>>) {
    let lexer = LocatedLexer::from_lexer(Token::lexer(text));

    let mut errors = vec![];

    let (output, parsing_errors) = {
        let token_iter = lexer.map(|(tok, loc)| match tok {
            Ok(tok) => (tok, loc),
            Err(err) => {
                let data = err.data.content.clone();
                errors.push(err.into());
                (Token::Err(data), loc)
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
