//! Tests for `textDocument/documentHighligh` and `textDocument/references`.
//!

use crate::common::*;

use tower_lsp::lsp_types::{Position, Range};

use super::get_handler;

const fn get_range(line: u32, character: u32, len: u32) -> Range {
    Range {
        start: Position { line, character },
        end: Position {
            line,
            character: character + len,
        },
    }
}

const M1_RANGES: &[Range] = &[
    get_range(0, 8, 2),
    get_range(6, 24, 2),
    get_range(9, 7, 2),
    get_range(10, 7, 2),
    get_range(16, 7, 2),
    get_range(18, 7, 2),
    get_range(22, 12, 2),
    get_range(28, 15, 2),
    get_range(33, 15, 2),
    get_range(46, 7, 2),
    get_range(48, 36, 2),
    get_range(52, 22, 2),
    get_range(53, 4, 2),
    get_range(55, 15, 2),
];

const M1F1_RANGES: &[Range] = &[
    get_range(1, 4, 2),
    get_range(10, 13, 2),
    get_range(22, 15, 2),
    get_range(48, 39, 2),
    get_range(53, 7, 2),
];

const M2F3_RANGES: &[Range] = &[
    get_range(9, 4, 2),
    get_range(10, 10, 2),
    get_range(18, 22, 2),
];

const M2D1_RANGES: &[Range] = &[
    get_range(6, 12, 2),
    get_range(6, 27, 2),
    get_range(9, 10, 2),
];

const SIMPLE_RANGES: &[Range] = &[get_range(25, 5, 6), get_range(42, 27, 6)];

const LITERALED_RANGES: &[Range] = &[get_range(32, 8, 9), get_range(48, 22, 9)];

const ALIAS_RANGES: &[Range] = &[get_range(53, 11, 5), get_range(55, 18, 5)];

const RANGES: &[&[Range]] = &[
    M1_RANGES,
    M1F1_RANGES,
    M2F3_RANGES,
    M2D1_RANGES,
    SIMPLE_RANGES,
    LITERALED_RANGES,
    ALIAS_RANGES,
];

fn check(target: &[Range], got: &[Range], comment: String) {
    for (exp, cur) in target.iter().zip(got) {
        assert!(
            exp == cur,
            "ranges not match:\nExpect:\n{target:#?},\nGot:\n{got:#?},\n{comment}"
        );
    }
}

fn check_ranges(ranges: &[Range]) {
    let h = get_handler();

    for r in ranges {
        let len = r.end.character - r.start.character;
        for i in 0..=len {
            let mut p = r.start;
            p.character += i;

            let resp = h.document_highlight(&TEST_WORKSPACE, p, &TEST_URL);

            assert!(resp.is_ok(), "document highlight raises no error");
            let resp = resp.unwrap();

            assert!(resp.is_some(), "document hightling generated");
            let resp: Vec<_> = resp.unwrap().into_iter().map(|d| d.range).collect();

            check(
                ranges,
                resp.as_ref(),
                format!("highlight mismatch at {p:?}").to_string(),
            );

            let resp = h.references(&TEST_WORKSPACE, p, &TEST_URL);

            assert!(resp.is_ok(), "references raises no error");
            let resp = resp.unwrap();

            assert!(resp.is_some(), "references generated");
            let resp: Vec<_> = resp.unwrap().into_iter().map(|l| l.range).collect();

            check(
                ranges,
                resp.as_ref(),
                format!("references mismatch at {p:?}").to_string(),
            );
        }
    }
}

#[test]
fn test_ranges() {
    for range in RANGES {
        check_ranges(range);
    }
}

#[test]
fn test_fuzzing() {
    let h = get_handler();
    for line in 0..60 {
        for character in 0..60 {
            let pos = Position { line, character };
            let _ = h.document_highlight(&TEST_WORKSPACE, pos, &TEST_URL);
            let _ = h.references(&TEST_WORKSPACE, pos, &TEST_URL);
        }
    }
}
