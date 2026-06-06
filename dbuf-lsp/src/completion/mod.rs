//! Module helps while user writes code.
//! Requests are using real-time updating parsed ast.
//!
//! Module should help with such requests:
//!
//! Also it might be good idea to handle such requests:
//!
//! Perhaps, next time:
//! * `textDocument/completion`
//! * `textDocument/signatureHelp`
//! * `completionItem/resolve`
//!
//! These methods are also about completition, but there no need to implement them:
//!
//!
mod completion_impl;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::InitializeParams;
use tower_lsp::lsp_types::*;

use crate::{WorkspaceAccess, handler_box};

pub struct Handler {}

/// Capabilities of completion Handler.
#[must_use]
pub struct Capabilities {
    pub completion_provider: Option<CompletionOptions>,
}

impl handler_box::Handler for Handler {
    type Capabilities = Capabilities;

    fn create(_init: &InitializeParams) -> (Self::Capabilities, Self) {
        let completion_provider = Some(CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec![".".to_string()]),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: Some(false),
            },
            completion_item: None,
        });

        (
            Capabilities {
                completion_provider,
            },
            Handler {},
        )
    }
}

impl Handler {
    // `textDocument/completion` implementation.
    ///
    /// Currently builds simple east based on cst, that checks nothing
    /// but stores fields of constructors. Then based on it provides
    /// completion.
    ///
    /// # Errors
    ///
    /// Errors are never return.
    pub fn completion(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: &Url,
        context: Option<CompletionContext>,
    ) -> Result<Option<CompletionResponse>> {
        /*
        let c1 = CompletionItem {
            label: "abc".to_string(),
            label_details: Some(CompletionItemLabelDetails {
                detail: Some("Signature".to_string()),
                description: Some("qualified_name".to_string()),
            }),
            kind: None,
            detail: Some("Details".to_string()),
            documentation: Some(Documentation::String(
                "That is a documentation of abc".to_string(),
            )),
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: None,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: Some(vec![".".to_string()]),
            data: None,
            tags: None,
        };

        let c2 = CompletionItem {
            label: "def".to_string(),
            label_details: None,
            kind: None,
            detail: None,
            documentation: None,
            deprecated: None,
            preselect: None,
            sort_text: None,
            filter_text: None,
            insert_text: None,
            insert_text_format: None,
            insert_text_mode: None,
            text_edit: None,
            additional_text_edits: None,
            command: None,
            commit_characters: Some(vec![".".to_string()]),
            data: None,
            tags: None,
        };
        */

        let file = access.read(document);
        Ok(completion_impl::run_completion(pos, &file, context))
    }
}
