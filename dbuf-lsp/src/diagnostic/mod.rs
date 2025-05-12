//! Module provides colorful diagnostic.
//! Requests are passing through full ast to produce response.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/documentSymbol`
//! * (✓) `textDocument/semanticTokens/full`
//! * (✓) `textDocument/documentHighlight`
//! * (✓) `textDocument/references`
//! * (✓) `textDocument/codeLens`
//!
//! Also it might be good idea to handle such requests:
//! ---
//!
//! Perhaps, next time:
//! * `codeLens/resolve`
//! * `textDocument/semanticTokens/full/delta`
//! * `textDocument/semanticTokens/range`
//! * `textDocument/diagnostic`
//! * `workspace/diagnostic`
//!
//! These methods are also about diagnostic, but there no need to implement them:
//! * `textDocument/documentColor`
//! * `textDocument/colorPresentation`
//! * `textDocument/inlineValue`
//!

mod code_lens;
mod document_symbol;
mod semantic_token;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;

use crate::handler_box;

use crate::core::ast_access::WorkspaceAccess;
use crate::core::navigator::Navigator;

#[derive(Default)]
pub struct Handler {}

/// Capabilities of diagnostic Handler.
#[must_use]
pub struct Capabilities {
    pub document_symbol_provider: Option<OneOf<bool, DocumentSymbolOptions>>,
    pub semantic_tokens_provider: Option<SemanticTokensServerCapabilities>,
    pub references_provider: Option<OneOf<bool, ReferencesOptions>>,
    pub document_highlight_provider: Option<OneOf<bool, DocumentHighlightOptions>>,
    pub code_lens_provider: Option<CodeLensOptions>,
}

impl handler_box::Handler for Handler {
    type Capabilities = Capabilities;

    fn create(_init: &InitializeParams) -> (Self::Capabilities, Self) {
        let legend = SemanticTokensLegend {
            token_types: semantic_token::get_token_types(),
            token_modifiers: semantic_token::get_token_modifiers(),
        };
        (
            Capabilities {
                document_symbol_provider: Some(Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensOptions {
                        legend,
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        ..Default::default()
                    }
                    .into(),
                ),
                references_provider: Some(Left(true)),
                document_highlight_provider: Some(Left(true)),
                code_lens_provider: Some(CodeLensOptions {
                    resolve_provider: Some(false),
                }),
            },
            Handler {},
        )
    }
}

impl Handler {
    /// `textDocument/documentSymbol` implementation.
    ///
    pub fn document_symbol(
        &self,
        access: &WorkspaceAccess,
        document: &Url,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let file = access.read(document);
        let symbols = document_symbol::provide_document_symbols(&file);
        Ok(Some(DocumentSymbolResponse::Nested(symbols)))
    }

    /// `textDocument/semanticTokens/full` implementation.
    ///
    pub fn semantic_tokens_full(
        &self,
        access: &WorkspaceAccess,
        document: &Url,
    ) -> Result<Option<SemanticTokensResult>> {
        let file = access.read(document);
        let tokens = semantic_token::provide_semantic_tokens(&file);

        Ok(Some(tokens.into()))
    }

    /// `textDocument/references` implementation.
    ///
    pub fn references(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: &Url,
    ) -> Result<Option<Vec<Location>>> {
        let file = access.read(document);
        let navigator = Navigator::new(&file);

        let symbol = navigator.get_symbol(pos);
        let ranges = navigator.find_symbols(&symbol);
        let ans = ranges
            .into_iter()
            .map(|r| Location {
                uri: document.to_owned(),
                range: r,
            })
            .collect();

        Ok(Some(ans))
    }

    /// `textDocument/documentHighlight` implementation.
    ///
    pub fn document_highlight(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: &Url,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let file = access.read(document);
        let navigator = Navigator::new(&file);

        let symbol = navigator.get_symbol(pos);
        let ranges = navigator.find_symbols(&symbol);

        let ans = ranges
            .into_iter()
            .map(|r| DocumentHighlight {
                range: r,
                kind: Some(DocumentHighlightKind::TEXT),
            })
            .collect();

        Ok(Some(ans))
    }

    /// `textDocument/codeLens` implementation.
    ///
    /// Currently shows only reference count.
    ///
    pub fn code_lens(
        &self,
        access: &WorkspaceAccess,
        document: &Url,
    ) -> Result<Option<Vec<CodeLens>>> {
        let file = access.read(document);
        let lens = code_lens::provide_code_lens(&file);

        Ok(Some(lens))
    }
}
