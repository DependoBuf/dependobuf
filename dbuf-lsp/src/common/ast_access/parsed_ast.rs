//! Module exports:
//! * type ParsedAst
//!

use dbuf_core::ast::parsed;

use super::Loc;
use super::Str;

pub type ParsedAst = parsed::Module<Loc, Str>;
