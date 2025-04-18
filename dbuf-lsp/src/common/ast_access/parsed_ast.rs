//! Module exports:
//! * trait ParsedHelper - helpfull getters for parsed ast.
//! * type ParsedAst - alias for parsed ast wich implements ParsedHelper
//!

use dbuf_core::ast::parsed;

use super::Loc;
use super::Str;

pub type ParsedAst = parsed::Module<Loc, Str>;

/// Trait with getters for ParsedAst.
pub trait ParsedHelper {}

impl ParsedHelper for ParsedAst {}
