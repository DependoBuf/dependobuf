//! Module helps while user writes code.
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
use tower_lsp::Client;

use crate::common::handler::Handler;

pub struct CompletitionHandler {
    _client: Client,
}

impl CompletitionHandler {}

impl Handler for CompletitionHandler {
    fn new(client: Client) -> Self {
        CompletitionHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, _capabilites: &mut ServerCapabilities) {}
}
