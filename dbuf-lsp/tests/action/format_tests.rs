//! Tests for `textDocument/format`.
//!
//! Currently only checks that parsed+elaborated asts are built for canon sample. Changing it should
//! result in reviewing all other handlers tests (FIXME when parser is ready).
//!

use crate::common::*;

use std::collections::HashMap;
use std::fs::read_to_string;

use tower_lsp::lsp_types::FormattingOptions;

use super::get_handler;

const CANON_FILE_PATH: &str = "./tests/sample.dbuf";

fn get_canon_strign() -> String {
    read_to_string(CANON_FILE_PATH).expect("file is exist")
}

#[test]
fn canon_formatting() {
    let h = get_handler();

    let options = FormattingOptions {
        tab_size: 4,
        insert_spaces: true,
        properties: HashMap::new(),
        trim_trailing_whitespace: None,
        insert_final_newline: None,
        trim_final_newlines: None,
    };

    let res = h.formatting(&TEST_WORKSPACE, &options, &TEST_URL);

    if let Ok(Some(edits)) = res {
        assert!(edits.len() == 1);

        let edit = edits.first().unwrap();
        assert!(edit.new_text == get_canon_strign());
    } else {
        panic!("bad result for canon formatting:\n{res:#?}");
    }
}
