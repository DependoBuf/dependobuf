//! Tests for `textDocument/semanticToken/full`.
//!
use crate::common::*;

use tower_lsp::lsp_types::SemanticTokensResult;

use super::get_handler;

use insta::{Settings, assert_debug_snapshot};

#[test]
fn test_semantic_token() {
    let h = get_handler();
    let r = h.semantic_tokens_full(&TEST_WORKSPACE, &TEST_URL);

    assert!(r.is_ok(), "semantic tokens raises no error");
    let r = r.unwrap();

    assert!(r.is_some(), "semnatic tokens are generated");
    let r = r.unwrap();

    let SemanticTokensResult::Tokens(t) = r else {
        panic!("Semantic tokes full returned not full response");
    };

    assert!(t.data.len() > 100, "response is too small");

    let mut settings = Settings::new();
    settings.set_snapshot_path("snapshots");
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_suffix("");
    settings.bind(move || {
        assert_debug_snapshot!("correct_semantic", t);
    });
}
