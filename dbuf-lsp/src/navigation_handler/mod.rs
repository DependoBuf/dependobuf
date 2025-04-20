//! Module aims to help with searches in dbuf files.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/definition`
//! * (✓) `textDocument/typeDefinition`
//! * (✓) `textDocument/references`
//! * (✓) `textDocument/hover`
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

mod hover;
mod navigation;

use hover::get_hover;
use navigation::find_definition;
use navigation::find_type;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::*;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::ast_access::WorkspaceAccess;
use crate::common::handler::Handler;
use crate::common::navigator::Navigator;

pub struct NavigationHandler {
    _client: Client,
}

impl NavigationHandler {
    /// `textDocument/definition` implementation.
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
            let navigator = Navigator::new(&file);

            let symbol = navigator.get_symbol(pos);
            range = find_definition(&navigator, &symbol);
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
    pub async fn goto_type_definition(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let range;
        {
            let file = access.read(&document);
            let navigator = Navigator::new(&file);

            let symbol = navigator.get_symbol(pos);
            let t = find_type(&navigator, symbol);

            range = find_definition(&navigator, &t);
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
    pub async fn references(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<Vec<Location>>> {
        let ranges;
        {
            let file = access.read(&document);
            let navigator = Navigator::new(&file);

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

    /// `textDocument/hover` implementation.
    ///
    /// Provides such information:
    /// * For types: returns full type definition (TODO: if type is too huge -- reduce)
    /// * For dependencies: Type name ('message Type') and dependency declaration
    /// * For fields: Type name ('message Type'), Constructor if not message('    Ctr'), field declaration
    /// * For constructors: Type name ('enum Enum'), Constructor declaration without pattern
    /// * For aliases: Type name ('enum Enum') with dependencies, enum branch
    ///
    pub async fn hover(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<Hover>> {
        let file = access.read(&document);
        let navigator = Navigator::new(&file);

        let symbol = navigator.get_symbol(pos);

        let strings = get_hover(symbol, &file);
        if strings.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Hover {
                contents: HoverContents::Array(strings),
                range: None,
            }))
        }
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
            let navigator = Navigator::new(&file);

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
    fn new(client: Client) -> Self {
        NavigationHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, capabilites: &mut ServerCapabilities) {
        capabilites.definition_provider = Some(Left(true));
        capabilites.type_definition_provider = Some(TypeDefinitionProviderCapability::Simple(true));
        capabilites.references_provider = Some(Left(true));
        capabilites.hover_provider = Some(HoverProviderCapability::Simple(true));
        capabilites.document_highlight_provider = Some(Left(true));
    }
}
