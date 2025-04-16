//! Helps find all instances of smth
//!

mod find_definitions;
mod find_symbols;
mod indentify;

use tower_lsp::lsp_types::{Position, Range};

use crate::common::ast_access::{ElaboratedAst, File, ParsedAst};

use find_definitions::find_definition_impl;
use find_symbols::find_symbols_impl;
use indentify::get_symbol_impl;

type Str = String;

#[derive(Debug, Clone)]
pub enum Symbol {
    Type(Str),
    Dependency { t: Str, dependency: Str },
    Field { constructor: Str, field: Str },
    Constructor(Str),
    None,
}

pub struct Navigator<'a> {
    parsed: &'a ParsedAst,
    elaborated: &'a ElaboratedAst,
}

impl Navigator<'_> {
    pub fn new<'a>(parsed: &'a ParsedAst, elaborated: &'a ElaboratedAst) -> Navigator<'a> {
        Navigator { parsed, elaborated }
    }
    pub fn for_file(file: &File) -> Navigator {
        Navigator {
            parsed: file.get_parsed(),
            elaborated: file.get_elaborated(),
        }
    }

    pub fn get_symbol(&self, pos: Position) -> Symbol {
        get_symbol_impl(self, pos)
    }

    pub fn find_symbols(&self, symbol: &Symbol) -> Vec<Range> {
        find_symbols_impl(self, symbol)
    }

    pub fn find_definition(&self, symbol: &Symbol) -> Option<Range> {
        find_definition_impl(self, symbol)
    }
}
