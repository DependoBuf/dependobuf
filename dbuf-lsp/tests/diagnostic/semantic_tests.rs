//! Tests for `textDocument/semanticToken/full`.
//!

use crate::common::*;

use tower_lsp::lsp_types::SemanticTokensResult;

use super::get_handler;

#[test]
fn test_semantic_token() {
    let h = get_handler();
    let r = h.semantic_tokens_full(&TEST_WORKSPACE, &TEST_URL);

    assert!(r.is_ok(), "semantic tokens raises no error");
    let r = r.unwrap();

    assert!(r.is_some(), "semnatic tokens are generated");
    let r = r.unwrap();

    if let SemanticTokensResult::Tokens(t) = r {
        assert!(t.data.len() > 100, "response is too small");
    } else {
        panic!("Semantic tokes full returned not full response");
    }
}
