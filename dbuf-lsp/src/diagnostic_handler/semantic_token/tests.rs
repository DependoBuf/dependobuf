use super::Modifier;
use super::Token;

#[test]
fn test_modifier_consistency() {
    for i in 0..Modifier::COUNT {
        let current = Modifier::from_index(i);
        assert!(Modifier::to_index(&current) == i);
    }
}

#[test]
fn test_modifier_convertible() {
    for i in 0..Modifier::COUNT {
        let current = Modifier::from_index(i);
        let _ = Modifier::to_lsp(&current);
    }
}

#[test]
fn test_token_consistency() {
    for i in 0..Token::COUNT {
        let current = Token::from_index(i);
        assert!(Token::to_index(&current) == i);
    }
}

#[test]
fn test_token_convertible() {
    for i in 0..Token::COUNT {
        let current = Token::from_index(i);
        let _ = Token::to_lsp(&current);
    }
}
