//! Tests for `textDocument/definition` and `textDocument/typeDefinition`.
//!

use crate::common::*;

use std::collections::HashMap;
use std::sync::LazyLock;

use tower_lsp::lsp_types::{GotoDefinitionResponse, Position, Range};

use super::get_handler;

fn get_range(line: u32, character: u32, len: u32) -> Range {
    Range {
        start: Position { line, character },
        end: Position {
            line,
            character: character + len,
        },
    }
}

fn get_definition_length(name: &str) -> u32 {
    let length = name.rfind(':').map_or(name.len(), |p| name.len() - p - 1);
    u32::try_from(length).unwrap()
}

fn get_definition(name: &'static str, line: u32, character: u32) -> (&'static str, Range) {
    (
        name,
        get_range(line, character, get_definition_length(name)),
    )
}

static DEFINITIONS: LazyLock<HashMap<&'static str, Range>> = LazyLock::new(|| {
    HashMap::from([
        get_definition("M1", 0, 8),
        get_definition("M1:d1", 0, 12),
        get_definition("M1:d2", 0, 21),
        get_definition("M1:f1", 1, 4),
        get_definition("M1:f2", 2, 4),
        get_definition("M1:f3", 3, 4),
        get_definition("M2", 6, 8),
        get_definition("M2:d1", 6, 12),
        get_definition("M2:d2", 6, 21),
        get_definition("M2:f1", 7, 4),
        get_definition("M2:f2", 8, 4),
        get_definition("M2:f3", 9, 4),
        get_definition("M2:f4", 10, 4),
        get_definition("M3", 13, 8),
        get_definition("M3:d1", 13, 12),
        get_definition("M3:f1", 14, 4),
        get_definition("M3:f2", 15, 4),
        get_definition("M3:f3", 16, 4),
        get_definition("M3:f4", 17, 4),
        get_definition("M3:f5", 18, 4),
        get_definition("Constructed", 21, 8),
        get_definition("Constructed:f1", 22, 4),
        get_definition("Simple", 25, 5),
        get_definition("Simple:d1", 25, 13),
        get_definition("Simple:alias", 26, 4),
        get_definition("Aliased", 27, 8),
        get_definition("Aliased:f1", 28, 12),
        get_definition("Literaled", 32, 8),
        get_definition("Literaled:f1", 33, 12),
        get_definition("Wild", 37, 8),
        get_definition("SimpleDepended", 42, 8),
        get_definition("SimpleDepended:d1", 42, 24),
        get_definition("EnumConstructed", 45, 8),
        get_definition("EnumConstructed:f1", 46, 4),
        get_definition("EnumConstructed:f2", 47, 4),
        get_definition("EnumConstructed:f3", 48, 4),
        get_definition("EnumConstructed:f4", 49, 4),
        get_definition("PatternMatch", 52, 5),
        get_definition("PatternMatch:d1", 52, 19),
        get_definition("PatternMatch:alias", 53, 11),
        get_definition("PatternMatched", 54, 8),
        get_definition("PatternMatched:f1", 55, 12),
    ])
});

#[derive(Debug)]
struct JumpingPoint {
    from: Position,
    to: &'static str,
    type_name: &'static str,
}

