use dbuf_core::ast::parsed::definition::Definition;
use dbuf_core::ast::parsed::{TypeDeclaration, TypeDefinition};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use dbuf_lsp::ast_access::AstAccess;
use dbuf_lsp::common::ast_builder::AstBuilder;
use dbuf_lsp::common::pretty_printer::PrettyWriter;

#[derive(Debug)]
struct Backend {
    client: Client,

    ast: AstAccess,
}

impl Backend {
    fn new(client: Client) -> Backend {
        Backend {
            client,
            ast: AstAccess::new(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let mut _ast = self.ast.write();
        _ast.push(Definition {
            loc: (),
            name: "kek".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![]),
            },
        });

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
        let mut _ast = self.ast.write();
        _ast.push(Definition {
            loc: (),
            name: "kek_open".to_string(),
            data: TypeDeclaration {
                dependencies: vec![],
                body: TypeDefinition::Message(vec![]),
            },
        });
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
        let _ast = self.ast.read();

        let mut builder = AstBuilder::new();
        builder
            .with_message("Example")
            .expect("")
            .with_dependency("d1", "String")
            .with_field("f1", "Int")
            .with_field("f2", "Int")
            .with_field("f3", "Int");

        builder.with_message("Empty");

        let mut ast = builder.construct();
        let mut buffer = vec![];
        let mut printer = PrettyWriter::new(&mut buffer);
        printer.parse_module(&mut ast).expect("serialized");

        let text = String::from_utf8(buffer).unwrap();

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
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}
