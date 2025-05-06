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

use crate::core::ast_access::WorkspaceAccess;
use crate::core::navigator::Navigator;
use crate::handler::{Capabilities, Handler};

pub struct DiagnosticHandler {}

impl DiagnosticHandler {
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

struct DiagnosticCapabilities {
    document_symbol: bool,
    semantic_tokens: Option<SemanticTokensLegend>,
    references: bool,
    document_highlight: bool,
    code_lens: bool,
}

impl Capabilities for DiagnosticCapabilities {
    fn apply(self, capabilities: &mut ServerCapabilities) {
        if self.document_symbol {
            capabilities.document_symbol_provider = Some(Left(true));
        }
        if let Some(legend) = self.semantic_tokens {
            capabilities.semantic_tokens_provider = Some(
                SemanticTokensOptions {
                    legend,
                    full: Some(SemanticTokensFullOptions::Bool(true)),
                    ..Default::default()
                }
                .into(),
            );
        }
        if self.references {
            capabilities.references_provider = Some(Left(true));
        }
        if self.document_highlight {
            capabilities.document_highlight_provider = Some(Left(true));
        }
        if self.code_lens {
            capabilities.code_lens_provider = Some(CodeLensOptions {
                resolve_provider: Some(false),
            });
        }
    }
}

impl Handler for DiagnosticHandler {
    fn new() -> Self {
        Self {}
    }

    fn init(&self, _init: &InitializeParams) -> impl Capabilities {
        let legend = SemanticTokensLegend {
            token_types: semantic_token::get_token_types(),
            token_modifiers: semantic_token::get_token_modifiers(),
        };

        DiagnosticCapabilities {
            document_symbol: true,
            semantic_tokens: Some(legend),
            references: true,
            document_highlight: true,
            code_lens: true,
        }
    }
}
