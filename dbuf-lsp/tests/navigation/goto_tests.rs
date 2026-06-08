//! Tests for `textDocument/definition` and `textDocument/typeDefinition`.
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
    /// Run test of file. Check based on number of provided locations:
    ///   * 0 locations - expect every cursor to not produce goto,
    ///   * 1 locations - expeect every cursor to jump to that location.
    #[track_caller]
    fn check_goto(&self, file: &FileMetadata) {
        assert!(
            !file.cursors().is_empty(),
            "File cursors shouldn't be empty for file:\n{file:#?}"
        );
        assert!(
            file.locations().len() < 2,
            "Invalid location number for file. Expect 1 or 0: \n{file:#?}"
        );

        let url = get_url(file);
        workspace_with_open_file(&self.workspace, file);
        check_file(&self.workspace, file);

        for pos in file.cursors() {
            let result = self.handler.goto_definition(&self.workspace, *pos, &url);

            let Ok(result) = result else {
                panic!(
                    "Server error while processing goto jump at {pos:?}:\n{file:#?}\nError: {result:#?}"
                );
            };

            let Some(resp) = result else {
                if file.locations().is_empty() {
                    continue;
                }
                panic!(
                    "Expected to have goto location, but got nothing after processing goto jump at {pos:?}:\n{file:#?}"
                );
            };

            assert!(
                !file.locations().is_empty(),
                "Expected to have no jump at {pos:?}, but got {resp:#?} while processing goto jump to file:\n{file:#?}"
            );

            let GotoDefinitionResponse::Scalar(resp) = resp else {
                panic!(
                    "Expected Scalar format for response, but got other while processing goto jump at {pos:?}:\n{file:#?}\nRespone: {resp:#?}"
                );
            };

            assert_eq!(
                resp.uri, url,
                "Jump to other file, that is not expected while processing goto jump at {pos:?}:\n{file:#?}"
            );

            let expected = file.locations().first().unwrap();

            assert_eq!(
                resp.range, *expected,
                "Wrong jump location while processing goto jump at {pos:?}:\n{file:#?}"
            );
        }
    }

    /// Run test of file. Check based on number of provided locations:
    ///   * 0 locations - expect every cursor to not produce goto,
    ///   * 1 locations - expeect every cursor to jump to that location.
    #[track_caller]
    fn check_goto_type(&self, file: &FileMetadata) {
        assert!(
            !file.cursors().is_empty(),
            "File cursors shouldn't be empty for file:\n{file:#?}"
        );
        assert!(
            file.locations().len() < 2,
            "Invalid location number for file. Expect 1 or 0: \n{file:#?}"
        );

        let url = get_url(file);
        workspace_with_open_file(&self.workspace, file);
        check_file(&self.workspace, file);

        for pos in file.cursors() {
            let result = self
                .handler
                .goto_type_definition(&self.workspace, *pos, &url);

            let Ok(result) = result else {
                panic!(
                    "Server error while processing goto type jump at {pos:?}:\n{file:#?}\nError: {result:#?}"
                );
            };

            let Some(resp) = result else {
                if file.locations().is_empty() {
                    continue;
                }
                panic!(
                    "Expected to have goto location, but got nothing after processing goto type jump at {pos:?}:\n{file:#?}"
                );
            };

            assert!(
                !file.locations().is_empty(),
                "Expected to have no jump at {pos:?}, but got {resp:#?} while processing goto type jump to file:\n{file:#?}"
            );

            let GotoDefinitionResponse::Scalar(resp) = resp else {
                panic!(
                    "Expected Scalar format for response, but got other while processing goto type jump at {pos:?}:\n{file:#?}\nRespone: {resp:#?}"
                );
            };

            assert_eq!(
                resp.uri, url,
                "Jump to other file, that is not expected while processing goto type jump at {pos:?}:\n{file:#?}"
            );

            let expected = file.locations().first().unwrap();

            assert_eq!(
                resp.range, *expected,
                "Wrong jump location while processing goto type jump at {pos:?}:\n{file:#?}"
            );
        }
    }
}

