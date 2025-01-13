use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[allow(unused_imports)]
use dbuf_core;

#[derive(Debug)]
#[allow(dead_code)]
struct Backend {
    client: Client,
    
    ast: Option<()>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        eprintln!("init");
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        eprintln!("inited");
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // eprintln!("did open: {:?}", params);
        // TODO: read params.text_document.text, containing full document text and build AST
        let _ = params;
        eprintln!("WARN: did open is not fully implemented")
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // eprintln!("did change: {:?}", params);
        // TODO: read params.content_changes[0].text, containing full document text and build AST
        let _ = params;
        eprintln!("WARN: did change is not fully implemented")
    }
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // eprintln!("did close: {:?}", params);
        // TODO: remove existing AST
        let _ = params;
        eprintln!("WARN: did close is not fully implemented");
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        eprintln!("WARN: completition is not fully implemented");
        let _ = params;
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem {
                label: "message".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword for message construction".to_string()),
                ..CompletionItem::default()
            },
            CompletionItem {
                label: "enum".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("keyword for enum construction".to_string()),
                ..CompletionItem::default()
            },
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        eprintln!("WARN: hover is not fully implemented");
        let _ = params;
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
            range: None,
        }))
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        ast: None,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
