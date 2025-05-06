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

use tower_lsp::lsp_types::*;

use crate::handler::{Capabilities, Handler};

pub struct CompletitionHandler {}

impl CompletitionHandler {}

struct CompletitionCapabilities {}

impl Capabilities for CompletitionCapabilities {
    fn apply(self, _capabilities: &mut ServerCapabilities) {}
}

impl Handler for CompletitionHandler {
    fn new() -> Self {
        CompletitionHandler {}
    }

    fn init(&self, _init: &InitializeParams) -> impl Capabilities {
        CompletitionCapabilities {}
    }
}
