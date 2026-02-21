use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
pub struct Backend {
    client: Client,
    document_map: DashMap<String, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "(".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Vidra Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
            language_id: params.text_document.language_id,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if params.content_changes.is_empty() { return; }
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.content_changes.remove(0).text,
            version: params.text_document.version,
            language_id: "vidra".to_string(),
        })
        .await
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            let item = TextDocumentItem {
                uri: params.text_document.uri,
                text,
                version: 0,
                language_id: "vidra".to_string(),
            };
            self.on_change(item).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.document_map.remove(params.text_document.uri.as_str());
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let completions = vec![
            CompletionItem {
                label: "project".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("project(width, height, fps) { ... }".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "scene".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("scene(name, duration) { ... }".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "layer".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("layer(name) { ... }".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "component".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("component(name) { ... }".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "text".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("text(content, font: \"Inter\", size: 32)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "image".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("image(path)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "audio".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("audio(path, trim_start: 0, trim_end: 1, volume: 1)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "video".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("video(path, trim_start: 0, trim_end: 1)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "solid".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("solid(color)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "shape".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("shape(type, fill: #color, stroke: #color, stroke_width: 1)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "position".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("position(x, y)".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "animation".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("animation(property, time, value, ease: \"linear\")".to_string()),
                ..Default::default()
            },
        ];

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(
                MarkedString::String("VidraScript code block".to_string())
            ),
            range: None,
        }))
    }
}

impl Backend {
    async fn on_change(&self, item: TextDocumentItem) {
        let text = item.text.clone();
        let uri = item.uri.clone();
        self.document_map.insert(uri.to_string(), text.clone());

        let mut diagnostics = vec![];

        let tokens = match vidra_lang::Lexer::new(&text).tokenize() {
            Ok(t) => t,
            Err(e) => {
                diagnostics.push(error_to_diagnostic(e));
                self.client.publish_diagnostics(uri.clone(), diagnostics, Some(item.version)).await;
                return;
            }
        };

        let mut parser = vidra_lang::Parser::new(tokens, uri.as_str());
        
        match parser.parse() {
            Ok(ast) => {
                let checker = vidra_lang::TypeChecker::new(uri.as_str().to_string());
                let checker_diags = match checker.check(&ast) {
                    Ok(d) => d,
                    Err(d) => d,
                };
                for diag in checker_diags {
                    diagnostics.push(checker_diag_to_lsp_diag(diag));
                }
            }
            Err(e) => {
                diagnostics.push(error_to_diagnostic(e));
            }
        }

        self.client
            .publish_diagnostics(uri.clone(), diagnostics, Some(item.version))
            .await;
    }
}

fn checker_diag_to_lsp_diag(diag: vidra_lang::checker::Diagnostic) -> Diagnostic {
    let severity = match diag.severity {
        vidra_lang::checker::DiagnosticSeverity::Error => DiagnosticSeverity::ERROR,
        vidra_lang::checker::DiagnosticSeverity::Warning => DiagnosticSeverity::WARNING,
        vidra_lang::checker::DiagnosticSeverity::Info => DiagnosticSeverity::INFORMATION,
    };
    
    let line_idx = diag.span.line.saturating_sub(1) as u32;
    let col_idx = diag.span.column.saturating_sub(1) as u32;
    let length = diag.span.end.saturating_sub(diag.span.start);
    let end_col_idx = (diag.span.column + length).saturating_sub(1) as u32;
    
    Diagnostic::new(
        Range::new(Position::new(line_idx, col_idx), Position::new(line_idx, end_col_idx)),
        Some(severity),
        None, None, diag.message, None, None,
    )
}

fn error_to_diagnostic(err: vidra_core::VidraError) -> Diagnostic {
    match err {
        vidra_core::VidraError::Parse { message, line, column, .. } | vidra_core::VidraError::Type { message, line, column, .. } => {
            let line_idx = line.saturating_sub(1) as u32;
            let col_idx = column.saturating_sub(1) as u32;
            Diagnostic::new(
                Range::new(Position::new(line_idx, col_idx), Position::new(line_idx, col_idx + 1)),
                Some(DiagnosticSeverity::ERROR),
                None, None, message, None, None,
            )
        }
        _ => {
            Diagnostic::new(
                Range::new(Position::new(0, 0), Position::new(0, 0)),
                Some(DiagnosticSeverity::ERROR),
                None, None, err.to_string(), None, None,
            )
        }
    }
}

/// Start the Language Server on standard I/O.
pub async fn start_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        document_map: DashMap::new(),
    });

    Server::new(stdin, stdout, socket).serve(service).await;
}
