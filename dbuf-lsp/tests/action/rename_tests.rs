//! Tests for `textDocument/rename` and `textDocument/prepareRename`.
//!

use crate::common::*;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use super::HandlerType;
use super::get_handler;

struct Scenario {
    h: HandlerType,
}

impl Scenario {
    fn new() -> Scenario {
        Scenario { h: get_handler() }
    }

    fn prepare_rename(&self, p: Position) -> Result<Option<PrepareRenameResponse>> {
        self.h.prepare_rename(&TEST_WORKSPACE, p, &TEST_URL)
    }

    fn rename(&self, p: Position, to: &str) -> Result<Option<WorkspaceEdit>> {
        self.h.rename(&TEST_WORKSPACE, to, p, &TEST_URL)
    }

    fn expect_valid_prepare_rename(&self, p: Position) {
        let r = self.prepare_rename(p);
        assert!(r.is_ok(), "expect prepare rename raise no error at {p:?}");
        assert!(r.unwrap().is_some(), "expect valid prepare rename at {p:?}");
    }

    fn expect_invalid_prepare_rename(&self, p: Position) {
        let r = self.prepare_rename(p);
        assert!(r.is_ok(), "expect prepare rename raise no error at {p:?}");
        assert!(
            r.unwrap().is_none(),
            "expect invalid prepare rename at {p:?}"
        );
    }

    fn expect_valid_rename(&self, p: Position, to: &str) -> DocumentChanges {
        let r = self.rename(p, to);
        assert!(r.is_ok(), "expect rename raise no error at {p:?}");
        let r = r.unwrap();
        assert!(r.is_some(), "expect rename modification at {p:?}");
        let r = r.unwrap();
        assert!(
            r.document_changes.is_some(),
            "expect rename modification at {p:?}"
        );
        r.document_changes.unwrap()
    }

    fn expect_invalid_rename(&self, p: Position, to: &str) {
        let r = self.rename(p, to);
        assert!(
            r.is_err(),
            "expect rename error at {p:?}, while renaming to '{to}'"
        );
    }
}

const fn get_position(line: u32, char: u32) -> Position {
    Position {
        line,
        character: char,
    }
}

const VALID_TYPE_RENAME: &[Position] = &[
    get_position(0, 9),   // messagae in declaration
    get_position(25, 6),  // enum in declaration
    get_position(6, 25),  // messgae in dependency Type declaration
    get_position(9, 7),   // message in field Type declaration
    get_position(28, 15), // messgae in field Type declaration of constructor
    get_position(48, 37), // messgae constructor call in other constructor call
    get_position(42, 29), // enum in dependency type declaration
    get_position(47, 14), // enum in field type declaration
    get_position(22, 14), // message in constructor call
    get_position(53, 6),  // message in pattern match
    get_position(27, 12), // constructor declaratino
    get_position(49, 23), // constructor in constructor call
];

const VALID_FIELD_RENAME: &[Position] = &[
    get_position(0, 13),  // dependency of message
    get_position(6, 29),  // dependency in type expression
    get_position(25, 14), // dependency of enum
    get_position(1, 6),   // field of message
    get_position(28, 13), // field of constructor
    get_position(9, 14),  // field in type expression
    get_position(18, 14), // field in access chain
    get_position(18, 20), // field in start of access chain
    get_position(47, 35), // field in constructor expr
    get_position(22, 23), // argument in constructor expr
    get_position(53, 7),  // argument in pattern expr
    get_position(26, 6),  // alias declaration
    get_position(28, 20), // alias usage
    get_position(53, 15), // alias in pattern
];

const INVALID_RENAME: &[Position] = &[
    get_position(0, 4),   // message keyword
    get_position(26, 0),  // enum keyword
    get_position(0, 17),  // type Int
    get_position(0, 27),  // type String
    get_position(22, 10), // int literal
    get_position(6, 32),  // string literal
    get_position(4, 1),   // '}'
    get_position(5, 0),   // empty
];

#[test]
fn test_valid_prepare() {
    let s = Scenario::new();

    VALID_TYPE_RENAME
        .iter()
        .chain(VALID_FIELD_RENAME)
        .for_each(|p| s.expect_valid_prepare_rename(*p));
}

#[test]
fn test_invalid_prepare() {
    let s = Scenario::new();

    INVALID_RENAME
        .iter()
        .for_each(|p| s.expect_invalid_prepare_rename(*p));
}

#[test]
fn test_valid_rename() {
    let s = Scenario::new();

    for p in VALID_TYPE_RENAME {
        s.expect_valid_rename(*p, "Name");
    }

    for p in VALID_FIELD_RENAME {
        s.expect_valid_rename(*p, "name");
    }
}

#[test]
fn test_invalid_rename() {
    let s = Scenario::new();

    for p in VALID_TYPE_RENAME {
        s.expect_invalid_rename(*p, ""); // to empty
        s.expect_invalid_rename(*p, "Int"); // to builtin
        s.expect_invalid_rename(*p, "M1"); // to existing / old
        s.expect_invalid_rename(*p, "m1"); // to bad format
    }

    let int = get_position(0, 17); // type Int
    s.expect_invalid_rename(int, "Name"); // of builtin

    let field = get_position(1, 5); // field f1 of M1
    s.expect_invalid_rename(field, "f1"); // to old
    s.expect_invalid_rename(field, "f3"); // to existing
    s.expect_invalid_rename(field, "d1"); // to existing

    let alias = get_position(28, 20); // alias of Simple
    s.expect_invalid_rename(alias, "alias"); // to old
    s.expect_invalid_rename(alias, "f1"); // to existing
    s.expect_invalid_rename(alias, "d1"); // to existing

    for p in VALID_FIELD_RENAME {
        s.expect_invalid_rename(*p, ""); // to empty
        s.expect_invalid_rename(*p, "message"); // to keyword
        s.expect_invalid_rename(*p, "M1"); // to bad format
    }

    for p in INVALID_RENAME {
        s.expect_invalid_rename(*p, ""); // to empty
        s.expect_invalid_rename(*p, "Int"); // to builtin
        s.expect_invalid_rename(*p, "message"); // to keyword
        s.expect_invalid_rename(*p, "m1"); // bad position
        s.expect_invalid_rename(*p, "M1"); // bad position
    }
}

#[test]
fn test_cahce_hit() {
    let s = Scenario::new();

    for p in VALID_TYPE_RENAME {
        s.expect_valid_prepare_rename(*p);
        s.expect_valid_rename(*p, "Name");
    }

    for p in VALID_FIELD_RENAME {
        s.expect_valid_prepare_rename(*p);
        s.expect_valid_rename(*p, "name");
    }
}

#[test]
fn test_cache_miss() {
    let s = Scenario::new();
    let evil = get_position(0, 8); // message M1

    for p in VALID_TYPE_RENAME {
        s.expect_valid_prepare_rename(*p);
        s.expect_valid_prepare_rename(evil);
        s.expect_valid_rename(*p, "Name");
    }

    for p in VALID_FIELD_RENAME {
        s.expect_valid_prepare_rename(*p);
        s.expect_valid_prepare_rename(evil);
        s.expect_valid_rename(*p, "name");
    }
}
