//! Module aims to help with searches in dbuf files.
//! Responses are easy to compute.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/definition`
//! * (✓) `textDocument/typeDefinition`
//! * (✓) `textDocument/hover
//!  
//! Also it might be good idea to handle such requests:
//!
//! Perhaps, next time:
//! * `textDocument/selectionRange`
//! * `textDocument/moniker`
//! * `textDocument/linkedEditingRange`
//!
//! These methods are also about navigation, but there no need to implement them:
//! * `textDocument/prepareTypeHierarchy`
//! * `typeHierarchy/supertypes`
//! * `typeHierarchy/subtypes`
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
        document: &Url,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let range;
        {
            let file = access.read(document);
            let navigator = Navigator::new(&file);

            let symbol = navigator.get_symbol(pos);
            range = find_definition(&navigator, &symbol);
        }

        match range {
            Some(range) => Ok(Some(GotoDefinitionResponse::Scalar(Location {
                uri: document.to_owned(),
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
        document: &Url,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let range;
        {
            let file = access.read(document);
            let navigator = Navigator::new(&file);

            let symbol = navigator.get_symbol(pos);
            let t = find_type(&navigator, symbol);

            range = find_definition(&navigator, &t);
        }

        match range {
            Some(range) => Ok(Some(GotoTypeDefinitionResponse::Scalar(Location {
                uri: document.to_owned(),
                range,
            }))),
            None => Ok(None),
        }
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
        document: &Url,
    ) -> Result<Option<Hover>> {
        let file = access.read(document);
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
}

impl Handler for NavigationHandler {
    fn new(client: Client) -> Self {
        NavigationHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, capabilites: &mut ServerCapabilities) {
        capabilites.definition_provider = Some(Left(true));
        capabilites.type_definition_provider = Some(TypeDefinitionProviderCapability::Simple(true));
        capabilites.hover_provider = Some(HoverProviderCapability::Simple(true));
    }
}
