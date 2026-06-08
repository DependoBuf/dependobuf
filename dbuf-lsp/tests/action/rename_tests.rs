//! Module contains all rename tests.
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
    /// Run rename expecting responce with ranges.
    #[track_caller]
    fn run_rename(&self, file: &FileMetadata, new_name: &str, pos: Position) -> Vec<Range> {
        let url = get_url(file);
        let result = self.handler.rename(&self.workspace, new_name, pos, &url);

        let Ok(result) = result else {
            panic!(
                "Server error while processing file with cursor at {pos:?}:\n{file:#?}\nError:{result:?}"
            );
        };

        let Some(edits) = result else {
            panic!("No edits while editing file with cursor at {pos:?}:\n{file:#?}");
        };

        let Some(document_changes) = edits.document_changes else {
            panic!("No document changes while editing file with cursor at {pos:?}:\n{file:#?}");
        };

        let DocumentChanges::Edits(edits) = document_changes else {
            panic!("Wrong edits format while editing file with cursor at {pos:?}:\n{file:#?}");
        };

        assert_eq!(
            edits.len(),
            1,
            "Wrong number of edits while editing file with cursor at {pos:?}:\n{file:#?}"
        );

        let edits = edits.first().expect("just checked");

        edits.edits.iter().map(|edit| {
            match edit {
                OneOf::Left(x) => {
                    assert_eq!(x.new_text, new_name, "Wrong name edit while editing file with cursor at {pos:?}:\n{file:#?}");
                    x.range
                },
                OneOf::Right(_) => panic!("found edit of annotation which is unsupported while editing file with cursor at {pos:?}:\n{file:#?}"),
            }
        }).collect::<Vec<_>>()
    }

    /// Run prepare rename.
    ///
    /// Result:
    /// * Some(()) - successfully prepared rename,
    /// * None - bad place for prepare rename.
    #[track_caller]
    fn run_prepare_rename(&self, file: &FileMetadata, pos: Position) -> Option<()> {
        let url = get_url(file);
        let result = self.handler.prepare_rename(&self.workspace, pos, &url);

        let Ok(opt) = result else {
            panic!(
                "Returned error in prepare rename while editing file with cursor at {pos:?}:\n{file:#?}\n"
            );
        };

        let resp = opt?;

        let PrepareRenameResponse::DefaultBehavior { default_behavior } = resp else {
            panic!(
                "Prepare rename with not default behavior while editing file with cursor at {pos:?}:\n{file:#?}\n"
            );
        };

        assert!(
            default_behavior,
            "Prepare rename with default behavior set to false while editing file with cursor at {pos:?}:\n{file:#?}\n"
        );

        Some(())
    }

    /// Run test on file, expecting every cursor location to produce all ranges.
    /// For each locations runs rename twice: without prepare and with, expecting
    /// to have similar results.
    #[track_caller]
    fn check_rename(&self, file: &FileMetadata, new_name: &str) {
        workspace_with_open_file(&self.workspace, file);
        check_file(&self.workspace, file);

        assert!(
            !file.cursors().is_empty(),
            "No cursors for file:\n{file:#?}"
        );
        for cursor_locaion in file.cursors() {
            let ranges = {
                let ranges1 = self.run_rename(file, new_name, *cursor_locaion);

                let Some(()) = self.run_prepare_rename(file, *cursor_locaion) else {
                    panic!(
                        "Cannot prepare rename while editing file with cursor at {cursor_locaion:?}:\n{file:#?}\n"
                    );
                };
                let ranges2 = self.run_rename(file, new_name, *cursor_locaion);

                assert_eq!(
                    ranges1, ranges2,
                    "Different ranges when preparing and not rename while editing file with cursor at {cursor_locaion:?}:\n{file:#?}\n"
                );
                ranges1
            };

            let mut extra = vec![];
            let mut missing = vec![];
            for range in &ranges {
                if !file.locations().contains(range) {
                    extra.push(range);
                }
            }

            for range in file.locations() {
                if !ranges.contains(range) {
                    missing.push(range);
                }
            }

            if !extra.is_empty() || !missing.is_empty() {
                let mut msg = format!(
                    "Wrong ranges while editing file with cursor at {cursor_locaion:?}:\n{file:#?}\n"
                );
                if !extra.is_empty() {
                    write!(&mut msg, "Extra ranges:\n{extra:#?}\n").unwrap();
                }
                if !missing.is_empty() {
                    write!(&mut msg, "Missing ranges:\n{missing:#?}\n").unwrap();
                }
                panic!("{msg}");
            }
        }

        workspace_with_close_file(&self.workspace, file);
    }

    /// Run test on file expecting all cursor locations produce errors.
    #[track_caller]
    fn check_invalid(&self, file: &FileMetadata, new_name: &str) {
        let url = get_url(file);
        workspace_with_open_file(&self.workspace, file);
        check_file(&self.workspace, file);

        assert!(
            !file.cursors().is_empty(),
            "No cursors for file:\n{file:#?}"
        );
        for cursor_locaion in file.cursors() {
            let result = self
                .handler
                .rename(&self.workspace, new_name, *cursor_locaion, &url);

            if let Ok(result) = result {
                panic!(
                    "Expected error while processing file with cursor at {cursor_locaion:?}:\n{file:#?}\nResult:{result:#?}"
                )
            }
        }

        workspace_with_close_file(&self.workspace, file);
    }

    /// Runs test expecting correct output on changed file. Produces cache miss by changing file in workspace
    /// between prepare rename and rename.
    #[track_caller]
    fn check_cache_miss(&self, file: &FileMetadata, changed: &FileMetadata, new_name: &str) {
        assert_eq!(
            file.file_name(),
            changed.file_name(),
            "Expect changes of one file"
        );
        assert!(
            !file.cursors().is_empty(),
            "No cursors for file:\n{file:#?}"
        );
        assert!(
            !changed.cursors().is_empty(),
            "No cursors for changed file:\n{changed:#?}"
        );

        for fst in file.cursors() {
            for snd in file.cursors() {
                if fst != snd {
                    continue;
                }

                workspace_with_open_file(&self.workspace, file);
                check_file(&self.workspace, file);
                self.run_prepare_rename(file, *fst).expect("prepared");
                workspace_with_change_file(&self.workspace, changed);

                let ranges = self.run_rename(changed, new_name, *snd);
                let mut extra = vec![];
                let mut missing = vec![];
                for range in &ranges {
                    if !changed.locations().contains(range) {
                        extra.push(range);
                    }
                }

                for range in changed.locations() {
                    if !ranges.contains(range) {
                        missing.push(range);
                    }
                }

                if !extra.is_empty() || !missing.is_empty() {
                    let mut msg = format!(
                        "Wrong ranges while editing file with cursor at {snd:?}:\nFrom file:\n{file:#?}To file:\n{changed:#?}\n"
                    );
                    if !extra.is_empty() {
                        write!(&mut msg, "Extra ranges:\n{extra:#?}\n").unwrap();
                    }
                    if !missing.is_empty() {
                        write!(&mut msg, "Missing ranges:\n{missing:#?}\n").unwrap();
                    }
                    panic!("{msg}");
                }

                workspace_with_close_file(&self.workspace, file);
            }
        }
    }
}

