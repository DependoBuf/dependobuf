//! Module aims to help with searches in dbuf files.
//! Responses are easy to compute.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/definition`
//! * (✓) `textDocument/typeDefinition`
//! * (✓) `textDocument/hover
//! * (✗!!) `textDocument/inlayHint` // for constructors type
//!  
//! Also it might be good idea to handle such requests:
//!
//! Perhaps, next time:
//! * `inlayHint/resolve`
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
mod inlay_hint;
mod navigation_impl;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::request::*;
use tower_lsp::lsp_types::*;

use crate::handler_box;

use crate::core::ast_access::WorkspaceAccess;
use crate::core::navigator::Navigator;

#[derive(Default)]
pub struct Handler {}

/// Capabilities of navigation Handler.
#[must_use]
pub struct Capabilities {
    pub definition_provider: Option<OneOf<bool, DefinitionOptions>>,
    pub type_definition_provider: Option<TypeDefinitionProviderCapability>,
    pub hover_provider: Option<HoverProviderCapability>,
}

impl handler_box::Handler for Handler {
    type Capabilities = Capabilities;

    fn create(_init: &InitializeParams) -> (Self::Capabilities, Self) {
        (
            Capabilities {
                definition_provider: Some(Left(true)),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
            },
            Handler {},
        )
    }
}

impl Handler {
    /// `textDocument/definition` implementation.
    ///
    pub fn goto_definition(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: &Url,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let file = access.read(document);
        let navigator = Navigator::new(&file);

        let symbol = navigator.get_symbol(pos);
        let range = navigation_impl::find_definition(&navigator, &symbol);

        Ok(range.map(|range| {
            GotoDefinitionResponse::Scalar(Location {
                uri: document.to_owned(),
                range,
            })
        }))
    }

    /// `textDocument/typeDefintion` implementation.
    ///
    pub fn goto_type_definition(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: &Url,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let file = access.read(document);
        let navigator = Navigator::new(&file);

        let symbol = navigator.get_symbol(pos);
        let t = navigation_impl::find_type(&navigator, symbol);

        let range = navigation_impl::find_definition(&navigator, &t);

        Ok(range.map(|range| {
            GotoTypeDefinitionResponse::Scalar(Location {
                uri: document.to_owned(),
                range,
            })
        }))
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
    pub fn hover(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: &Url,
    ) -> Result<Option<Hover>> {
        let file = access.read(document);
        let navigator = Navigator::new(&file);

        let symbol = navigator.get_symbol(pos);

        let strings = hover::get_hover(symbol, &file);
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
