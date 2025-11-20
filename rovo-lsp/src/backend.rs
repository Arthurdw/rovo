use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::handlers;

/// LSP backend implementation for Rovo language server
pub struct Backend {
    /// LSP client for communicating with the editor
    client: Client,
    /// In-memory cache of document contents
    document_map: Arc<RwLock<HashMap<String, String>>>,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn on_change(&self, params: TextDocumentItem) {
        let uri = params.uri.to_string();
        let content = params.text.clone();

        // Store document content
        self.document_map
            .write()
            .await
            .insert(uri.clone(), content.clone());

        // Run diagnostics
        let diagnostics = handlers::text_document_did_change(&content, params.uri.clone());

        // Publish diagnostics
        self.client
            .publish_diagnostics(params.uri, diagnostics, Some(params.version))
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "rovo-lsp".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["@".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Rovo LSP initialized")
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
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.on_change(TextDocumentItem {
                uri: params.text_document.uri.clone(),
                text: change.text.clone(),
                version: params.text_document.version,
                language_id: "rust".to_string(),
            })
            .await;
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let content = {
            let document_map = self.document_map.read().await;
            match document_map.get(&uri) {
                Some(content) => content.clone(),
                None => return Ok(None),
            }
        };

        Ok(handlers::text_document_completion(&content, position))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        let content = {
            let document_map = self.document_map.read().await;
            match document_map.get(&uri) {
                Some(content) => content.clone(),
                None => return Ok(None),
            }
        };

        Ok(handlers::text_document_hover(&content, position))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.to_string();

        let content = {
            let document_map = self.document_map.read().await;
            match document_map.get(&uri) {
                Some(content) => content.clone(),
                None => return Ok(None),
            }
        };

        let mut actions = crate::code_actions::get_code_actions(
            &content,
            params.range,
            params.text_document.uri.clone(),
        );

        // Add diagnostic-specific code actions
        for diagnostic in &params.context.diagnostics {
            actions.extend(crate::code_actions::get_diagnostic_code_actions(
                &content,
                diagnostic,
                params.text_document.uri.clone(),
            ));
        }

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        let content = {
            let document_map = self.document_map.read().await;
            match document_map.get(&uri) {
                Some(content) => content.clone(),
                None => return Ok(None),
            }
        };

        let line_idx = position.line as usize;
        let lines: Vec<&str> = content.lines().collect();

        if line_idx >= lines.len() {
            return Ok(None);
        }

        // Only work near #[rovo]
        if !crate::parser::is_near_rovo_attribute(&content, line_idx) {
            return Ok(None);
        }

        let line = lines[line_idx];
        let char_idx = position.character as usize;

        // Check if cursor is on a type
        if let Some((response_type, _, _)) =
            crate::type_resolver::get_type_at_position(line, char_idx)
        {
            if let Some(type_name) =
                crate::type_resolver::extract_type_from_response(&response_type)
            {
                if let Some(def_line) =
                    crate::type_resolver::find_type_definition(&content, &type_name)
                {
                    // Find the exact position of the type name in the definition line
                    let def_col = lines
                        .get(def_line)
                        .and_then(|l| l.find(&type_name))
                        .unwrap_or(0);

                    let location = Location {
                        uri: params
                            .text_document_position_params
                            .text_document
                            .uri
                            .clone(),
                        range: Range {
                            start: Position {
                                line: def_line as u32,
                                character: def_col as u32,
                            },
                            end: Position {
                                line: def_line as u32,
                                character: (def_col + type_name.len()) as u32,
                            },
                        },
                    };

                    return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let content = {
            let document_map = self.document_map.read().await;
            match document_map.get(&uri) {
                Some(content) => content.clone(),
                None => return Ok(None),
            }
        };

        Ok(handlers::find_tag_references(
            &content,
            position,
            params.text_document_position.text_document.uri,
        ))
    }
}
