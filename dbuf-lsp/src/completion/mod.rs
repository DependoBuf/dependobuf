//! Module helps while user writes code.
//! Requests are using real-time updating parsed ast.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/completion`
//!
//! Also it might be good idea to handle such requests:
//!
//! Perhaps, next time:
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
    /// Currently builds simple east based on ast, that checks nothing
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
        let file = access.read(document);
        Ok(completion_impl::run_completion(pos, &file, context))
    }
}
