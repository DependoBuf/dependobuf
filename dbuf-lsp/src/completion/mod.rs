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

use tower_lsp::lsp_types::InitializeParams;

use crate::handler_box;

pub struct Handler {}

/// Capabilities of completion Handler.
#[must_use]
pub struct Capabilities {}

impl handler_box::Handler for Handler {
    type Capabilities = Capabilities;

    fn create(_init: &InitializeParams) -> (Self::Capabilities, Self) {
        (Capabilities {}, Handler {})
    }
}

impl Handler {}
