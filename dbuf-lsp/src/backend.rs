use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::*;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tokio::sync::Mutex;
use tokio::sync::mpsc;
use tokio::task;
use tracing::{error, info, instrument};

use crate::WorkspaceAccess;
use crate::handler_box::HandlerBox;

use crate::action;
use crate::completion;
use crate::diagnostic;
use crate::navigation;

use super::trace::*;

struct Backend {
    _client: Client,
    workspace: WorkspaceAccess,
    action_handler: HandlerBox<action::Handler>,
    completion_handler: HandlerBox<completion::Handler>,
    diagnostic_handler: HandlerBox<diagnostic::Handler>,
    navigation_handler: HandlerBox<navigation::Handler>,
    notifier_task: Mutex<Option<task::JoinHandle<()>>>,
}

impl Backend {
    #[must_use]
    fn new(client: Client) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        setup_tracing(tx);

        let notifier = LoggerNotifier {
            client: client.clone(),
            rx,
        };

        let task = notifier.run_task();
        let option_task = Some(task);

        Self {
            _client: client,
            workspace: WorkspaceAccess::new(),
            action_handler: HandlerBox::default(),
            completion_handler: HandlerBox::default(),
            diagnostic_handler: HandlerBox::default(),
            navigation_handler: HandlerBox::default(),
            notifier_task: option_task.into(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, init: InitializeParams) -> Result<InitializeResult> {
        let mut capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(
                TextDocumentSyncOptions {
                    open_close: Some(true),
                    change: Some(TextDocumentSyncKind::FULL),
                    ..Default::default()
                },
            )),
            ..Default::default()
        };

        let current_capabilities = self.action_handler.init(&init);
        capabilities.document_formatting_provider =
            current_capabilities.document_formatting_provider;
        capabilities.rename_provider = current_capabilities.rename_provider;

        let _current_capabilities = self.completion_handler.init(&init);

        let current_capabilities = self.diagnostic_handler.init(&init);
        capabilities.document_symbol_provider = current_capabilities.document_symbol_provider;
        capabilities.semantic_tokens_provider = current_capabilities.semantic_tokens_provider;
        capabilities.references_provider = current_capabilities.references_provider;
        capabilities.document_highlight_provider = current_capabilities.document_highlight_provider;
        capabilities.code_lens_provider = current_capabilities.code_lens_provider;
        capabilities.diagnostic_provider = current_capabilities.diagnostic_provider;

        let current_capabilities = self.navigation_handler.init(&init);
        capabilities.definition_provider = current_capabilities.definition_provider;
        capabilities.type_definition_provider = current_capabilities.type_definition_provider;
        capabilities.hover_provider = current_capabilities.hover_provider;
        capabilities.inlay_hint_provider = current_capabilities.inlay_hint_provider;

        info!("initialize done");

        Ok(InitializeResult {
            capabilities,
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        if let Some(task) = self.notifier_task.lock().await.take() {
            task.await.expect("Notifier not finished");
        }
        Ok(())
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.workspace.open(doc.uri, doc.version, &doc.text);

        info!("file opened");
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if params.content_changes.len() != 1 {
            error!("bad param for did change (content_changes.len() != 1). Ignoring.");
            return;
        }

        let doc = params.text_document;
        let new_text = &params.content_changes[0].text;

        self.workspace.change(&doc.uri, doc.version, new_text);

        info!("file changed");
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let doc = params.text_document;
        self.workspace.close(&doc.uri);

        info!("file closed");
    }

    #[instrument(skip_all, fields(changes_count=_params.changes.len()))]
    async fn did_change_watched_files(&self, _params: DidChangeWatchedFilesParams) {
        info!("ignoring did changed watched files");
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let doc = params.text_document.uri;

        info!("document symbol request");

        self.diagnostic_handler
            .document_symbol(&self.workspace, &doc)
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let doc = params.text_document.uri;

        info!("semantic tokens full request");

        self.diagnostic_handler
            .semantic_tokens_full(&self.workspace, &doc)
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let doc = params.text_document.uri;

        info!("code lens request");

        self.diagnostic_handler.code_lens(&self.workspace, &doc)
    }

    #[instrument(skip_all, fields(file, pos.line, pos.character))]
    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        tracing::Span::current().record("file", uri.to_string());
        tracing::Span::current().record("pos.line", pos.line);
        tracing::Span::current().record("pos.character", pos.character);

        info!("goto definition request");

        self.navigation_handler
            .goto_definition(&self.workspace, pos, &uri)
    }

    #[instrument(skip_all, fields(file, pos.line, pos.character))]
    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        tracing::Span::current().record("file", uri.to_string());
        tracing::Span::current().record("pos.line", pos.line);
        tracing::Span::current().record("pos.character", pos.character);

        info!("goto type definition request");

        self.navigation_handler
            .goto_type_definition(&self.workspace, pos, &uri)
    }

    #[instrument(skip_all, fields(file, pos.line, pos.character))]
    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let doc_pos = params.text_document_position;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        tracing::Span::current().record("file", uri.to_string());
        tracing::Span::current().record("pos.line", pos.line);
        tracing::Span::current().record("pos.character", pos.character);

        info!("references request");

        self.diagnostic_handler
            .references(&self.workspace, pos, &uri)
    }

    #[instrument(skip_all, fields(file, pos.line, pos.character))]
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        tracing::Span::current().record("file", uri.to_string());
        tracing::Span::current().record("pos.line", pos.line);
        tracing::Span::current().record("pos.character", pos.character);

        info!("hover request");

        self.navigation_handler.hover(&self.workspace, pos, &uri)
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let range = params.range;
        let uri = params.text_document.uri;

        info!("inlay hint request");

        self.navigation_handler
            .inlay_hint(&self.workspace, range, &uri)
    }

    #[instrument(skip_all, fields(file, pos.line, pos.character))]
    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        tracing::Span::current().record("file", uri.to_string());
        tracing::Span::current().record("pos.line", pos.line);
        tracing::Span::current().record("pos.character", pos.character);

        info!("document highlight request");

        self.diagnostic_handler
            .document_highlight(&self.workspace, pos, &uri)
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> Result<DocumentDiagnosticReportResult> {
        let uri = params.text_document.uri;

        info!("diagnostic request");

        self.diagnostic_handler.diagnostic(&self.workspace, &uri)
    }

    #[instrument(skip_all, fields(file=params.text_document.uri.to_string()))]
    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;

        info!("formatting request");

        self.action_handler
            .formatting(&self.workspace, &params.options, &uri)
    }

    #[instrument(skip_all, fields(file, pos.line, pos.character))]
    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let pos = params.position;
        let uri = params.text_document.uri;

        tracing::Span::current().record("file", uri.to_string());
        tracing::Span::current().record("pos.line", pos.line);
        tracing::Span::current().record("pos.character", pos.character);

        info!("prepare rename request");

        self.action_handler
            .prepare_rename(&self.workspace, pos, &uri)
    }

    #[instrument(skip_all, fields(file, pos.line, pos.character))]
    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let doc_pos = params.text_document_position;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        tracing::Span::current().record("file", uri.to_string());
        tracing::Span::current().record("pos.line", pos.line);
        tracing::Span::current().record("pos.character", pos.character);

        info!("rename request");

        self.action_handler
            .rename(&self.workspace, &params.new_name, pos, &uri)
    }
}

pub fn run() {
    run_async();
}

#[tokio::main]
async fn run_async() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
