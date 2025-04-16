//! Module provides colorful diagnostic.
//!
//! Module should help with such requests:
//! * `textDocument/diagnostic`
//!
//! Also it might be good idea to handle such requests:
//! * `workspace/diagnostic`
//! * `textDocument/documentSymbol`
//! * `textDocument/semanticTokens/full`
//! * `textDocument/semanticTokens/full/delta`
//! * `textDocument/semanticTokens/range`
//! * `textDocument/inlineValue`
//! * `textDocument/inlayHint`
//!
//! These methods are also about diagnostic, but there no need to implement them:
//! * `textDocument/documentColor`
//! * `textDocument/colorPresentation`
//!
//!

use std::sync::Arc;

use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::handler::Handler;

pub struct DiagnosticHandler {
    _client: Arc<Client>,
}

impl DiagnosticHandler {}

impl Handler for DiagnosticHandler {
    fn new(client: Arc<Client>) -> Self {
        DiagnosticHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, _capabilites: &mut ServerCapabilities) {}
}
