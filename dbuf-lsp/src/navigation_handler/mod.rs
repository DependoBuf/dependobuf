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

use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::*;
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;

use crate::common::ast_access::ElaboratedHelper;
use crate::common::ast_access::WorkspaceAccess;
use crate::common::dbuf_language::get_bultin_types;
use crate::common::handler::Handler;
use crate::common::navigator::Navigator;
use crate::common::pretty_printer::PrettyPrinter;

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
            let navigator = Navigator::new(&file);

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
            let navigator = Navigator::new(&file);

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
    /// * For Constructors: Type name ('enum Enum'), Constructor declaration without pattern
    ///
    /// TODO:
    /// * Enum + constructor support
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

        let mut strings = Vec::new();
        match symbol {
            crate::common::navigator::Symbol::Type(t) => {
                let mut code = String::new();
                if get_bultin_types().contains(&t) {
                    code = t;
                } else {
                    let mut printer = PrettyPrinter::new(&mut code);
                    printer.print_type(file.get_parsed(), t.as_ref());
                }
                let ls = LanguageString {
                    language: "dbuf".to_owned(),
                    value: code,
                };
                strings.push(MarkedString::LanguageString(ls));
            }
            crate::common::navigator::Symbol::Dependency { t, dependency } => {
                let mut type_header = String::new();

                let mut p_header = PrettyPrinter::new(&mut type_header)
                    .with_header_only()
                    .without_dependencies();
                p_header.print_type(file.get_parsed(), t.as_ref());

                let ls1 = LanguageString {
                    language: "dbuf".to_owned(),
                    value: type_header,
                };

                strings.push(MarkedString::LanguageString(ls1));

                let mut dependency_declaration = String::new();

                let mut p_dependency_decl = PrettyPrinter::new(&mut dependency_declaration);
                p_dependency_decl.print_selected_dependency(file.get_parsed(), &t, &dependency);

                let ls2 = LanguageString {
                    language: "dbuf".to_owned(),
                    value: dependency_declaration,
                };

                strings.push(MarkedString::LanguageString(ls2));
                strings.push(MarkedString::String(format!("dependency of {}", t)));
            }
            crate::common::navigator::Symbol::Field {
                t,
                constructor,
                field,
            } => {
                let mut type_header = String::new();

                let mut p_header = PrettyPrinter::new(&mut type_header)
                    .with_header_only()
                    .without_dependencies();
                p_header.print_type(file.get_parsed(), t.as_ref());

                let ls1 = LanguageString {
                    language: "dbuf".to_owned(),
                    value: type_header,
                };

                strings.push(MarkedString::LanguageString(ls1));

                if !file.get_elaborated().is_message(&t) {
                    todo!(); // Enums are not implemented
                }

                let mut s_field = String::new();

                let mut p_field = PrettyPrinter::new(&mut s_field);
                p_field.print_selected_field(file.get_parsed(), &t, &constructor, &field);

                let ls3 = LanguageString {
                    language: "dbuf".to_owned(),
                    value: s_field,
                };

                strings.push(MarkedString::LanguageString(ls3));
                strings.push(MarkedString::String(format!("field of {}", constructor)));
            }
            crate::common::navigator::Symbol::Constructor(_) => {} // Not implemented
            crate::common::navigator::Symbol::None => {}
        };

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
    fn new(client: std::sync::Arc<Client>) -> Self {
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
