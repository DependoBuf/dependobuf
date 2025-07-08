//! Module exports:
//! * type `ParsedAst` - alias for parsed ast wich implements `ParsedHelper`
//!

use dbuf_core::ast::parsed;

use super::Loc;
use super::Str;

pub type ParsedAst = parsed::Module<Loc, Str>;
