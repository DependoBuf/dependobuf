//! Module provides enum Modifier - enum of modifiers lsp uses to
//! response to `textDocument/semantic` request.
//!

use tower_lsp::lsp_types::*;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, PartialEq, Eq)]
pub enum Modifier {
    Declaration,
}

impl Modifier {
    pub fn to_index(self) -> u32 {
        Modifier::iter().position(|m| m == self).unwrap() as u32
    }
    pub fn to_lsp(&self) -> SemanticTokenModifier {
        match &self {
            Modifier::Declaration => SemanticTokenModifier::DECLARATION,
        }
    }
}

pub fn get_all_modifiers() -> Vec<SemanticTokenModifier> {
    Modifier::iter().map(|m| m.to_lsp()).collect()
}
