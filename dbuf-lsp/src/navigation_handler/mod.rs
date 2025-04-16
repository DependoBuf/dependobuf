//! Module aims to help with searches in dbuf files.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/definition`
//! * (✓) `textDocument/typeDefinition`
//! * (✓) `textDocument/references`
//! * (✗) `textDocument/hover`
//! * (✓) `textDocument/documentHighlight`
//!  
//! Also it might be good idea to handle such requests:
//! * `textDocument/prepareTypeHierarchy`
//! * `typeHierarchy/supertypes`
//! * `typeHierarchy/subtypes`
//! * `textDocument/linkedEditingRange`
//! * `textDocument/moniker`
//!
//! These methods are also about navigation, but there no need to implement them:
//! * `textDocument/declaration`
//! * `textDocument/implementation`
//! * `textDocument/prepareCallHierarchy`
//! * `callHierarchy/incomingCalls`
//! * `callHierarchy/outgoingCalls`
//! * `textDocument/documentLink`
//! * `documentLink/resolve`
//!

use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::*;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::ast_access::WorkspaceAccess;
use crate::common::handler::Handler;

use crate::common::navigator::Navigator;

pub struct NavigationHandler {
    _client: Arc<Client>,
}

impl NavigationHandler {
    /// `textDocument/definition` implementation.
    ///
    /// TODO:
    /// * Enum + constructor support
    ///
    pub async fn goto_definition(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let range;
        {
            let file = access.read(&document);
            let navigator = Navigator::for_file(&file);

            let symbol = navigator.get_symbol(pos);
            range = navigator.find_definition(&symbol);
        }

        match range {
            Some(range) => Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: document,
                range,
            }))),
            None => Ok(None),
        }
    }

    /// `textDocument/typeDefintion` implementation.
    ///
    /// TODO:
    /// * Enum + constructor support
    ///
    pub async fn goto_type_definition(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let range;
        {
            let file = access.read(&document);
            let navigator = Navigator::for_file(&file);

            let symbol = navigator.get_symbol(pos);
            let t = navigator.find_type(&symbol);

            range = navigator.find_definition(&t);
        }

        match range {
            Some(range) => Ok(Some(GotoTypeDefinitionResponse::Scalar(Location {
                uri: document,
                range,
            }))),
            None => Ok(None),
        }
    }

    /// `textDocument/references` implementation.
    ///
    /// TODO:
    /// * Enum + constructor support
    /// * message field support
    ///
    pub async fn references(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<Vec<Location>>> {
        let ranges;
        {
            let file = access.read(&document);
            let navigator = Navigator::for_file(&file);

            let symbol = navigator.get_symbol(pos);
            ranges = navigator.find_symbols(&symbol);
        }

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
    pub async fn document_highlight(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let ranges;
        {
            let file = access.read(&document);
            let navigator = Navigator::for_file(&file);

            let symbol = navigator.get_symbol(pos);
            ranges = navigator.find_symbols(&symbol);
        }

        let mut ans = Vec::with_capacity(ranges.len());
        for r in ranges.iter() {
            ans.push(DocumentHighlight {
                range: *r,
                kind: Some(DocumentHighlightKind::TEXT),
            });
        }

        Ok(Some(ans))
    }
}

impl Handler for NavigationHandler {
    fn new(client: std::sync::Arc<Client>) -> Self {
        NavigationHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, capabilites: &mut ServerCapabilities) {
        capabilites.definition_provider = Some(Left(true));
        capabilites.type_definition_provider = Some(TypeDefinitionProviderCapability::Simple(true));
        capabilites.references_provider = Some(Left(true));
        capabilites.document_highlight_provider = Some(Left(true));
    }
}