#[test]
fn goto_test_type_message() {
    const TEXT: &str = r"
      |message |Tar|get| {}
      |        ^^^^^^^^^     <---
      |
      |message Dep (x |Tar|get|) {}
      |
      |message Field {
      |  x |Tar|get|;
      |}
      |
      |enum EDep (x |Tar|get|) {
      |  * => {
      |    Ctr0 {}
      |  }
      |}
      |
      |enum Ctr {
      |  Ctr1 {
      |    x |Tar|get|;
      |  }
      |}
      |
      |enum Pat (x |Tar|get|) (y Field) {
      |  |Tar|get|{}, * => {}
      |  *, Field{x: |Tar|get|{}} => {}
      |  *, * => {
      |    Ctr2 {}
      |  }
      |}
      |
      |message FieldD (f Field) {}
      |
      |message Constructor (d1 Dep |Tar|get|{}) (d2 FieldD Field{x: |Tar|get|{}}) {
      |  d3 Dep |Tar|get|{};
      |  d4 FieldD Field{x: |Tar|get|{}};
      |}
      |
      |enum EConstructor (d1 Dep |Tar|get|{}) (d2 FieldD Field{x: |Tar|get|{}}) {
      |  *, * => {
      |    Ctr3 {
      |      d3 Dep |Tar|get|{};
      |      d4 FieldD Field{x: |Tar|get|{}};
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_test_type_enum() {
    const TEXT: &str = r"
      |enum |Tar|get| {
      |     ^^^^^^^^^     <---
      |  Ctr0 {}
      |}
      |
      |message Dep (x |Tar|get|) {}
      |
      |message Field {
      |  x |Tar|get|;
      |}
      |
      |enum EDep (x |Tar|get|) {
      |  * => {
      |    Ctr1 {}
      |  }
      |}
      |
      |enum Ctr {
      |  Ctr2 {
      |    x |Tar|get|;
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_test_constructor() {
    const TEXT: &str = r"
      |enum Enum {
      |  |Tar|get| {}
      |  ^^^^^^^^^     <---
      |}
      |
      |message Dep (x Enum) {}
      |
      |message Field {
      |  x Enum;
      |}
      |
      |
      |enum Pat (x Enum) (y Field) {
      |  |Tar|get|{}, * => {}
      |  *, Field{x: |Tar|get|{}} => {}
      |  *, * => {
      |    Ctr2 {}
      |  }
      |}
      |
      |message FieldD (f Field) {}
      |
      |message Constructor (d1 Dep |Tar|get|{}) (d2 FieldD Field{x: |Tar|get|{}}) {
      |  d3 Dep |Tar|get|{};
      |  d4 FieldD Field{x: |Tar|get|{}};
      |}
      |
      |enum EConstructor (d1 Dep |Tar|get|{}) (d2 FieldD Field{x: |Tar|get|{}}) {
      |  *, * => {
      |    Ctr3 {
      |      d3 Dep |Tar|get|{};
      |      d4 FieldD Field{x: |Tar|get|{}};
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_test_dependency_message() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct {
      |  field Int;
      |}
      |
      |message StructD (d Struct) {}
      |
      |message Message (|tar|get| Int) {
      |                 ^^^^^^^^^         <---
      |  f1 IntD |tar|get|;
      |  f2 IntD (1 + |tar|get|);
      |  f3 StructD Struct{field: |tar|get|};
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_test_dependency_enum() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct {
      |  field Int;
      |}
      |
      |message StructD (d Struct) {}
      |
      |enum Enum (|tar|get| Int) {
      |           ^^^^^^^^^         <---
      |  * => {
      |    Ctr {
      |      f1 IntD |tar|get|;
      |      f2 IntD (1 + |tar|get|);
      |      f3 StructD Struct{field: |tar|get|};
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_test_field_message() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct {
      |  field Int;
      |}
      |
      |message StructD (d Struct) {}
      |
      |message Message {
      |  |tar|get| Int;
      |  ^^^^^^^^^        <---
      |  f1 IntD |tar|get|;
      |  f2 IntD (1 + |tar|get|);
      |  f3 StructD Struct{field: |tar|get|};
      |}
      |
      |message MDep (d Message) {}
      |
      |enum Pat (x Message) {
      |  Message{|tar|get|: a, f1: b, f2: c, f3: d} => {
      |    Ctr0 {}
      |  }
      |}
      |
      |message Constructor {
      |  d MDep Message{|tar|get|: 0, f1: IntD{}, f2: IntD{}, f3: StructD{}};
      |}
      |
      |message Access {
      |  f1 Message;
      |  f2 IntD f1.|tar|get|;
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_test_field_constructor() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct {
      |  field Int;
      |}
      |
      |message StructD (d Struct) {}
      |
      |enum Enum {
      |  Ctr0 {
      |    |tar|get| Int;
      |    ^^^^^^^^^        <---
      |    f1 IntD |tar|get|;
      |    f2 IntD (1 + |tar|get|);
      |    f3 StructD Struct{field: |tar|get|};
      |  }
      |}
      |
      |message EDep (d Enum) {}
      |
      |enum Pat (x Enum) {
      |  Ctr0{|tar|get|: a, f1: b, f2: c, f3: d} => {
      |    Ctr1 {}
      |  }
      |}
      |
      |message Constructor {
      |  d EDep Ctr0{|tar|get|: 0, f1: IntD{}, f2: IntD{}, f3: StructD{}};
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_test_alias() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct {
      |  field Int;
      |}
      |
      |message StructD (d Struct) {}
      |
      |enum Enum (d Struct) {
      |  Struct{field: |tar|get|} => {
      |                ^^^^^^^^^        <---
      |    Ctr {
      |      f1 IntD |tar|get|;
      |      f2 IntD (1 + |tar|get|);
      |      f3 StructD Struct{field: |tar|get|};
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto(&meta);
}

#[test]
fn goto_type_test_dependency_message() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^     <---
      |
      |message TDep (d Target) {}
      |
      |message Struct {
      |  field Target;
      |}
      |
      |message StructD (d Struct) {}
      |
      |message Message (|tar|get| Target) {
      |  f1 TDep |tar|get|;
      |  f2 StructD Struct{field: |tar|get|};
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto_type(&meta);
}

#[test]
fn goto_type_test_dependency_enum() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^     <---
      |
      |message TDep (d Target) {}
      |
      |message Struct {
      |  field Target;
      |}
      |
      |message StructD (d Struct) {}
      |
      |enum Enum (|tar|get| Target) {
      |  * => {
      |    Ctr {
      |      f1 TDep |tar|get|;
      |      f2 StructD Struct{field: |tar|get|};
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto_type(&meta);
}

#[test]
fn goto_type_test_field_message() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^     <---
      |
      |message TDep (d Target) {}
      |
      |message Struct {
      |  field Target;
      |}
      |
      |message StructD (d Struct) {}
      |
      |message Message {
      |  |tar|get| Target;
      |  f1 TDep |tar|get|;
      |  f2 StructD Struct{field: |tar|get|};
      |}
      |
      |message MDep (d Message) {}
      |
      |enum Pat (x Message) {
      |  Message{|tar|get|: a, f1: b, f2: c} => {
      |    Ctr0 {}
      |  }
      |}
      |
      |message Constructor {
      |  d MDep Message{|tar|get|: Target{}, f1: TDep{}, f2: StructD{}};
      |}
      |
      |message Access {
      |  f1 Message;
      |  f2 TDep f1.|tar|get|;
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto_type(&meta);
}

#[test]
fn goto_type_test_field_constructor() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^     <---
      |
      |message TDep (d Target) {}
      |
      |message Struct {
      |  field Target;
      |}
      |
      |message StructD (d Struct) {}
      |
      |enum Enum {
      |  Ctr0 {
      |    |tar|get| Target;
      |    f1 TDep |tar|get|;
      |    f2 StructD Struct{field: |tar|get|};
      |  }
      |}
      |
      |message EDep (d Enum) {}
      |
      |enum Pat (x Enum) {
      |  Ctr0{|tar|get|: a, f1: b, f2: c} => {
      |    Ctr1 {}
      |  }
      |}
      |
      |message Constructor {
      |  d EDep Ctr0{|tar|get|: Target{}, f1: TDep{}, f2: StructD{}};
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto_type(&meta);
}

#[test]
fn goto_type_test_alias() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^     <---
      |
      |message TDep (d Target) {}
      |
      |message Struct {
      |  field Target;
      |}
      |
      |message StructD (d Struct) {}
      |
      |enum Enum (d Struct) {
      |  Struct{field: |tar|get|} => {
      |    Ctr {
      |      f1 TDep |tar|get|;
      |      f2 StructD Struct{field: |tar|get|};
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_goto_type(&meta);
}
