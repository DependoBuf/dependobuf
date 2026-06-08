//! Tests for `textDocument/documentSymbol`.
//!
use insta::{Settings, assert_debug_snapshot};

use crate::common::*;

use tower_lsp::lsp_types::DocumentSymbolResponse;

use super::get_handler;

#[test]
fn test_document_symbol() {
    let h = get_handler();

    let r = h.document_symbol(&TEST_WORKSPACE, &TEST_URL);

    assert!(r.is_ok(), "document symbol raises no error");
    let r = r.unwrap();

    assert!(r.is_some(), "document symbol is generated");
    let r = r.unwrap();

    let DocumentSymbolResponse::Nested(n) = r else {
        panic!("document symbols generated not nested");
    };

    assert!(n.len() > 7, "not enough document symbols");
    assert!(n.len() < 13, "too many document symbols");

    let mut settings = Settings::new();
    settings.set_snapshot_path("snapshots");
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_suffix("");
    settings.bind(move || {
        assert_debug_snapshot!("correct_symbols", n);
    });
}
