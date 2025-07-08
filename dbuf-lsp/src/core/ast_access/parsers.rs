//! Module exports:
//! * `get_parsed()` function, wich parses text to parsed ast.
//! * `get_elaborated()` function, wich parses text to elaborated ast.
//!

use super::ElaboratedAst;
use super::ParsedAst;

use crate::core::default_ast::default_elaborated_ast;
use crate::core::default_ast::default_parsed_ast;

/// Builds `ParsedAst` based on `text`.
pub fn get_parsed(_text: &str) -> ParsedAst {
    default_parsed_ast()
}

/// Builds `ElaboratedAst` based of `text`.
pub fn get_elaborated(_text: &str) -> ElaboratedAst {
    default_elaborated_ast()
}
