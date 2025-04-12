use std::sync::Arc;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use dbuf_lsp::common::ast_access::AstAccess;
use dbuf_lsp::common::default_ast::default_ast;
use dbuf_lsp::common::pretty_printer::PrettyPrinter;

use dbuf_lsp::action_handler::ActionHandler;
use dbuf_lsp::common::handler::Handler;

#[derive(Debug)]
struct Backend {
    client: Arc<Client>,
    ast: AstAccess,
    action_handler: ActionHandler,
}

impl Backend {
    fn new(client: Client) -> Backend {
        let client_arc = Arc::new(client);
        Backend {
            client: client_arc.clone(),
            ast: AstAccess::new(),
            action_handler: ActionHandler::new(client_arc),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, init: InitializeParams) -> Result<InitializeResult> {
        let mut ast = self.ast.write();
        *ast = default_ast();

        let mut capabilities = ServerCapabilities::default();
        capabilities.text_document_sync =
            Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL));

        self.action_handler.init(init, &mut capabilities);

        capabilities.hover_provider = Some(HoverProviderCapability::Simple(true));
        capabilities.completion_provider = Some(CompletionOptions::default());

        eprintln!("init");
        Ok(InitializeResult {
            capabilities,
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
        let mut _ast = self.ast.write();
        eprintln!("WARN: did change is not fully implemented")
    }
    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        // eprintln!("did close: {:?}", params);
        // TODO: remove existing AST
        let _ = params;
        let mut _ast = self.ast.write();
        eprintln!("WARN: did close is not fully implemented");
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        eprintln!("WARN: completition is not fully implemented");
        let _ = params;
        let _ast = self.ast.read();
        eprintln!("ast: {:?}", _ast);
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
        let ast = self.ast.read();

        let mut text = String::new();
        let mut printer = PrettyPrinter::new(&mut text);
        printer.print_module(&ast).expect("serialized");

        let ans = Hover {
            contents: HoverContents::Array(vec![
                MarkedString::LanguageString(LanguageString {
                    language: "dbuf".to_string(),
                    value: text,
                }),
                MarkedString::String("explanation".to_string()),
            ]),
            range: None,
        };
        return Ok(Some(ans));
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        self.action_handler
            .formatting(&self.ast, params.options)
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
