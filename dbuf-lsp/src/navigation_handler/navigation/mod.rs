mod find_definition;
mod find_type;

use tower_lsp::lsp_types::Range;

use crate::core::navigator::Navigator;
use crate::core::navigator::Symbol;

use find_definition::find_definition_impl;
use find_type::find_type_impl;

pub fn find_definition(navigator: &Navigator, symbol: &Symbol) -> Option<Range> {
    find_definition_impl(navigator, symbol)
}

pub fn find_type(navigator: &Navigator, symbol: Symbol) -> Symbol {
    find_type_impl(navigator, symbol)
}
