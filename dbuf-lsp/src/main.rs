use dbuf_lsp::completion_handler::CompletitionHandler;
use dbuf_lsp::diagnostic_handler::DiagnosticHandler;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::*;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use dbuf_lsp::common::ast_access::WorkspaceAccess;

use dbuf_lsp::action_handler::ActionHandler;
use dbuf_lsp::common::handler::Handler;
use dbuf_lsp::navigation_handler::NavigationHandler;

struct Backend {
    client: Client,
    workspace: WorkspaceAccess,
    action_handler: ActionHandler,
    completition_handler: CompletitionHandler,
    diagnostic_handler: DiagnosticHandler,
    navigation_handler: NavigationHandler,
}

impl Backend {
    fn new(client: Client) -> Backend {
        Backend {
            client: client.clone(),
            workspace: WorkspaceAccess::new(),
            action_handler: ActionHandler::new(client.clone()),
            completition_handler: CompletitionHandler::new(client.clone()),
            diagnostic_handler: DiagnosticHandler::new(client.clone()),
            navigation_handler: NavigationHandler::new(client),
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
                    will_save: Some(false),
                    will_save_wait_until: Some(false),
                    save: Some(TextDocumentSyncSaveOptions::Supported(false)),
                },
            )),
            ..Default::default()
        };

        self.action_handler.init(&init, &mut capabilities);
        self.completition_handler.init(&init, &mut capabilities);
        self.diagnostic_handler.init(&init, &mut capabilities);
        self.navigation_handler.init(&init, &mut capabilities);

        Ok(InitializeResult {
            capabilities,
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        self.workspace.open(doc.uri, doc.version, &doc.text);

        self.client
            .log_message(MessageType::INFO, "file opened")
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if params.content_changes.len() != 1 {
            self.client
                .log_message(MessageType::ERROR, "file change is not full")
                .await;
            panic!("bad param for did change");
        }

        let doc = params.text_document;
        let new_text = &params.content_changes[0].text;

        self.workspace.change(&doc.uri, doc.version, new_text);

        self.client
            .log_message(MessageType::INFO, "file changed")
            .await;
    }
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let doc = params.text_document;
        self.workspace.close(&doc.uri);

        self.client
            .log_message(MessageType::INFO, "file closed")
            .await;
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let doc = params.text_document.uri;

        self.diagnostic_handler
            .document_symbol(&self.workspace, &doc)
            .await
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let doc = params.text_document.uri;

        self.diagnostic_handler
            .semantic_tokens_full(&self.workspace, &doc)
            .await
    }

    async fn code_lens(&self, params: CodeLensParams) -> Result<Option<Vec<CodeLens>>> {
        let doc = params.text_document.uri;

        self.diagnostic_handler
            .code_lens(&self.workspace, &doc)
            .await
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        self.navigation_handler
            .goto_definition(&self.workspace, pos, &uri)
            .await
    }

    async fn goto_type_definition(
        &self,
        params: GotoTypeDefinitionParams,
    ) -> Result<Option<GotoTypeDefinitionResponse>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        self.navigation_handler
            .goto_type_definition(&self.workspace, pos, &uri)
            .await
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let doc_pos = params.text_document_position;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        self.diagnostic_handler
            .references(&self.workspace, pos, &uri)
            .await
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        self.navigation_handler
            .hover(&self.workspace, pos, &uri)
            .await
    }

    async fn document_highlight(
        &self,
        params: DocumentHighlightParams,
    ) -> Result<Option<Vec<DocumentHighlight>>> {
        let doc_pos = params.text_document_position_params;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        self.diagnostic_handler
            .document_highlight(&self.workspace, pos, &uri)
            .await
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        self.action_handler
            .formatting(&self.workspace, params.options, &uri)
            .await
    }

    async fn prepare_rename(
        &self,
        params: TextDocumentPositionParams,
    ) -> Result<Option<PrepareRenameResponse>> {
        let pos = params.position;
        let uri = params.text_document.uri;

        self.action_handler
            .prepare_rename(&self.workspace, pos, &uri)
            .await
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let doc_pos = params.text_document_position;
        let pos = doc_pos.position;
        let uri = doc_pos.text_document.uri;

        self.action_handler
            .rename(&self.workspace, params.new_name, pos, &uri)
            .await
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
