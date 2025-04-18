//! Module provides colorful diagnostic.
//!
//! Module should help with such requests:
//! * `textDocument/documentSymbol`
//!
//! Also it might be good idea to handle such requests:
//! * `textDocument/diagnostic`
//! * `workspace/diagnostic`
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

use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::handler::Handler;

pub struct DiagnosticHandler {
    _client: Client,
}

impl DiagnosticHandler {}

impl Handler for DiagnosticHandler {
    fn new(client: Client) -> Self {
        DiagnosticHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, _capabilites: &mut ServerCapabilities) {}
}
