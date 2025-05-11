//! Helps with navigation in ParsedAst, based on ElaboratedAst.
//!

mod find_symbols;
mod indentify;

use tower_lsp::lsp_types::{Position, Range};

use crate::core::ast_access::{ElaboratedAst, File, ParsedAst};

use find_symbols::find_symbols_impl;
use indentify::get_symbol_impl;

type Str = String;

/// Symbol specification in dbuf file.
#[derive(Debug, Clone)]
pub enum Symbol {
    Type {
        type_name: Str,
    },
    Dependency {
        type_name: Str,
        dependency: Str,
    },
    Field {
        type_name: Str,
        constructor: Str,
        field: Str,
    },
    Alias {
        type_name: Str,
        branch_id: usize,
        alias: Str,
    },
    Constructor {
        type_name: Str,
        constructor: Str,
    },
    None,
}

/// Tuple of parsed and elaborated ast with method for navigation.
pub struct Navigator<'a> {
    pub parsed: &'a ParsedAst,
    pub elaborated: &'a ElaboratedAst,
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
}
