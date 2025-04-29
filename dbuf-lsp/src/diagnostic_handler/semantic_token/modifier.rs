//! Module provides enum Modifier - enum of modifiers lsp uses to
//! response to `textDocument/semantic` request.
//!

use tower_lsp::lsp_types::*;

#[derive(Debug)]
pub enum Modifier {
    Declaration,
}

impl Modifier {
    pub const COUNT: u32 = 1;

    pub fn to_index(&self) -> u32 {
        match &self {
            Modifier::Declaration => 0,
        }
    }
    pub fn from_index(index: u32) -> Modifier {
        match index {
            0 => Modifier::Declaration,
            _ => panic!("bad modifier index"),
        }
    }
    pub fn to_lsp(&self) -> SemanticTokenModifier {
        match &self {
            Modifier::Declaration => SemanticTokenModifier::DECLARATION,
        }
    }
}

pub fn get_all_modifiers() -> Vec<SemanticTokenModifier> {
    let mut ans = Vec::with_capacity(Modifier::COUNT as usize);
    for i in 0..Modifier::COUNT {
        ans.push(Modifier::from_index(i).to_lsp());
    }
    ans
}
