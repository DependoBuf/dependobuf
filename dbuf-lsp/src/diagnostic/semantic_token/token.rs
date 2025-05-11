//! Module provides enum Token - enum of tokens lsp uses to
//! response to `textDocument/semantic` request.
//!

use std::sync::LazyLock;

use tower_lsp::lsp_types::*;

use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Debug, EnumIter, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    Type,
    Message,
    Enum,
    Parameter,
    Property,
    EnumConstructor,
    Keyword,
    String,
    Number,
    Operator,
}

static TOKENS_ORDER: LazyLock<Vec<Token>> = LazyLock::new(|| Token::iter().collect());

impl Token {
    pub fn to_index(self) -> u32 {
        TOKENS_ORDER.iter().position(|t| *t == self).unwrap() as u32
    }
    pub fn to_lsp(self) -> SemanticTokenType {
        match self {
            Token::Type => SemanticTokenType::TYPE,
            Token::Message => SemanticTokenType::STRUCT,
            Token::Enum => SemanticTokenType::ENUM,
            Token::Parameter => SemanticTokenType::PARAMETER,
            Token::Property => SemanticTokenType::PROPERTY,
            Token::EnumConstructor => SemanticTokenType::ENUM_MEMBER,
            Token::Keyword => SemanticTokenType::new("storage.type"),
            Token::String => SemanticTokenType::STRING,
            Token::Number => SemanticTokenType::NUMBER,
            Token::Operator => SemanticTokenType::OPERATOR,
        }
    }
}

pub fn get_all_tokens() -> Vec<SemanticTokenType> {
    TOKENS_ORDER.iter().map(|t| t.to_lsp()).collect()
}
