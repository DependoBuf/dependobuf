//! Module exports:
//! * `get_parsed()` function, which parses text to parsed ast.
//! * `get_elaborated()` function, which parses text to elaborated ast.
//!

use super::Cst;
use super::ElaboratedAst;
use super::ParsedAst;

use crate::core::default_ast::default_elaborated_ast;
use crate::core::errors::Error;

use dbuf_core::cst::convert_to_ast;
use dbuf_core::cst::parse_to_cst;

/// Builds `CST` based on `text`.
pub fn get_cst(text: &str) -> (Option<Cst>, Vec<Error>) {
    let (cst, err) = parse_to_cst(text);
    (cst, err.into_iter().map(Into::into).collect())
}

/// Builds `ParsedAst` based on `CST`.
pub fn get_parsed(cst: &Cst) -> (Option<ParsedAst>, Vec<Error>) {
    let ast = convert_to_ast(cst);
    (Some(ast), vec![])
}

/// Builds `ElaboratedAst` based on `ParsedAst`.
pub fn get_elaborated(_past: &ParsedAst) -> (Option<ElaboratedAst>, Vec<Error>) {
    let res = default_elaborated_ast();
    // (None, vec![])
    (Some(res), vec![])
}
