//! Module aims to help with searches in dbuf files.
//!
//! Module should help with such requests:
//! * `textDocument/declaration` (fn goto_declaration)
//! * `textDocument/typeDefinition` (fn goto_type_definition)
//! * `textDocument/references` (fn references)
//! * `textDocument/hover`
//! * `textDocument/documentHighlight` (✓)
//!  
//! Also it might be good idea to handle such requests:
//! * `textDocument/prepareTypeHierarchy`
//! * `typeHierarchy/supertypes`
//! * `typeHierarchy/subtypes`
//! * `textDocument/linkedEditingRange`
//! * `textDocument/moniker`
//!
//! These methods are also about navigation, but there no need to implement them:
//! * `textDocument/definition`
//! * `textDocument/implementation`
//! * `textDocument/prepareCallHierarchy`
//! * `callHierarchy/incomingCalls`
//! * `callHierarchy/outgoingCalls`
//! * `textDocument/documentLink`
//! * `documentLink/resolve`
//!

use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::ast_access::WorkspaceAccess;
use crate::common::handler::Handler;

use crate::common::navigator::Navigator;

#[derive(Debug)]
pub struct NavigationHandler {
    _client: Arc<Client>,
}

impl NavigationHandler {
    /// `textDocument/documentHighlight` implementation
    /// 
    pub async fn document_highlight(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: Url,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        self._client
            .log_message(MessageType::LOG, format!("hover at {:?}", pos))
            .await;

        let ranges;
        let symbol;
        {
            let file = access.read(&document);
            let navigator = Navigator::new(file.get_parsed(), file.get_elaborated());

            symbol = navigator.get_symbol(pos);
            ranges = navigator.find_symbols(&symbol);
        }

        self._client
            .log_message(
                MessageType::INFO,
                format!("symbol at {:?} is {:#?}", pos, &symbol),
            )
            .await;


        let mut ans = Vec::new();
        ans.reserve(ranges.len());
        for r in ranges.iter() {
            ans.push(DocumentHighlight {
                range: *r,
                kind: Some(DocumentHighlightKind::TEXT),
            });
        }

        self._client
        .log_message(
            MessageType::INFO,
            format!("ans: {:#?}", &ans),
        )
        .await;

        Ok(Some(ans))
    }
}

impl Handler for NavigationHandler {
    fn new(client: std::sync::Arc<Client>) -> Self {
        NavigationHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, capabilites: &mut ServerCapabilities) {
        capabilites.document_highlight_provider = Some(OneOf::Left(true));
    }
}
