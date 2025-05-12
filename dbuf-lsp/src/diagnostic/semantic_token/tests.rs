use super::modifier;
use super::token;

use strum::IntoEnumIterator;

#[test]
fn test_tokens_consistency() {
    let tokens_order = token::get_all_tokens();

    for token in token::Token::iter() {
        let token_convertation = token.to_lsp();
        let token_index = token.to_index() as usize;
        let vector_convertation = tokens_order
            .get(token_index)
            .expect("tokens_order contains all tokens");

        assert!(token_convertation == *vector_convertation);
    }
}

#[test]
fn test_modifiers_consistency() {
    let modifiers_order = modifier::get_all_modifiers();

    for modifier in modifier::Modifier::iter() {
        let modifier_convertation = modifier.to_lsp();
        let modifier_index = modifier.to_index() as usize;
        let vector_convertation = modifiers_order
            .get(modifier_index)
            .expect("modifiers_order contains all tokens");

        assert!(modifier_convertation == *vector_convertation);
    }
}
