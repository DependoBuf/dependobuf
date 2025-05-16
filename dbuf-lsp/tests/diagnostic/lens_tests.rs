//! Tests for `testDocument/codeLens`.
//!

use crate::common::*;

use super::get_handler;

struct ExpectCodeLens {
    line: u32,
    ref_count: u32,
}

const fn expect_code_lens(line: u32, ref_count: u32) -> ExpectCodeLens {
    ExpectCodeLens { line, ref_count }
}

const CODE_LENS: &[ExpectCodeLens] = &[
    expect_code_lens(0, 13),
    expect_code_lens(6, 2),
    expect_code_lens(13, 0),
    expect_code_lens(21, 0),
    expect_code_lens(25, 1),
    expect_code_lens(42, 3),
    expect_code_lens(45, 0),
    expect_code_lens(52, 0),
];

#[test]
fn test_code_lens() {
    let h = get_handler();

    let resp = h.code_lens(&TEST_WORKSPACE, &TEST_URL);

    assert!(resp.is_ok(), "code lens raise no error");
    let resp = resp.unwrap();

    assert!(resp.is_some(), "code lens calculated");
    let resp = resp.unwrap();

    assert!(resp.len() == CODE_LENS.len(), "no extra code lens");

    for lens in resp.into_iter() {
        let line = lens.range.start.line;

        let expect_ref = CODE_LENS
            .iter()
            .find(|c| c.line == line)
            .map(|c| c.ref_count);

        assert!(expect_ref.is_some(), "unknow code lens at line {line}");
        let expect_ref = expect_ref.unwrap();

        let expect_text = format!("{expect_ref} references").to_string();

        let command = lens.command;

        assert!(command.is_some(), "lens at line {line} have command");
        let command = command.unwrap();

        assert!(
            expect_text == command.title,
            "bad lens text at {line}:\nexpect: {expect_text},\ngot: {}",
            command.title
        );
    }
}
