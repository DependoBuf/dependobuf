//! Module provides enum Modifier - enum of modifiers lsp uses to
//! response to `textDocument/semantic` request.
//!

use tower_lsp::lsp_types::*;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

/// FIXME when more Modifier variants appear: Use `EnumSet` instead.
#[derive(Debug, EnumIter, PartialEq, Eq, Clone, Copy)]
pub enum Modifier {
    Declaration,
}

impl Modifier {
    pub fn to_index(self) -> u32 {
        self as u32
    }
    pub fn to_lsp(self) -> SemanticTokenModifier {
        match self {
            Modifier::Declaration => SemanticTokenModifier::DECLARATION,
        }
    }
}

pub fn get_all_modifiers() -> Vec<SemanticTokenModifier> {
    Modifier::iter().map(Modifier::to_lsp).collect()
}
