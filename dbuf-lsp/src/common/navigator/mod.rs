//! Helps with navigation in ParsedAst, based on ElaboratedAst.
//!

mod find_definitions;
mod find_symbols;
mod find_type;
mod indentify;

use find_type::find_type_impl;
use tower_lsp::lsp_types::{Position, Range};

use crate::common::ast_access::{ElaboratedAst, File, ParsedAst};

use find_definitions::find_definition_impl;
use find_symbols::find_symbols_impl;
use indentify::get_symbol_impl;

type Str = String;

/// Symbol specification in dbuf file.
#[derive(Debug, Clone)]
pub enum Symbol {
    Type(Str),
    Dependency {
        t: Str,
        dependency: Str,
    },
    Field {
        t: Str,
        constructor: Str,
        field: Str,
    },
    Constructor(Str),
    None,
}

/// Tuple of parsed and elaborated ast with method for navigation.
pub struct Navigator<'a> {
    parsed: &'a ParsedAst,
    elaborated: &'a ElaboratedAst,
}

impl Navigator<'_> {
    /// Creates navigator for file.
    pub fn new(file: &File) -> Navigator {
        Navigator {
            parsed: file.get_parsed(),
            elaborated: file.get_elaborated(),
        }
    }

    /// Returns symbol in `pos`.
    pub fn get_symbol(&self, pos: Position) -> Symbol {
        get_symbol_impl(self, pos)
    }

    /// Finds all locations of `symbol`.
    pub fn find_symbols(&self, symbol: &Symbol) -> Vec<Range> {
        find_symbols_impl(self, symbol)
    }

    /// Finds definition of `symbol`.
    pub fn find_definition(&self, symbol: &Symbol) -> Option<Range> {
        find_definition_impl(self, symbol)
    }

    /// Finds type of `symbol`.
    pub fn find_type(&self, symbol: &Symbol) -> Symbol {
        find_type_impl(self, symbol)
    }
}
