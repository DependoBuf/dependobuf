//! Tests for `textDocument/hover`.
//!

use crate::common::*;

use tower_lsp::lsp_types::Position;

use super::get_handler;

#[test]
fn test_fuzzing() {
    let h = get_handler();

    for line in 0..60 {
        for character in 0..60 {
            let pos = Position { line, character };
            let r = h.hover(&TEST_WORKSPACE, pos, &TEST_URL);
            assert!(r.is_ok(), "hover raises no error");
        }
    }
}
