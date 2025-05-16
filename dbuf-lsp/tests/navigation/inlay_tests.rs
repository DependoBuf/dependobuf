//! Tests for `textDocument/inlayHint`
//!

use crate::common::*;

use tower_lsp::lsp_types::{InlayHintLabel, Position, Range};

use super::get_handler;

#[derive(Debug)]
struct ExpectedHint {
    at: Position,
    text: &'static str,
}

impl ExpectedHint {
    const fn new(line: u32, column: u32, text: &'static str) -> ExpectedHint {
        ExpectedHint {
            at: Position {
                line,
                character: column,
            },
            text,
        }
    }
}

const ALL_HINTS: &[ExpectedHint] = &[
    ExpectedHint::new(22, 18, "Int"),
    ExpectedHint::new(22, 25, "Int"),
    ExpectedHint::new(22, 32, "String"),
    ExpectedHint::new(47, 33, "M1"),
    ExpectedHint::new(48, 35, "M1"),
    ExpectedHint::new(48, 42, "Int"),
    ExpectedHint::new(48, 49, "Int"),
    ExpectedHint::new(48, 56, "String"),
    ExpectedHint::new(53, 10, "Int"),
    ExpectedHint::new(53, 21, "Int"),
    ExpectedHint::new(53, 28, "String"),
];

fn test_range(r: Range) {
    let h = get_handler();
    let hints = h.inlay_hint(&TEST_WORKSPACE, r, &TEST_URL);

    assert!(hints.is_ok(), "inlay hint raise no error (r={r:?})");
    let hints = hints.unwrap();

    assert!(hints.is_some(), "inlay hints created (r={r:?})");
    let hints = hints.unwrap();

    let expected_hints = ALL_HINTS
        .iter()
        .filter(|h| r.start <= h.at && h.at <= r.end);

    let mut count = 0;
    for hint in expected_hints {
        count += 1;
        assert!(
            hints.iter().any(|h| {
                if let InlayHintLabel::String(s) = &h.label {
                    h.position == hint.at && s == hint.text
                } else {
                    panic!("expected only string hints");
                }
            }),
            "Hint {hint:?} not found (r={r:?})"
        );
    }

    assert!(hints.len() == count, "no extra hints (r={r:?})");
}

#[test]
fn test_all_hints() {
    test_range(Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 100,
            character: 0,
        },
    });
}

#[test]
fn test_render_only_needed() {
    test_range(Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: 25,
            character: 0,
        },
    });
    test_range(Range {
        start: Position {
            line: 25,
            character: 0,
        },
        end: Position {
            line: 50,
            character: 0,
        },
    });
    test_range(Range {
        start: Position {
            line: 50,
            character: 0,
        },
        end: Position {
            line: 75,
            character: 0,
        },
    });
    test_range(Range {
        start: Position {
            line: 75,
            character: 0,
        },
        end: Position {
            line: 100,
            character: 0,
        },
    });
}
