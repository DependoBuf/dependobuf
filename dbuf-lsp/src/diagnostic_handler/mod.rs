//! Module provides colorful diagnostic.
//!
//! Module should help with such requests:
//! * (✗) `textDocument/documentSymbol`
//! * (✓)`textDocument/semanticTokens/full`
//!
//! Also it might be good idea to handle such requests:
//! * `textDocument/diagnostic`
//! * `workspace/diagnostic`
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

mod semantic_token;

use semantic_token::SemanticTokenProvider;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::ast_access::WorkspaceAccess;
use crate::common::handler::Handler;

pub struct DiagnosticHandler {
    _client: Client,
}

impl DiagnosticHandler {
    pub async fn semantic_tokens_full(
        &self,
        access: &WorkspaceAccess,
        document: &Url,
    ) -> Result<Option<SemanticTokensResult>> {
        let file = access.read(document);
        let mut provider = SemanticTokenProvider::new(&file);
        let tokens = provider.provide();

        Ok(Some(tokens.into()))
    }
}

impl Handler for DiagnosticHandler {
    fn new(client: Client) -> Self {
        DiagnosticHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, capabilites: &mut ServerCapabilities) {
        let legend = SemanticTokensLegend {
            token_types: SemanticTokenProvider::get_token_types(),
            token_modifiers: SemanticTokenProvider::get_token_modifiers(),
        };

        capabilites.semantic_tokens_provider = Some(
            SemanticTokensOptions {
                legend,
                full: Some(SemanticTokensFullOptions::Bool(true)),
                ..Default::default()
            }
            .into(),
        );
    }
}