#[test]
fn rename_messsage_declaration() {
    const TEXT: &str = r"
      |message |Tar|get| {}
      |        ^^^^^^^^^     <---
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_dependency() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^                  <---
      |
      |message Other (x |Tar|get|) {}
      |                 ^^^^^^^^^      <---
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_field() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^     <---
      |
      |message Other {
      |  f |Tar|get|;
      |    ^^^^^^^^^      <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_field_constructor() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^     <---
      |
      |enum Enum {
      |  Constructor {
      |    x |Tar|get|;
      |      ^^^^^^^^^    <---
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_constructor() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^                <---
      |
      |message Other (x Target) {}
      |                 ^^^^^^       <---
      |
      |message Test {
      |    f Other |Tar|get|{};
      |            ^^^^^^^^^         <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_constructor_field() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^                       <---
      |
      |message Other {
      |    f Target;
      |      ^^^^^^                         <---
      |}
      |
      |message OtherD (o Other) {}
      |
      |message Test {
      |    f OtherD Other{f: |Tar|get|{}};
      |                      ^^^^^^^^^      <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_pattern() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^           <---
      |
      |enum Other (x Target) {
      |              ^^^^^^     <---
      |  |Tar|get|{} => {
      |  ^^^^^^^^^              <---
      |    Ctr{}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_pattern_field() {
    const TEXT: &str = r"
      |message Target {}
      |        ^^^^^^                <---
      |
      |message Other {
      |  f Target;
      |    ^^^^^^                    <---
      |}
      |
      |enum Other2 (x Other) {
      |  Other{f: |Tar|get|{}} => {
      |           ^^^^^^^^^          <---
      |    Ctr {}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_message_all() {
    const TEXT: &str = r"
      |message |Tar|get| {}
      |        ^^^^^^^^^                                     <---
      |
      |message TargetD (t |Tar|get|) {
      |                   ^^^^^^^^^                          <---
      |  f |Tar|get|;
      |    ^^^^^^^^^                                         <---
      |}
      |
      |enum Enum (t1 |Tar|get|) (t2 TargetD t1) {
      |              ^^^^^^^^^                               <---
      |  |Tar|get|{}, TargetD{f: |Tar|get|{}} => {
      |  ^^^^^^^^^               ^^^^^^^^^                   <---
      |    Ctr1 {}
      |    Ctr2 {
      |        f1 |Tar|get|;
      |           ^^^^^^^^^                                  <---
      |        f2 Enum |Tar|get|{} TargetD{f: |Tar|get|{}};
      |                ^^^^^^^^^              ^^^^^^^^^      <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_enum_declaration() {
    const TEXT: &str = r"
      |enum |Tar|get| {
      |     ^^^^^^^^^    <---
      |  Ctr{}
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_enum_dependency() {
    const TEXT: &str = r"
      |enum Target {
      |     ^^^^^^                     <---
      |  Ctr{}
      |}
      |
      |message Other (x |Tar|get|) {}
      |                 ^^^^^^^^^      <---
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_enum_field() {
    const TEXT: &str = r"
      |enum Target {
      |     ^^^^^^      <---
      |  Ctr{}
      |}
      |
      |message Other {
      |  f |Tar|get|;
      |    ^^^^^^^^^    <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_enum_field_constructor() {
    const TEXT: &str = r"
      |enum Target {
      |     ^^^^^^       <---
      |  Ctr{}
      |}
      |
      |enum Enum {
      |  Constructor {
      |    x |Tar|get|;
      |      ^^^^^^^^^   <---
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_enum_all() {
    const TEXT: &str = r"
      |enum |Tar|get| {
      |     ^^^^^^^^^                   <---
      |  Ctr1{}
      |  Ctr2{
      |    f |Tar|get|;
      |      ^^^^^^^^^                  <---
      |  }
      |}
      |
      |message Message (t |Tar|get|) {
      |                   ^^^^^^^^^     <---
      |  f |Tar|get|;
      |    ^^^^^^^^^                    <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_constructor_declaration() {
    const TEXT: &str = r"
      |enum Enum {
      |  |Tar|get|{}
      |  ^^^^^^^^^    <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_constructor_constructor() {
    const TEXT: &str = r"
      |enum Enum {
      |  Target{}
      |  ^^^^^^                   <---
      |}
      |
      |message Other (x Enum) {}
      |
      |message Test {
      |    f Other |Tar|get|{};
      |            ^^^^^^^^^      <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_constructor_constructor_field() {
    const TEXT: &str = r"
      |enum Enum {
      |  Target{}
      |  ^^^^^^                             <---
      |}
      |
      |message Other {
      |    f Enum;
      |}
      |
      |message OtherD (o Other) {}
      |
      |message Test {
      |    f OtherD Other{f: |Tar|get|{}};
      |                      ^^^^^^^^^      <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_constructor_pattern() {
    const TEXT: &str = r"
      |enum Enum {
      |  Target{}
      |  ^^^^^^               <---
      |}
      |
      |enum Other (x Enum) {
      |  |Tar|get|{} => {
      |  ^^^^^^^^^            <---
      |    Ctr{}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_constructor_pattern_field() {
    const TEXT: &str = r"
      |enum Enum {
      |  Target{}
      |  ^^^^^^                      <---
      |}
      |
      |message Other {
      |  f Enum;
      |}
      |
      |enum Other2 (x Other) {
      |  Other{f: |Tar|get|{}} => {
      |           ^^^^^^^^^          <---
      |    Ctr {}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_constructor_all() {
    const TEXT: &str = r"
      |enum Enum {
      |  |Tar|get|{}
      |  ^^^^^^^^^                                <---
      |}
      |
      |message EnumD (e Enum) {}
      |
      |message Other {
      |  f Enum;
      |}
      |
      |message OtherD (o Other) {}
      |
      |enum Other2 (x Enum) (y Other) {
      |  |Tar|get|{}, Other{f: |Tar|get|{}} => {
      |  ^^^^^^^^^             ^^^^^^^^^          <---
      |    Ctr {
      |      f1 EnumD |Tar|get|{};
      |               ^^^^^^^^^                   <---
      |      f2 OtherD Other{f: |Tar|get|{}};
      |                         ^^^^^^^^^         <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "NewName");
}

#[test]
fn rename_dependency_message_declaration() {
    const TEXT: &str = r"
      |message Message (|tar|get| Int) {}
      |                 ^^^^^^^^^          <---
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_message_dependency() {
    const TEXT: &str = r"
      |message IntD (i Int) {}
      |
      |message Message (target Int) (i IntD |tar|get|) {}
      |                 ^^^^^^              ^^^^^^^^^      <---
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_message_fields() {
    const TEXT: &str = r"
      |message IntD (i Int) {}
      |
      |message Message (target Int) {
      |                 ^^^^^^         <---
      |  i IntD |tar|get|;
      |         ^^^^^^^^^              <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_message_constructor_use() {
    const TEXT: &str = r"
      |message Struct {
      |  i Int;
      |}
      |
      |message StructD (s Struct) {}
      |
      |message Message (target Int) {
      |                 ^^^^^^            <---
      |  s StructD Struct{i: |tar|get|};
      |                      ^^^^^^^^^    <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_message_all() {
    const TEXT: &str = r"
      |message Struct {
      |  i Int;
      |}
      |
      |message IntD (d Int) {}
      |
      |message StructD (s Struct) {}
      |
      |message Message (|tar|get| Int) (i IntD |tar|get|) {
      |                 ^^^^^^^^^              ^^^^^^^^^     <---
      |  f1 IntD |tar|get|;
      |          ^^^^^^^^^                                   <---
      |  f2 StructD Struct{i: |tar|get|};
      |                       ^^^^^^^^^                      <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_enum_declaration() {
    const TEXT: &str = r"
      |enum Enum (|tar|get| Int) {
      |           ^^^^^^^^^         <---
      |  * => {
      |    Ctr{}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_enum_dependency() {
    const TEXT: &str = r"
      |message IntD (i Int) {}
      |
      |enum Enum (target Int) (i IntD |tar|get|) {
      |           ^^^^^^              ^^^^^^^^^     <---
      |  *, * => {
      |    Ctr{}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_enum_fields() {
    const TEXT: &str = r"
      |message IntD (i Int) {}
      |
      |enum Enum (target Int) {
      |           ^^^^^^         <---
      |  * => {
      |    Ctr{
      |      i IntD |tar|get|;
      |             ^^^^^^^^^    <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_enum_constructor_use() {
    const TEXT: &str = r"
      |message Struct {
      |  i Int;
      |}
      |
      |message StructD (s Struct) {}
      |
      |message Message (target Int) {
      |                 ^^^^^^           <---
      |  s StructD Struct{i: |tar|get|};
      |                      ^^^^^^^^^   <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_dependency_enum_all() {
    const TEXT: &str = r"
      |message Struct {
      |  i Int;
      |}
      |
      |message IntD (d Int) {}
      |
      |message StructD (s Struct) {}
      |
      |enum Enum (|tar|get| Int) (i IntD |tar|get|) {
      |           ^^^^^^^^^              ^^^^^^^^^     <---
      |  *, * => {
      |    Ctr {
      |      f1 IntD |tar|get|;
      |              ^^^^^^^^^                         <---
      |      f2 StructD Struct{i: |tar|get|};
      |                           ^^^^^^^^^            <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_message_declaration() {
    const TEXT: &str = r"
      |message Message {
      |  |tar|get| Int;
      |  ^^^^^^^^^        <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_message_field() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Message {
      |  target Int;
      |  ^^^^^^             <---
      |  f IntD |tar|get|;
      |         ^^^^^^^^^   <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_message_constructor_decl() {
    const TEXT: &str = r"
      |message Message {
      |  target Int;
      |  ^^^^^^                         <---
      |}
      |
      |message MDep (m Message) {}
      |
      |message Test {
      |  f MDep Message{|tar|get|: 0};
      |                 ^^^^^^^^^       <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_message_constructor_use() {
    const TEXT: &str = r"
      |message Struct {
      |  f Int;
      |}
      |
      |message SDep (d Struct) {}
      |
      |message Message {
      |  target Int;
      |  ^^^^^^                        <---
      |  f SDep Struct{f: |tar|get|};
      |                   ^^^^^^^^^    <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_message_call_chain() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Message {
      |  target Int;
      |  ^^^^^^                 <---
      |}
      |
      |message Test {
      |  f1 Message;
      |  f2 IntD f1.|tar|get|;
      |             ^^^^^^^^^   <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_message_pattern_decl() {
    const TEXT: &str = r"
      |message Message {
      |  target Int;
      |  ^^^^^^                      <---
      |}
      |
      |enum Enum (m Message) {
      |  Message{|tar|get|: 0} => {
      |          ^^^^^^^^^           <---
      |    Ctr {}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
#[should_panic(expected = "")] /* FIXME: actually shouldn't panic (ElaboratingStage error: Unknown field target) */
fn rename_field_message_all() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct {
      |  f Int;
      |}
      |
      |message SDep (d Struct) {}
      |
      |message Message {
      |  |tar|get| Int;
      |  ^^^^^^^^^                                                   <---
      |  f1 IntD |tar|get|;
      |          ^^^^^^^^^                                           <---
      |  f2 SDep Struct{f: |tar|get|};
      |                    ^^^^^^^^^                                 <---
      |}
      |
      |enum Enum (m Message) {
      |  Message{|tar|get|: 0, f1: f1, f2: f2} => {
      |          ^^^^^^^^^                                           <---
      |    Ctr1 {}
      |    Ctr2 {
      |      f1 Enum Message{|tar|get|: 0, f1: IntD{}, f2: SDep{}};
      |                      ^^^^^^^^^                               <---
      |      f2 Message;
      |      f3 IntD f2.|tar|get|;
      |                 ^^^^^^^^^                                    <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_constructor_declaration() {
    const TEXT: &str = r"
      |enum Enum {
      |  Ctr {
      |    |tar|get| Int;
      |    ^^^^^^^^^       <---
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_constructor_field() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |enum Enum {
      |  Ctr {
      |    |tar|get| Int;
      |    ^^^^^^^^^          <---
      |    f IntD |tar|get|;
      |           ^^^^^^^^^   <---
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_constructor_constructor_decl() {
    const TEXT: &str = r"
      |enum Enum {
      |  Ctr {
      |    target Int;
      |    ^^^^^^                   <---
      |  }
      |}
      |
      |message EDep (e Enum) {}
      |
      |message Test {
      |  f EDep Ctr{|tar|get|: 0};
      |             ^^^^^^^^^       <---
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_constructor_constructor_use() {
    const TEXT: &str = r"
      |message Struct {
      |  f Int;
      |}
      |
      |message SDep (d Struct) {}
      |
      |enum Enum {
      |  Ctr {
      |    target Int;
      |    ^^^^^^                        <---
      |    f SDep Struct{f: |tar|get|};
      |                     ^^^^^^^^^    <---
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_constructor_pattern_decl() {
    const TEXT: &str = r"
      |enum Enum {
      |  Ctr {
      |    target Int;
      |    ^^^^^^                <---
      |  }
      |}
      |
      |enum Test (e Enum) {
      |  Ctr{|tar|get|: 0} => {
      |      ^^^^^^^^^           <---
      |    Ctr2 {}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_field_constructor_all() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct {
      |  f Int;
      |}
      |
      |message SDep (d Struct) {}
      |
      |enum Enum {
      |  Ctr {
      |    |tar|get| Int;
      |    ^^^^^^^^^                                             <---
      |    f1 IntD |tar|get|;
      |            ^^^^^^^^^                                     <---
      |    f2 SDep Struct{f: |tar|get|};
      |                      ^^^^^^^^^                           <---
      |  }
      |}
      |
      |enum Test (e Enum) {
      |  Ctr{|tar|get|: 0, f1: f1, f2: f2} => {
      |      ^^^^^^^^^                                           <---
      |    Ctr1 {}
      |    Ctr2 {
      |      f1 Test Ctr{|tar|get|: 0, f1: IntD{}, f2: SDep{}};
      |                  ^^^^^^^^^                               <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_alias_declaration() {
    const TEXT: &str = r"
      |message Struct {
      |  f Int;
      |}
      |
      |enum Test (s Struct) {
      |  Struct{f: |tar|get|} => {
      |            ^^^^^^^^^        <---
      |    Ctr {}
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_alias_field() {
    const TEXT: &str = r"
      |message Struct {
      |  f Int;
      |}
      |
      |message IntD (d Int) {}
      |
      |enum Test (s Struct) {
      |  Struct{f: target} => {
      |            ^^^^^^        <---
      |    Ctr {
      |      i IntD |tar|get|;
      |             ^^^^^^^^^    <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_alias_constructor() {
    const TEXT: &str = r"
      |message Struct {
      |  f Int;
      |}
      |
      |message SDep (d Struct) {}
      |
      |enum Test (s Struct) {
      |  Struct{f: target} => {
      |            ^^^^^^                  <---
      |    Ctr {
      |      i SDep Struct{f: |tar|get|};
      |                       ^^^^^^^^^    <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_alias_call_chain() {
    const TEXT: &str = r"
      |message Struct {
      |  f Int;
      |}
      |
      |message Struct2 {
      |  f Struct;
      |}
      |
      |message IntD (d Int) {}
      |
      |enum Test (s Struct2) {
      |  Struct2{f: target} => {
      |             ^^^^^^        <---
      |    Ctr {
      |      i IntD |tar|get|.f;
      |             ^^^^^^^^^     <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn rename_alias_all() {
    const TEXT: &str = r"
      |message Struct {
      |  f Int;
      |}
      |
      |message Struct2 {
      |  f Struct;
      |}
      |
      |message IntD (d Int) {}
      |
      |message SDep (d Struct) {}
      |
      |message S2Dep (d Struct2) {}
      |
      |enum Test (s Struct2) {
      |  Struct2{f: |tar|get|} => {
      |             ^^^^^^^^^                 <---
      |    Ctr {
      |      f1 SDep |tar|get|;
      |              ^^^^^^^^^                <---
      |      f2 S2Dep Struct2{f: |tar|get|};
      |                          ^^^^^^^^^    <---
      |      f3 IntD |tar|get|.f;
      |              ^^^^^^^^^                <---
      |    }
      |  }
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();
    scenario.check_rename(&meta, "newName");
}

#[test]
fn invalid_rename_place() {
    {
        const TEXT: &str = r"
          ||mes|sage| Simple {}
        ";

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "message");
    }

    {
        const TEXT: &str = r"
          ||e|num| Simple {
          |  Ctr {}
          |}
        ";

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "enum");
    }

    {
        const TEXT: &str = r"
          |message Simple {
          |  f |I|nt|;
          |}
        ";

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "Num");
    }

    {
        const TEXT: &str = r"
          |message Simple {
          |  f |St|ring|;
          |}
        ";

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "Text");
    }

    {
        const TEXT: &str = r"
          |message IntD (d Int) {}
          |
          |message Simple {
          |  f IntD |1|00|;
          |}
        ";

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "f");
    }

    {
        const TEXT: &str = r#"
          |message SDep (d String) {}
          |
          |message Simple {
          |  f SDep |"a|ba"|;
          |}
        "#;

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "f");
    }

    {
        const TEXT: &str = r"
          | |/|/| |co|mm|
          | |/|*| co|mm| |*|/|
          |message Simple {
          |  f Int;
          |}|
          |  |
        ";

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "f");
    }

    {
        const TEXT: &str = r"
          |message Simple {
          |  f Int;
          |}|
          |  |
        ";

        let meta = FileConfig::default().construct(TEXT);
        let scenario = Scenario::default();
        scenario.check_invalid(&meta, "f");
    }
}

#[test]
fn invalid_rename_type() {
    const TEXT: &str = r"
      |message |Simple {}
      |
      |message Other {}
      |
      |enum Enum {
      |  Ctr {}
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_invalid(&meta, "");

    scenario.check_invalid(&meta, "lowletter");

    scenario.check_invalid(&meta, "Int");
    scenario.check_invalid(&meta, "String");

    scenario.check_invalid(&meta, "Simple");
    scenario.check_invalid(&meta, "Other");
    scenario.check_invalid(&meta, "Enum");
    scenario.check_invalid(&meta, "Ctr");
}

#[test]
fn invalid_rename_field() {
    const TEXT: &str = r"
      |message Simple {
      |  |f1 Int;
      |  ^^^             <---
      |  f2 String;
      |}
      |
      |message Other {
      |  f3 Int;
      |}
    ";

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_invalid(&meta, "");

    scenario.check_invalid(&meta, "UpLetter");

    scenario.check_invalid(&meta, "enum");
    scenario.check_invalid(&meta, "message");

    scenario.check_invalid(&meta, "f1");
    scenario.check_invalid(&meta, "f2");

    scenario.check_rename(&meta, "f3");
}

#[test]
fn rename_cache_miss() {
    const TEXT1: &str = r"
      |message IntD (d Int) {}
      |
      |message Simple {
      |  f|1 Int;
      |  ^^^          <---
      |  f2 IntD f1;
      |          ^^   <---
      |}
    ";

    const TEXT2: &str = r"
      |message IntD (d Int) {}
      |
      |message Simple {
      |  f|1 Int;
      |  ^^^         <---
      |  f2 IntD 0;
      |}
    ";

    let meta1 = FileConfig::default().construct(TEXT1);
    let meta2 = FileConfig::default().construct(TEXT2);
    let scenario = Scenario::default();

    scenario.check_rename(&meta1, "newField");
    scenario.check_rename(&meta2, "newField");

    scenario.check_cache_miss(&meta1, &meta2, "newField");
}
