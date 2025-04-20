//! Module provides enum Token - enum of tokens lsp uses to
//! response to `textDocument/semantic` request.
//!

use tower_lsp::lsp_types::*;

#[derive(Debug)]
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

impl Token {
    const COUNT: u32 = 10;

    pub fn to_index(&self) -> u32 {
        match &self {
            Token::Type => 0,
            Token::Message => 1,
            Token::Enum => 2,
            Token::Parameter => 3,
            Token::Property => 4,
            Token::EnumConstructor => 5,
            Token::Keyword => 6,
            Token::String => 7,
            Token::Number => 8,
            Token::Operator => 9,
        }
    }
    pub fn from_index(index: u32) -> Token {
        match index {
            0 => Token::Type,
            1 => Token::Message,
            2 => Token::Enum,
            3 => Token::Parameter,
            4 => Token::Property,
            5 => Token::EnumConstructor,
            6 => Token::Keyword,
            7 => Token::String,
            8 => Token::Number,
            9 => Token::Operator,
            _ => panic!("bad token index"),
        }
    }
    pub fn to_lsp(&self) -> SemanticTokenType {
        match &self {
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
    let mut ans = Vec::with_capacity(Token::COUNT as usize);
    for i in 0..Token::COUNT {
        ans.push(Token::from_index(i).to_lsp());
    }
    ans
}