const fn get_jump(
    l_from: u32,
    c_from: u32,
    to: &'static str,
    type_name: &'static str,
) -> JumpingPoint {
    JumpingPoint {
        from: Position {
            line: l_from,
            character: c_from,
        },
        to,
        type_name,
    }
}
const JUMPS: &[JumpingPoint] = &[
    get_jump(6, 24, "M1", "M1"),
    get_jump(6, 27, "M2:d1", ""),
    get_jump(9, 7, "M1", "M1"),
    get_jump(9, 10, "M2:d1", ""),
    get_jump(9, 13, "M2:f2", ""),
    get_jump(10, 7, "M1", "M1"),
    get_jump(10, 10, "M2:f3", "M1"),
    get_jump(10, 13, "M1:f1", ""),
    get_jump(16, 7, "M1", "M1"),
    get_jump(16, 10, "M3:f1", ""),
    get_jump(17, 7, "M2", "M2"),
    get_jump(17, 10, "M3:f1", ""),
    get_jump(17, 13, "M3:f3", "M1"),
    get_jump(18, 7, "M1", "M1"),
    get_jump(18, 10, "M3:f4", "M2"),
    get_jump(18, 13, "M2:f4", "M1"),
    get_jump(18, 16, "M1:f2", ""),
    get_jump(18, 19, "M3:f4", "M2"),
    get_jump(18, 22, "M2:f3", "M1"),
    get_jump(18, 25, "M1:f3", ""),
    get_jump(22, 7, "M2", "M2"),
    get_jump(22, 12, "M1", "M1"),
    get_jump(22, 15, "M1:f1", ""),
    get_jump(22, 22, "M1:f2", ""),
    get_jump(22, 29, "M1:f3", ""),
    get_jump(28, 15, "M1", "M1"),
    get_jump(28, 18, "Simple:alias", ""),
    get_jump(33, 15, "M1", "M1"),
    get_jump(42, 27, "Simple", "Simple"),
    get_jump(46, 7, "M1", "M1"),
    get_jump(47, 7, "SimpleDepended", "SimpleDepended"),
    get_jump(47, 22, "Aliased", "Simple"),
    get_jump(47, 30, "Aliased:f1", "M1"),
    get_jump(47, 34, "EnumConstructed:f1", "M1"),
    get_jump(48, 7, "SimpleDepended", "SimpleDepended"),
    get_jump(48, 22, "Literaled", "Simple"),
    get_jump(48, 32, "Literaled:f1", "M1"),
    get_jump(48, 36, "M1", "M1"),
    get_jump(48, 39, "M1:f1", ""),
    get_jump(48, 46, "M1:f2", ""),
    get_jump(48, 53, "M1:f3", ""),
    get_jump(49, 7, "SimpleDepended", "SimpleDepended"),
    get_jump(49, 22, "Wild", "Simple"),
    get_jump(52, 22, "M1", "M1"),
    get_jump(53, 4, "M1", "M1"),
    get_jump(53, 7, "M1:f1", ""),
    get_jump(53, 18, "M1:f2", ""),
    get_jump(53, 25, "M1:f3", ""),
    get_jump(55, 15, "M1", "M1"),
    get_jump(55, 18, "PatternMatch:alias", ""),
];

fn check_jump(j: &JumpingPoint) {
    let h = get_handler();
    let size = get_definition_length(j.to);

    for i in 0..=size {
        let mut p = j.from;
        p.character += i;

        let r = h.goto_definition(&TEST_WORKSPACE, p, &TEST_URL);

        assert!(r.is_ok(), "goto definition raise no error");
        let r = r.unwrap();

        assert!(r.is_some(), "{j:?} is correct jump");
        let r = r.unwrap();

        if let GotoDefinitionResponse::Scalar(l) = r {
            let range = l.range;

            let expect = DEFINITIONS.get(j.to);

            assert!(expect.is_some(), "definition {:?} not found", j.to);
            let expect = expect.unwrap();

            assert!(
                range == *expect,
                "incorrect jump {p:?}:\n{j:#?}:\n  expect:\n{:#?},\n  got:\n{:#?}",
                *expect,
                range
            );
        } else {
            panic!("goto returned not scalar at jump {j:?}");
        }
    }
}

fn check_type_definition_jump(j: &JumpingPoint) {
    let h = get_handler();
    let size = get_definition_length(j.to);

    for i in 0..=size {
        let mut p = j.from;
        p.character += i;

        let r = h.goto_type_definition(&TEST_WORKSPACE, p, &TEST_URL);

        assert!(r.is_ok(), "goto definition raise no error");
        let r = r.unwrap();

        if j.type_name.is_empty() {
            assert!(r.is_none(), "{j:?} has no type");
            continue;
        }

        assert!(r.is_some(), "{j:?} is correct jump");
        let r = r.unwrap();

        if let GotoDefinitionResponse::Scalar(l) = r {
            let range = l.range;

            let expect = DEFINITIONS.get(j.type_name);

            assert!(expect.is_some(), "definition {:?} not found", j.type_name);
            let expect = expect.unwrap();

            assert!(
                range == *expect,
                "incorrect jump:\n{j:#?}:\n  expect:\n{:#?},\n  got:\n{:#?}",
                *expect,
                range
            );
        } else {
            panic!("goto returned not scalar at jump {j:?}");
        }
    }
}

#[test]
fn test_self_jump() {
    DEFINITIONS.iter().for_each(|f| {
        let jump = JumpingPoint {
            from: f.1.start,
            to: f.0,
            type_name: "",
        };
        check_jump(&jump);
    });
}

#[test]
fn test_jumps() {
    JUMPS.iter().for_each(check_jump);
}

#[test]
fn test_type_definition_jumps() {
    JUMPS.iter().for_each(check_type_definition_jump);
}
