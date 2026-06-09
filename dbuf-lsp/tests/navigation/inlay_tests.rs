//! Tests for `textDocument/inlayHint`.
use std::fmt::Write;

use tower_lsp::lsp_types::*;

use crate::common::*;

use super::HandlerType;
use super::get_handler;

use pretty_assertions::assert_eq;

/// Scenario of test.
struct Scenario {
    /// Testing workspace.
    workspace: WorkspaceAccess,
    /// Testing handler.
    handler: HandlerType,
}

impl Default for Scenario {
    fn default() -> Self {
        let workspace = empty_workspace();
        let handler = get_handler();
        Self { workspace, handler }
    }
}

impl Scenario {
    /// Run tests for file. Expect every location in file to produce hint with
    /// labels from `expected` in correct order. Runs inlay hint request
    /// with provided range.
    #[track_caller]
    fn check_inlay_hint(&self, file: &FileMetadata, expected: &[&str], range: Range) {
        let url = get_url(file);
        workspace_with_open_file(&self.workspace, file);
        check_file(&self.workspace, file);

        assert_eq!(
            file.locations().len(),
            expected.len(),
            "Wrong amount of locations for:\n{file:#?}"
        );

        let result = self.handler.inlay_hint(&self.workspace, range, &url);
        let Ok(result) = result else {
            panic!("Server error while processing inlay hint: {result:?}");
        };

        let Some(hints) = result else {
            assert!(
                file.locations().is_empty(),
                "Got no hints, while expected some for:\n{file:#?}"
            );
            return;
        };

        let expect_pos = file.locations().iter().map(|l| l.start).collect::<Vec<_>>();

        let mut extra = vec![];
        let mut missing = vec![];
        for hint in &hints {
            if !expect_pos.contains(&hint.position) {
                extra.push(hint);
            }
        }

        for expected in &expect_pos {
            if !hints.iter().any(|h| h.position == *expected) {
                missing.push(expected);
            }
        }

        if !extra.is_empty() || !missing.is_empty() {
            let mut msg = format!("Wrong hints for:\n{file:#?}");
            if !extra.is_empty() {
                write!(&mut msg, "Extra hints at:\n{extra:#?}\n").unwrap();
            }
            if !missing.is_empty() {
                write!(&mut msg, "Missing hints at:\n{missing:#?}\n").unwrap();
            }
            panic!("{msg}");
        }

        for (hint, expect) in hints.iter().zip(expected) {
            let InlayHintLabel::String(actual) = &hint.label else {
                panic!("Unsupported hint label type. Hint:\n{hint:#?}\nFile:\n{file:#?}");
            };

            assert_eq!(
                actual, expect,
                "Wrong hint label. Hint:\n{hint:#?}\nFile:\n{file:#?}"
            );
        }

        workspace_with_close_file(&self.workspace, file);
    }
}

fn line_range(from: u32, to: u32) -> Range {
    Range {
        start: Position {
            line: from,
            character: 0,
        },
        end: Position {
            line: to,
            character: 0,
        },
    }
}

fn whole_range() -> Range {
    line_range(0, 1000)
}

#[test]
fn test_builtin_constructor() {
    const TEXT: &str = r#"
      |message Constructor {
      |  f1 Int;
      |  f2 String;
      |  f3 Bool;
      |  f4 Int;
      |}
      |
      |message CDep (d Constructor) {}
      |
      |message Test {
      |  f CDep Constructor{f1: 0, f2: "a", f3: true, f4: 0};
      |                        ^      ^        ^         ^         <---
      |}
    "#;

    const HINTS: &[&str] = &["Int", "String", "Bool", "Int"];

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_inlay_hint(&meta, HINTS, whole_range());
}

#[test]
fn test_builtin_pattern() {
    const TEXT: &str = r"
      |message Pattern {
      |  f1 Int;
      |  f2 String;
      |  f3 Bool;
      |  f4 Int;
      |}
      |
      |enum Enum (c Pattern) {
      |  Pattern{f1: a, f2: b, f3: c, f4: d} => {
      |             ^      ^      ^      ^         <---
      |    Ctr{}
      |  }
      |}
    ";

    const HINTS: &[&str] = &["Int", "String", "Bool", "Int"];

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_inlay_hint(&meta, HINTS, whole_range());
}

