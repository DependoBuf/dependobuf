//! Tests for `textDocument/completion`.
use tower_lsp::lsp_types::*;

use crate::common::*;

use super::HandlerType;
use super::get_handler;

use pretty_assertions::assert_eq;

use dbuf_core::error::Error as CoreError;
use dbuf_core::error::parsing::ErrorExtra;
use dbuf_core::error::parsing::ParsingStage;
use dbuf_lsp::Error;

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

/// Context of completion operation.
enum Context {
    /// Regular call.
    Regular,
    /// On dot call.
    Dot,
}

impl From<Context> for CompletionContext {
    fn from(value: Context) -> Self {
        match value {
            Context::Regular => CompletionContext {
                trigger_kind: CompletionTriggerKind::INVOKED,
                trigger_character: None,
            },
            Context::Dot => CompletionContext {
                trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
                trigger_character: Some(".".to_string()),
            },
        }
    }
}

impl Scenario {
    /// Check file to open without error, expect acceptable. Acceptable:
    /// * Elaboration errors,
    /// * Parsing `BadCallChain` error.
    #[track_caller]
    fn check_file(&self, file: &FileMetadata) {
        let url = get_url(file);
        let file = self.workspace.read(&url);
        for err in file.get_errors() {
            if matches!(err, Error::ElaboratingError(_)) {
                continue;
            }

            if matches!(
                err,
                Error::Parsing(CoreError {
                    stage: ParsingStage {
                        found: _,
                        expected: _,
                        at: _,
                        extra: Some(ErrorExtra::BadCallChain(_))
                    }
                })
            ) {
                continue;
            }
            panic!("Got non acceptable error while processing file:\n{err:#?}");
        }
    }

    #[track_caller]
    fn check_completion(&self, file: &FileMetadata, expect: &[&str], ctx: Context) {
        assert_eq!(file.cursors().len(), 1, "Too many cursors in file");
        assert!(file.locations().is_empty(), "Expect to have no locations");

        let url = get_url(file);
        workspace_with_open_file(&self.workspace, file);
        self.check_file(file);

        let pos = file.cursors().first().unwrap();
        let res = self
            .handler
            .completion(&self.workspace, *pos, &url, Some(ctx.into()));

        let Ok(res) = res else {
            panic!("Server error while processing completion action: {res:#?}");
        };

        let Some(res) = res else {
            assert!(
                expect.is_empty(),
                "Got empty completion response, while waiting some"
            );
            return;
        };

        let CompletionResponse::Array(resp) = res else {
            panic!("Expect Array format of completion response");
        };

        assert_eq!(
            resp.into_iter().map(|c| c.label).collect::<Vec<_>>(),
            expect,
            "Wrong answer on completion request"
        );

        workspace_with_close_file(&self.workspace, file);
    }
}

#[test]
fn test_regular_simple() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Test (d1 Int) (d2 String) {
      |  f1 Int;
      |  f2 String;
      |  f3 IntD f|;
      |}
    ";

    let expect: &[&str] = &["f1", "f2", "f3", "d1", "d2"];
    let ctx = Context::Regular;

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_completion(&meta, expect, ctx);
}

#[test]
fn test_regular_call() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct (d1 Int) (d2 String) {
      |  f1 Int;
      |  f2 String;
      |}
      |
      |message Test (d3 Int) (d4 String) {
      |  f3 Struct d3 d4;
      |  f4 IntD f3.f|;
      |}
    ";

    let expect: &[&str] = &["f1", "f2"];
    let ctx = Context::Regular;

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_completion(&meta, expect, ctx);
}

#[test]
#[should_panic = "Wrong answer on completion request"] // FIXME: actually not collecting alias information.
fn test_regular_enum() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |enum Test (d1 Int) (d2 String) {
      |  a1, * => {
      |    Ctr1 {
      |      f1 Int;
      |      f2 String;
      |    }
      |    Ctr2 {
      |      f3 Int;
      |      f4 String;
      |      f5 IntD f|;
      |    }
      |  }
      |}
    ";

    let expect: &[&str] = &["f3", "f4", "f5", "d1", "d2", "a1"];
    let ctx = Context::Regular;

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_completion(&meta, expect, ctx);
}

#[test]
fn test_regular_enum_call() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |enum Enum (d1 Int) (d2 String) {
      |  a1, * => {
      |    Ctr1 {
      |      f1 Int;
      |      f2 String;
      |    }
      |    Ctr2 {
      |      f3 Int;
      |      f4 String;
      |    }
      |  }
      |}
      |
      |message Test {
      |  f5 Enum;
      |  f6 IntD f5.f|;
      |}
    ";

    let expect: &[&str] = &[];
    let ctx = Context::Regular;

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_completion(&meta, expect, ctx);
}

#[test]
fn test_dot_call() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |message Struct (d1 Int) (d2 String) {
      |  f1 Int;
      |  f2 String;
      |}
      |
      |message Test (d3 Int) (d4 String) {
      |  f3 Struct d3 d4;
      |  f4 IntD f3.|;
      |}
    ";

    let expect: &[&str] = &["f1", "f2"];
    let ctx = Context::Dot;

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_completion(&meta, expect, ctx);
}

#[test]
fn test_dot_enum_call() {
    const TEXT: &str = r"
      |message IntD (d Int) {}
      |
      |enum Enum (d1 Int) (d2 String) {
      |  a1, * => {
      |    Ctr1 {
      |      f1 Int;
      |      f2 String;
      |    }
      |    Ctr2 {
      |      f3 Int;
      |      f4 String;
      |    }
      |  }
      |}
      |
      |message Test {
      |  f5 Enum;
      |  f6 IntD f5.|;
      |}
    ";

    let expect: &[&str] = &[];
    let ctx = Context::Dot;

    let meta = FileConfig::default().construct(TEXT);
    let scenario = Scenario::default();

    scenario.check_completion(&meta, expect, ctx);
}
