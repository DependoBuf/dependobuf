//! Module aims to help formatting code.
//! Reqeusts are changing ast.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/formatting`
//! * (✓) `textDocument/rename`
//! * (✓) `textDocument/prepareRename`
//!
//! Perhaps, next time:
//! * `textDocument/codeAction`
//! * `codeAction/resolve`
//! * `textDocument/rangeFormatting`
//! * `textDocument/onTypeFormatting`
//! * `textDocument/foldingRange`
//!
//! These methods are also about action, but there no need to implement them:
//!

mod rename;
mod rename_cache;

use rename_cache::RenameCache;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;

use crate::core::ast_access::WorkspaceAccess;
use crate::core::errors::format_errors;
use crate::core::navigator::Navigator;
use crate::core::pretty_printer::PrettyPrinter;
use crate::handler::Handler;

pub struct ActionHandler {
    rename_cache: RenameCache,
}

impl ActionHandler {
    /// `textDocument/formatting` implementation.
    ///
    /// Currently implementation is simple: just rewrite whole file, using pretty printer.
    /// Thats why function returns error on non default option.
    ///
    pub fn formatting(
        &self,
        access: &WorkspaceAccess,
        options: FormattingOptions,
        document: &Url,
    ) -> Result<Option<Vec<TextEdit>>> {
        if !options.insert_spaces {
            return format_errors::bad_insert_spaces();
        }
        if !options.properties.is_empty() {
            return format_errors::bad_propertis();
        }
        if options.trim_trailing_whitespace.is_some() {
            return format_errors::bad_trim_trailing_whitespace();
        }
        if options.insert_final_newline.is_some() {
            return format_errors::bad_insert_final_newline();
        }
        if options.trim_final_newlines.is_some() {
            return format_errors::bad_trim_final_newlines();
        }

        let mut edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(2e9 as u32, 0)),
            new_text: String::new(),
        };

        let file = access.read(document);
        let ast = file.get_parsed();

        let mut writer = PrettyPrinter::new(&mut edit.new_text).with_tab_size(options.tab_size);
        writer.print_ast(ast);

        Ok(Some(vec![edit]))
    }

    /// `textDocument/prepareRename` implementation.
    ///
    /// Currently checks if symbol can be renamed and,
    /// if so, caches it.
    ///
    pub fn prepare_rename(
        &self,
        access: &WorkspaceAccess,
        pos: Position,
        document: &Url,
    ) -> Result<Option<PrepareRenameResponse>> {
        let symbol;
        let doc_version;
        {
            let file = access.read(document);
            doc_version = file.get_version();

            let navigator = Navigator::new(&file);

            symbol = navigator.get_symbol(pos);
        }

        if rename::renameable_symbol(&symbol) {
            self.rename_cache
                .set(document.to_owned(), doc_version, pos, symbol);
            Ok(Some(PrepareRenameResponse::DefaultBehavior {
                default_behavior: true,
            }))
        } else {
            Ok(None)
        }
    }

    /// `textDocument/rename` implementation.
    ///
    /// Renames symbol if possible. Checks that
    /// there is no conflicts after rename.
    ///
    pub fn rename(
        &self,
        access: &WorkspaceAccess,
        new_name: String,
        pos: Position,
        document: &Url,
    ) -> Result<Option<WorkspaceEdit>> {
        let file = access.read(document);
        let navigator = Navigator::new(&file);

        let symbol = if let Some(cached) = self.rename_cache.get(document, file.get_version(), pos)
        {
            cached
        } else {
            navigator.get_symbol(pos)
        };

        rename::renameable_to_symbol(&symbol, &new_name, file.get_elaborated())?;

        let ranges = navigator.find_symbols(&symbol);

        let edits = ranges
            .into_iter()
            .map(|s| {
                Left(TextEdit {
                    range: s,
                    new_text: new_name.to_owned(),
                })
            })
            .collect();

        let text_document = OptionalVersionedTextDocumentIdentifier {
            uri: document.to_owned(),
            version: Some(file.get_version()),
        };

        let text_document_edits = TextDocumentEdit {
            text_document,
            edits,
        };

        let workspace_edit = WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![text_document_edits])),
            change_annotations: None,
        };

        Ok(Some(workspace_edit))
    }
}

impl Handler for ActionHandler {
    fn new() -> ActionHandler {
        ActionHandler {
            rename_cache: RenameCache::default(),
        }
    }

    fn init(&self, _init: &InitializeParams, capabilites: &mut ServerCapabilities) {
        capabilites.document_formatting_provider = Some(Left(true));
        capabilites.rename_provider = Some(Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
        }));
    }
}