#[test]
fn test_user_constructor() {
    const TEXT: &str = r"
      |message User1 {}
      |enum Another (i Int) {
      |  * => {
      |    Ctr {}
      |  }
      |}
      |
      |message Constructor {
      |  f1 User1;
      |  f2 Another 0;
      |}
      |
      |message CDep (d Constructor) {}
      |
      |message Test {
      |  f CDep Constructor{f1: User1{}, f2: Ctr{}};
      |                        ^            ^         <---
      |}
    ";

    const HINTS: &[&str] = &["User1", "Another"];

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_inlay_hint(&meta, HINTS, whole_range());
}

#[test]
fn test_user_pattern() {
    const TEXT: &str = r"
      |message User1 {}
      |enum Another (i Int) {
      |  * => {
      |    Ctr {}
      |  }
      |}
      |
      |message Pattern {
      |  f1 User1;
      |  f2 Another 0;
      |}
      |
      |enum Enum (c Pattern) {
      |  Pattern{f1: User1{}, f2: Ctr{}} => {
      |             ^            ^             <---
      |    Ctr1{}
      |  }
      |}
    ";

    const HINTS: &[&str] = &["User1", "Another"];

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_inlay_hint(&meta, HINTS, whole_range());
}

#[test]
fn test_complex_constructor() {
    const TEXT: &str = r"
      |message User {
      |  f1 Int;
      |  f2 Bool;
      |}
      |message Constructor {
      |  f1 Int;
      |  f2 User;
      |  f3 Int;
      |}
      |
      |message CDep (d Constructor) {}
      |
      |message Test {
      |  f CDep Constructor{f1: 0, f2: User{f1: 0, f2: true}, f3: 0};
      |                        ^      ^        ^      ^          ^     <---
      |}
    ";

    const HINTS: &[&str] = &["Int", "User", "Int", "Bool", "Int"];

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_inlay_hint(&meta, HINTS, whole_range());
}

#[test]
fn test_complex_pattern() {
    const TEXT: &str = r"
      |message User {
      |  f1 Int;
      |  f2 Bool;
      |}
      |message Pattern {
      |  f1 Int;
      |  f2 User;
      |  f3 Int;
      |}
      |
      |enum Enum (p Pattern) {
      |  Pattern{f1: 0, f2: User{f1: 0, f2: true}, f3: 0} => {
      |             ^      ^        ^      ^          ^         <---
      |    Ctr{}
      |  }
      |}
    ";

    const HINTS: &[&str] = &["Int", "User", "Int", "Bool", "Int"];

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_inlay_hint(&meta, HINTS, whole_range());
}

#[test]
fn test_render() {
    {
        const TEXT: &str = r"
          |message Constructor {
          |  f1 Int;
          |  f2 Bool;
          |}
          |
          |message CDep (d Constructor) {}
          |
          |message Test1 {
          |  f1 CDep Constructor{f1: 0, f2: true};
          |  f2 CDep Constructor{f1: 0, f2: true};
          |}
        ";

        const HINTS: &[&str] = &[];
        let range = line_range(0, 8);

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_inlay_hint(&meta, HINTS, range);
    }

    {
        const TEXT: &str = r"
          |message Constructor {
          |  f1 Int;
          |  f2 Bool;
          |}
          |
          |message CDep (d Constructor) {}
          |
          |message Test1 {
          |  f1 CDep Constructor{f1: 0, f2: true};
          |                         ^      ^        <---
          |  f2 CDep Constructor{f1: 0, f2: true};
          |}
        ";

        const HINTS: &[&str] = &["Int", "Bool"];
        let range = line_range(0, 9);

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_inlay_hint(&meta, HINTS, range);
    }

    {
        const TEXT: &str = r"
          |message Constructor {
          |  f1 Int;
          |  f2 Bool;
          |}
          |
          |message CDep (d Constructor) {}
          |
          |message Test1 {
          |  f1 CDep Constructor{f1: 0, f2: true};
          |                         ^      ^        <---
          |  f2 CDep Constructor{f1: 0, f2: true};
          |                         ^      ^        <---
          |}
        ";

        const HINTS: &[&str] = &["Int", "Bool", "Int", "Bool"];
        let range = line_range(0, 10);

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_inlay_hint(&meta, HINTS, range);
    }
}
