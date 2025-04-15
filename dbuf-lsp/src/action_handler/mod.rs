//! Module aims to help formatting code.
//!
//! Module should help with such requests:
//! * `textDocument/formatting` (✓)
//! * `textDocument/rename`
//! * `textDocument/prepareRename`
//!
//! Also it might be good idea to handle such requests:
//! * `textDocument/selectionRange`
//! * `textDocument/inlayHint`
//! * `inlayHint/resolve`
//! * `textDocument/foldingRange`
//! * `textDocument/codeAction`
//! * `codeAction/resolve`
//! * `textDocument/rangeFormatting`
//! * `textDocument/onTypeFormatting`
//!
//! These methods are also about action, but there no need to implement them:
//! * `textDocument/codeLens`
//! * `codeLens/resolve`
//!
//!

use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::ast_access::WorkspaceAccess;
use crate::common::errors::*;
use crate::common::handler::Handler;
use crate::common::pretty_printer::PrettyPrinter;

#[derive(Debug)]
pub struct ActionHandler {
    _client: Arc<Client>,
}

impl ActionHandler {
    /// `textDocument/formatting` implementation.
    ///
    /// Currently implementation is simple: just rewrite whole file, using pretty printer.
    /// Thats why function returns error on non default option.
    ///
    ///
    pub async fn formatting(
        &self,
        access: &WorkspaceAccess,
        options: FormattingOptions,
        document: &Url,
    ) -> Result<Option<Vec<TextEdit>>> {
        self._client
            .log_message(MessageType::LOG, format!("{:#?}", options))
            .await;

        if options.insert_spaces != true {
            return Err(bad_param_error("property 'insert_spaces' not true"));
        }
        if !options.properties.is_empty() {
            return Err(bad_param_error("property 'properties' not empty"));
        }
        if let Some(_) = options.trim_trailing_whitespace {
            return Err(bad_param_error(
                "property 'trim_trailing_whitespace' not none",
            ));
        }
        if let Some(_) = options.insert_final_newline {
            return Err(bad_param_error("property 'insert_final_newline' not none"));
        }
        if let Some(_) = options.trim_final_newlines {
            return Err(bad_param_error("property 'trim_final_newlines' not none"));
        }

        let mut edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(2e9 as u32, 0)),
            new_text: String::new(),
        };

        let file = access.read(&document);
        let ast = file.get_parsed();

        let mut writer = PrettyPrinter::new(&mut edit.new_text).with_tab_size(options.tab_size);
        if let Err(_) = writer.print_ast(&ast) {
            return Err(internal_error("pretty printer couldn't parse ast"));
        }

        return Ok(Some(vec![edit]));
    }
}

impl Handler for ActionHandler {
    fn new(client: Arc<Client>) -> ActionHandler {
        ActionHandler { _client: client }
    }

    fn init(&self, _init: &InitializeParams, capabilites: &mut ServerCapabilities) {
        capabilites.document_formatting_provider = Some(Left(true));
    }
}
