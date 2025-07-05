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

use std::{cell::RefCell, rc::Rc};

use chumsky::{error::Rich, input::*, span::Span as _, Parser};
use lexer::Token;
use logos::Logos;
use parser::create_parser;

use crate::ast::parsed::{
    location::{Location, Offset},
    *,
};

pub mod lexer;
pub mod parser;

pub fn parse<'src>(
    input: &'src str,
) -> Result<Module<Location<Offset>, String>, Vec<Rich<'src, Token, Location<Offset>>>> {
    let extra = Rc::new(RefCell::new(vec![0]));

    let lexer = Token::lexer_with_extras(input, extra.clone());

    let extra_clone = extra.clone();
    let token_iter = lexer.spanned().map(move |(tok, span)| {
        let start = calc_offset(extra_clone.clone(), span.start);
        let end = calc_offset(extra_clone.clone(), span.end);
        let loc = Location::new((), start..end);

        match tok {
            Ok(tok) => (tok, loc),
            Err(()) => (Token::Error, loc),
        }
    });

    let token_stream = Stream::from_iter(token_iter.clone()).map(
        Location::new(
            (),
            calc_offset(extra.clone(), input.len())..calc_offset(extra.clone(), input.len()),
        ),
        |(t, s): (_, _)| (t, s),
    );

    create_parser().parse(token_stream).into_result()
}

fn calc_offset(newlines: Rc<RefCell<Vec<usize>>>, pos: usize) -> Offset {
    let newlines = newlines.borrow();

    let lines = newlines.binary_search(&pos).unwrap_or_else(|i| i - 1);
    Offset {
        lines,
        columns: pos - newlines[lines],
    }
}
