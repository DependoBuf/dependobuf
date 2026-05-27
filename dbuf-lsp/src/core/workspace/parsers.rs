//! Module exports:
//! * `get_parsed()` function, which parses text to parsed ast.
//! * `get_elaborated()` function, which parses text to elaborated ast.
//!

use super::Cst;
use super::ElaboratedAst;
use super::ParsedAst;

use crate::core::errors::Error;

use dbuf_core::cst::convert_to_ast;
use dbuf_core::cst::parse_to_cst;
use dbuf_core::elaboration::elaborate;

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
pub fn get_elaborated(past: &ParsedAst) -> (Option<ElaboratedAst>, Vec<Error>) {
    match elaborate(past) {
        Ok(east) => (Some(east), vec![]),
        Err(err) => (None, vec![err.into()]),
    }
}
