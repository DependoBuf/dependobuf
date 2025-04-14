use std::sync::Arc;

use tower_lsp::jsonrpc::{Error, Result};
use tower_lsp::lsp_types::OneOf::*;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use dbuf_lsp::common::ast_access::WorkspaceAccess;

use dbuf_lsp::action_handler::ActionHandler;
use dbuf_lsp::common::handler::Handler;

#[derive(Debug)]
struct Backend {
    client: Arc<Client>,
    workspace: WorkspaceAccess,
    action_handler: ActionHandler,
}

impl Backend {
    fn new(client: Client) -> Backend {
        let client_arc = Arc::new(client);
        Backend {
            client: client_arc.clone(),
            workspace: WorkspaceAccess::new(),
            action_handler: ActionHandler::new(client_arc),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, init: InitializeParams) -> Result<InitializeResult> {
        let mut capabilities = ServerCapabilities::default();
        capabilities.text_document_sync = Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(true),
                change: Some(TextDocumentSyncKind::FULL),
                will_save: Some(false),
                will_save_wait_until: Some(false),
                save: Some(TextDocumentSyncSaveOptions::Supported(false)),
            },
        ));

        self.action_handler.init(init, &mut capabilities);

        capabilities.rename_provider = Some(Left(true));

        // capabilities.hover_provider = Some(HoverProviderCapability::Simple(true));
        // capabilities.completion_provider = Some(CompletionOptions::default());

        eprintln!("init");
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
                .log_message(MessageType::ERROR, "file change is full")
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

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        eprintln!("WARN: completition is not fully implemented");
        Err(Error::method_not_found())
    }

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        eprintln!("WARN: hover is not fully implemented");
        Err(Error::method_not_found())
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        eprintln!("rename with params: {:?}", params);
        let my_edit = TextEdit {
            range: Range::new(Position::new(0, 0), Position::new(0, 1)),
            new_text: "kek".to_string(),
        };
        let edit = TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier {
                uri: params.text_document_position.text_document.uri,
                version: None,
            },
            edits: vec![Left(my_edit)],
        };
        Ok(Some(WorkspaceEdit {
            document_changes: Some(DocumentChanges::Edits(vec![edit])),
            ..Default::default()
        }))
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        self.action_handler
            .formatting(&self.workspace, params.options, &uri)
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
