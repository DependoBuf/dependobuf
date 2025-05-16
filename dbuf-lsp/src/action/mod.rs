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

mod format;
mod rename;
mod rename_cache;

use rename_cache::RenameCache;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;

use crate::handler_box;

use crate::core::ast_access::WorkspaceAccess;
use crate::core::navigator::Navigator;
use crate::core::pretty_printer::PrettyPrinter;

pub struct Handler {
    rename_cache: RenameCache,
}

/// Capabilities of action Handler.
#[must_use]
pub struct Capabilities {
    pub document_formatting_provider: Option<OneOf<bool, DocumentFormattingOptions>>,
    pub rename_provider: Option<OneOf<bool, RenameOptions>>,
}

impl handler_box::Handler for Handler {
    type Capabilities = Capabilities;

    fn create(_init: &InitializeParams) -> (Self::Capabilities, Self) {
        (
            Capabilities {
                document_formatting_provider: Some(Left(true)),
                rename_provider: Some(Right(RenameOptions {
                    prepare_provider: Some(true),
                    work_done_progress_options: WorkDoneProgressOptions {
                        work_done_progress: None,
                    },
                })),
            },
            Handler {
                rename_cache: RenameCache::default(),
            },
        )
    }
}

impl Handler {
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
        format::valid_options(&options)?;

        let mut edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(2e9 as u32, 0)),
            new_text: String::new(),
        };

        let file = access.read(document);
        let ast = file.get_parsed();

        let mut writer =
            PrettyPrinter::new(&mut edit.new_text).with_tab_size(options.tab_size as usize);
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
        let file = access.read(document);
        let doc_version = file.get_version();

        let navigator = Navigator::new(&file);
        let symbol = navigator.get_symbol(pos);

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

        let symbol = self
            .rename_cache
            .get(document, file.get_version(), pos)
            .map_or_else(|| navigator.get_symbol(pos), |cached| cached);

        rename::renameable_to_symbol(&symbol, &new_name, file.get_elaborated())?;

        let ranges = navigator.find_symbols(&symbol);

        Ok(Some(WorkspaceEdit {
            changes: None,
            document_changes: Some(DocumentChanges::Edits(vec![TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier {
                    uri: document.to_owned(),
                    version: Some(file.get_version()),
                },
                edits: ranges
                    .into_iter()
                    .map(|s| {
                        Left(TextEdit {
                            range: s,
                            new_text: new_name.to_owned(),
                        })
                    })
                    .collect(),
            }])),
            change_annotations: None,
        }))
    }
}
