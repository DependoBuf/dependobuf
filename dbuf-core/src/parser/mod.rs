//! This module provides a parser, that converts &str to CST. For AST definition look module ast/parsed.
//!
//! # Ungrammar Syntax:
//!
//! ```text
//! = definition
//! | or
//! a? 0 or 1 occurrence of 'a'
//! a* 0 or more occurrences of 'a'
//! ```
//!
//! All lines ending with '_token' are tokens that need to be parsed separately in the tokenizer.
//! In general, all lines will be handled separately in the tokenizer, but this notation was introduced
//! to avoid overcomplicating the grammar of small building blocks that are simply tokenized.
//!
//! ```text
//! Module = TypeDefinition*
//!
//! TypeDefinition = MessageDef | EnumDef
//! MessageDef = 'message' TypeIdentifier Dependencies FieldsBlock
//! EnumDef = DependentEnumDef | IndependentEnumDef
//! DependentEnumDef = 'enum' TypeIdentifier Dependencies '{' MappingRule* '}'
//! IndependentEnumDef = 'enum' TypeIdentifier ConstructorsBlock
//!
//! Dependencies = '(' TypedVariable ')' ('(' TypedVariable ')')*
//!
//! MappingRule = InputPatterns '=>' ConstructorsBlock
//! InputPatterns = Pattern (',' Pattern)*
//!
//! Pattern = '*' | VarIdentifier | Value | ConstructedValue_pattern
//! ConstructedValue_pattern = ConstructorIdentifier '{' FieldInitList? '}'
//! FieldInitList_pattern = FieldInit_pattern (',' FieldInit_pattern)*
//! FieldInit_pattern = VarIdentifier ':' Pattern
//!
//! ConstructorsBlock = '{' ConstructorDeclaration* '}'
//! ConstructorDeclaration = ConstructorIdentifier FieldsBlock?
//! FieldsBlock = '{' FieldDeclaration* '}'
//! FieldDeclaration = TypedVariable ';'
//!
//! TypedVariable = VarIdentifier TypeExpr
//! TypeExpr = TypeIdentifier Primary*
//!
//! Expression = Expression BinaryOperation Expression | UnaryOperation Expression | Primary | TypeExpr // ????
//! BinaryOperation = '+' | '-' | '*' | '/' | '&' | '|'
//! UnaryOperation = '-' | '!'
//!
//! Primary = Value | VarAccess | ConstructedValue | '(' Expression ')' | UnaryOperation Primary
//!
//! ConstructedValue = ConstructorIdentifier '{' FieldInitList? '}'
//! FieldInitList = FieldInit (',' FieldInit)*
//! FieldInit = VarIdentifier ':' Expression
//!
//! VarAccess = VarIdentifier ('.' VarIdentifier)*
//! Value =
//!     BooleanLiteral
//!   | FloatLiteral
//!   | IntLiteral
//!   | UintLiteral
//!   | StringLiteral
//!
//! BooleanLiteral = 'true' | 'false'
//! IntLiteral = 'int_literal_token'
//! UintLiteral = 'uint_literal_token'
//! FloatLiteral = 'float_literal_token'
//! StringLiteral = 'string_literal_token'
//!
//! TypeIdentifier = 'UC_IDENTIFIER_token'
//! ConstructorIdentifier = 'UC_IDENTIFIER_token'
//! VarIdentifier = 'LC_IDENTIFIER_token'
//! ```

use chumsky::{error::Rich, input::*, span::SimpleSpan, Parser};
use lexer::Token;
use logos::Logos;
use parser_impl::create_parser;

use located_iter::*;

use crate::ast::parsed::{
    located_name::LocatedName,
    location::{Location, Offset},
    *,
};

pub mod lexer;
pub mod located_iter;
pub mod parser_impl;

type ParsedModule = Module<Location<Offset>, LocatedName<String, Offset>>;
type ParseError<'src> = Rich<'src, Token, SimpleSpan<Offset>>;

/// # Errors
///
/// TODO: explain when `Err` is returned.
pub fn parse(input: &str) -> Result<ParsedModule, Vec<ParseError<'_>>> {
    let lexer = Token::lexer(input);

    let token_iter = lexer.located().map(move |(tok, span)| match tok {
        Ok(tok) => (tok, span.into()),
        Err(()) => (Token::Err, span.into()),
    });

    let end_offset = Offset {
        lines: input.lines().count() + 1,
        columns: 0,
    };
    let eoi_loc = (end_offset..end_offset).into();

    let token_stream = Stream::from_iter(token_iter).map(eoi_loc, |(t, s): (_, _)| (t, s));

    create_parser().parse(token_stream).into_result()
}
