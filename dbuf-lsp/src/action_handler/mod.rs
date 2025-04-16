//! Module aims to help formatting code.
//!
//! Module should help with such requests:
//! * (✓) `textDocument/formatting`
//! * (✓) `textDocument/rename`
//! * (✓) `textDocument/prepareRename`
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

use std::cell::RefCell;
use std::sync::Arc;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::ast_access::ElaboratedAst;
use crate::common::ast_access::ElaboratedHelper;
use crate::common::ast_access::WorkspaceAccess;
use crate::common::dbuf_language;
use crate::common::errors::*;
use crate::common::handler::Handler;
use crate::common::navigator::Navigator;
use crate::common::navigator::Symbol;
use crate::common::pretty_printer::PrettyPrinter;

#[derive(Default)]
struct SymbolInfo {
    document: Option<Url>,
    version: i32,
    pos: Position,
    symbol: Option<Symbol>,
}

pub struct ActionHandler {
    _client: Arc<Client>,

    rename_cache: Mutex<RefCell<SymbolInfo>>,
}

impl ActionHandler {
    /// `textDocument/formatting` implementation.
    ///
    /// Currently implementation is simple: just rewrite whole file, using pretty printer.
    /// Thats why function returns error on non default option.
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

        if !options.insert_spaces {
            return Err(bad_param_error("property 'insert_spaces' not true"));
        }
        if !options.properties.is_empty() {
            return Err(bad_param_error("property 'properties' not empty"));
        }
        if options.trim_trailing_whitespace.is_some() {
            return Err(bad_param_error(
                "property 'trim_trailing_whitespace' not none",
            ));
        }
        if options.insert_final_newline.is_some() {
            return Err(bad_param_error("property 'insert_final_newline' not none"));
        }
        if options.trim_final_newlines.is_some() {
            return Err(bad_param_error("property 'trim_final_newlines' not none"));
        }

        let mut edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(2e9 as u32, 0)),
            new_text: String::new(),
        };

        let file = access.read(document);
        let ast = file.get_parsed();

        let mut writer = PrettyPrinter::new(&mut edit.new_text).with_tab_size(options.tab_size);
        if writer.print_ast(ast).is_err() {
            return Err(internal_error("pretty printer couldn't parse ast"));
        }

        Ok(Some(vec![edit]))
    }

    /// `textDocument/prepareRename` implementation.
    ///
    /// Currently checks if symbol can be renamed and,
    /// if so, caches it.
    ///
    /// TODO:
    /// * Enum + constructors support.
    ///
    pub async fn prepare_rename(
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

            let navigator = Navigator::new(file.get_parsed(), file.get_elaborated());

            symbol = navigator.get_symbol(pos);
        }

        if ActionHandler::renameable_symbol(&symbol) {
            if let Ok(cell) = self.rename_cache.lock() {
                let mut cache = cell.borrow_mut();
                cache.document = Some(document.to_owned());
                cache.version = doc_version;
                cache.pos = pos;
                cache.symbol = Some(symbol);
            }

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
    /// there is no collision after rename.
    ///
    /// TODO:
    /// * Enum + constructors support.
    ///
    pub async fn rename(
        &self,
        access: &WorkspaceAccess,
        new_name: String,
        pos: Position,
        document: &Url,
    ) -> Result<Option<WorkspaceEdit>> {
        let mut symbol = Symbol::None;
        let ranges;
        let text_document;
        {
            let file = access.read(document);

            text_document = OptionalVersionedTextDocumentIdentifier {
                uri: document.to_owned(),
                version: Some(file.get_version()),
            };

            let navigator = Navigator::new(file.get_parsed(), file.get_elaborated());

            let mut cached_symbol = false;
            if let Ok(cell) = self.rename_cache.lock() {
                let mut last = cell.borrow_mut();
                if let Some(url) = &last.document {
                    if last.pos == pos && url == document && last.version == file.get_version() {
                        cached_symbol = true;
                        symbol = last.symbol.take().expect("added when add url");
                        last.document.take();
                    }
                }
            }

            if !cached_symbol {
                symbol = navigator.get_symbol(pos);
            }

            ActionHandler::renameable_to_symbol(&symbol, &new_name, file.get_elaborated())?;

            ranges = navigator.find_symbols(&symbol);
        }

        let edits = ranges
            .into_iter()
            .map(|s| {
                Left(TextEdit {
                    range: s,
                    new_text: new_name.to_owned(),
                })
            })
            .collect();

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

    fn renameable_symbol(symbol: &Symbol) -> bool {
        match symbol {
            Symbol::Type(t) => !dbuf_language::get_bultin_types().contains(t),
            Symbol::Dependency {
                t: _,
                dependency: _,
            } => true,
            Symbol::Field {
                constructor: _,
                field: _,
            } => true,
            Symbol::Constructor(_) => false,
            Symbol::None => false,
        }
    }

    fn renameable_to_symbol(symbol: &Symbol, new_name: &String, ast: &ElaboratedAst) -> Result<()> {
        if new_name.is_empty() {
            return Err(bad_rename_error("rename to empty string"));
        }
        if dbuf_language::get_bultin_types().contains(new_name) {
            return Err(bad_rename_error("rename to buildin type is forbidden"));
        }
        if dbuf_language::get_keywords().contains(new_name) {
            return Err(bad_rename_error("rename to keyword is forbidden"));
        }

        match symbol {
            Symbol::Type(t) => {
                if dbuf_language::get_bultin_types().contains(t) {
                    return Err(bad_rename_error("buildin type can't be renamed"));
                }
                if !dbuf_language::is_correct_type_name(new_name) {
                    return Err(bad_rename_error(
                        format!("'{}' is not correct type name", new_name).as_ref(),
                    ));
                }
                if t == new_name {
                    return Err(bad_rename_error("useless rename"));
                }
                if ast.has_type_or_constructor(new_name) {
                    return Err(bad_rename_error(
                        format!("constructor or type '{}' exist", new_name).as_ref(),
                    ));
                }
            }
            Symbol::Dependency {
                t: type_name,
                dependency: d,
            } => {
                if d == new_name {
                    return Err(bad_rename_error("useless rename"));
                }
                if !dbuf_language::is_correct_dependency_name(new_name) {
                    return Err(bad_rename_error(
                        format!("'{}' is not correct dependency name", new_name).as_ref(),
                    ));
                }
                if !ast.type_dependency_valid_rename(type_name, new_name) {
                    return Err(bad_rename_error(
                        format!("type '{}' already contains '{}'", type_name, new_name).as_ref(),
                    ));
                }
            }
            Symbol::Field {
                constructor: ctr,
                field: f,
            } => {
                if f == new_name {
                    return Err(bad_rename_error("useless rename"));
                }
                if !dbuf_language::is_correct_field_name(new_name) {
                    return Err(bad_rename_error(
                        format!("'{}' is not correct field name", new_name).as_ref(),
                    ));
                }
                if !ast.constructor_field_valid_rename(ctr, new_name) {
                    return Err(bad_rename_error(
                        format!("constructor '{}' already contains '{}'", ctr, new_name).as_ref(),
                    ));
                }
            }
            Symbol::Constructor(_) => {
                return Err(bad_rename_error("Constructors rename is not supported yet"))
            }
            Symbol::None => return Err(bad_rename_error("can't rename not symbol")),
        };

        Ok(())
    }
}

impl Handler for ActionHandler {
    fn new(client: Arc<Client>) -> ActionHandler {
        ActionHandler {
            _client: client,
            rename_cache: Mutex::new(RefCell::new(SymbolInfo::default())),
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
