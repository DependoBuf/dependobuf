//! Module provides enum Modifier - enum of modifiers lsp uses to
//! response to `textDocument/semantic` request.
//!

use std::sync::LazyLock;

use tower_lsp::lsp_types::*;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, PartialEq, Eq, Clone, Copy)]
pub enum Modifier {
    Declaration,
}

static MODIFIERS_ORDER: LazyLock<Vec<Modifier>> = LazyLock::new(|| Modifier::iter().collect());

impl Modifier {
    pub fn to_index(self) -> u32 {
        MODIFIERS_ORDER.iter().position(|m| *m == self).unwrap() as u32
    }
    pub fn to_lsp(self) -> SemanticTokenModifier {
        match self {
            Modifier::Declaration => SemanticTokenModifier::DECLARATION,
        }
    }
}

pub fn get_all_modifiers() -> Vec<SemanticTokenModifier> {
    MODIFIERS_ORDER.iter().map(|m| m.to_lsp()).collect()
}
