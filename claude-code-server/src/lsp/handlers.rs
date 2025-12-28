use serde_json::Value;
use tower_lsp::jsonrpc::Result as LspResult;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;
use tracing::info;

use super::notifications::{AtMentionedNotification, SelectionChangedNotification, SelectionInfo};
use super::server::ClaudeCodeLanguageServer;
use super::utils::read_text_from_range;

#[tower_lsp::async_trait]
impl LanguageServer for ClaudeCodeLanguageServer {
    async fn initialize(&self, params: InitializeParams) -> LspResult<InitializeResult> {
        info!("LSP Server initializing...");

        // Log client capabilities to understand what Zed supports
        info!("=== Client Capabilities ===");

        // Window capabilities (includes showDocument for LSP 3.16+)
        if let Some(window) = &params.capabilities.window {
            info!("Window capabilities: {:?}", window);
            if let Some(show_document) = &window.show_document {
                info!("  showDocument support: {:?}", show_document);
            } else {
                info!("  showDocument: NOT SUPPORTED");
            }
            if let Some(work_done_progress) = &window.work_done_progress {
                info!("  workDoneProgress: {}", work_done_progress);
            }
        } else {
            info!("Window capabilities: NONE");
        }

        // Workspace capabilities
        if let Some(workspace) = &params.capabilities.workspace {
            info!("Workspace capabilities:");
            if let Some(apply_edit) = &workspace.apply_edit {
                info!("  applyEdit: {}", apply_edit);
            }
            if let Some(workspace_edit) = &workspace.workspace_edit {
                info!("  workspaceEdit: {:?}", workspace_edit);
            }
            if let Some(did_change_config) = &workspace.did_change_configuration {
                info!("  didChangeConfiguration: {:?}", did_change_config);
            }
            if let Some(workspace_folders) = &workspace.workspace_folders {
                info!("  workspaceFolders: {}", workspace_folders);
            }
        }

        // Text document capabilities
        if let Some(text_doc) = &params.capabilities.text_document {
            info!("TextDocument capabilities (summary):");
            if text_doc.synchronization.is_some() {
                info!("  synchronization: supported");
            }
            if text_doc.completion.is_some() {
                info!("  completion: supported");
            }
            if text_doc.hover.is_some() {
                info!("  hover: supported");
            }
            if text_doc.code_action.is_some() {
                info!("  codeAction: supported");
            }
            if text_doc.publish_diagnostics.is_some() {
                info!("  publishDiagnostics: supported");
            }
        }

        // General capabilities (LSP version info)
        if let Some(general) = &params.capabilities.general {
            info!("General capabilities: {:?}", general);
        }

        info!("=== End Client Capabilities ===");

        if let Some(workspace_folders) = &params.workspace_folders {
            for folder in workspace_folders {
                info!("Workspace folder: {}", folder.uri);
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["@".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec![
                        "claude-code.explain".to_string(),
                        "claude-code.improve".to_string(),
                        "claude-code.fix".to_string(),
                        "claude-code.at-mention".to_string(),
                    ],
                    work_done_progress_options: Default::default(),
                }),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "Claude Code Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Claude Code LSP server initialized!");

        self.client
            .log_message(MessageType::INFO, "Claude Code Language Server is ready!")
            .await;
    }

    async fn shutdown(&self) -> LspResult<()> {
        info!("LSP Server shutting down...");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        info!("Document opened: {}", params.text_document.uri);

        self.client
            .log_message(
                MessageType::INFO,
                format!("Opened document: {}", params.text_document.uri),
            )
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        info!("Document changed: {}", params.text_document.uri);
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        info!("Document saved: {}", params.text_document.uri);
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        info!("Document closed: {}", params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> LspResult<Option<Hover>> {
        let position = params.text_document_position_params.position;
        info!(
            "Hover requested at {}:{}",
            position.line, position.character
        );

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> LspResult<Option<CompletionResponse>> {
        let position = params.text_document_position.position;
        info!(
            "Completion requested at {}:{}",
            position.line, position.character
        );

        let completions = vec![
            CompletionItem {
                label: "@claude explain".to_string(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("Explain this code with Claude".to_string()),
                documentation: Some(Documentation::String(
                    "Ask Claude to explain the selected code or current context".to_string(),
                )),
                insert_text: Some("@claude explain".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "@claude improve".to_string(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("Improve this code with Claude".to_string()),
                documentation: Some(Documentation::String(
                    "Ask Claude to suggest improvements for the selected code".to_string(),
                )),
                insert_text: Some("@claude improve".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "@claude fix".to_string(),
                kind: Some(CompletionItemKind::TEXT),
                detail: Some("Fix issues in this code with Claude".to_string()),
                documentation: Some(Documentation::String(
                    "Ask Claude to identify and fix issues in the selected code".to_string(),
                )),
                insert_text: Some("@claude fix".to_string()),
                ..Default::default()
            },
        ];

        Ok(Some(CompletionResponse::Array(completions)))
    }

    async fn code_action(&self, params: CodeActionParams) -> LspResult<Option<CodeActionResponse>> {
        info!("Code action requested for range: {:?}", params.range);

        // Send selection_changed notification when code action is requested
        let selected_text = read_text_from_range(params.text_document.uri.path(), params.range);
        let selection_notification = SelectionChangedNotification {
            text: selected_text,
            file_path: params.text_document.uri.path().to_string(),
            file_url: params.text_document.uri.to_string(),
            selection: SelectionInfo {
                start: params.range.start,
                end: params.range.end,
                is_empty: params.range.start == params.range.end,
            },
        };

        info!(
            "Sending selection_changed notification for range: {:?}",
            params.range
        );
        self.send_notification(
            "selection_changed",
            serde_json::to_value(selection_notification).unwrap(),
        )
        .await;

        let actions = vec![CodeActionOrCommand::CodeAction(CodeAction {
            title: "Explain with Claude".to_string(),
            kind: Some(CodeActionKind::REFACTOR),
            diagnostics: None,
            edit: None,
            command: None,
            is_preferred: Some(false),
            disabled: None,
            data: Some(serde_json::json!({
                "action": "explain",
                "uri": params.text_document.uri,
                "range": params.range
            })),
        })];

        Ok(Some(actions))
    }

    async fn execute_command(&self, params: ExecuteCommandParams) -> LspResult<Option<Value>> {
        info!("Execute command: {}", params.command);

        match params.command.as_str() {
            "claude-code.explain" => {
                self.client
                    .show_message(
                        MessageType::INFO,
                        "Claude Code: Explain command executed (not yet implemented)",
                    )
                    .await;
            }
            "claude-code.improve" => {
                self.client
                    .show_message(
                        MessageType::INFO,
                        "Claude Code: Improve command executed (not yet implemented)",
                    )
                    .await;
            }
            "claude-code.fix" => {
                self.client
                    .show_message(
                        MessageType::INFO,
                        "Claude Code: Fix command executed (not yet implemented)",
                    )
                    .await;
            }
            "claude-code.at-mention" => {
                info!(
                    "At-mention command executed with args: {:?}",
                    params.arguments
                );

                // Parse arguments to extract file path and line range
                if let Some(args) = params.arguments.first() {
                    if let Ok(mention_data) =
                        serde_json::from_value::<serde_json::Value>(args.clone())
                    {
                        let file_path = mention_data
                            .get("filePath")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let line_start = mention_data
                            .get("lineStart")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;
                        let line_end = mention_data
                            .get("lineEnd")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32;

                        let at_mention_notification = AtMentionedNotification {
                            file_path: file_path.to_string(),
                            line_start,
                            line_end,
                        };

                        self.send_notification(
                            "at_mentioned",
                            serde_json::to_value(at_mention_notification).unwrap(),
                        )
                        .await;

                        self.client
                            .show_message(
                                MessageType::INFO,
                                format!(
                                    "At-mention sent for {}:{}-{}",
                                    file_path, line_start, line_end
                                ),
                            )
                            .await;
                    }
                }
            }
            _ => {
                self.client
                    .show_message(
                        MessageType::WARNING,
                        format!("Unknown command: {}", params.command),
                    )
                    .await;
            }
        }

        Ok(None)
    }

    async fn selection_range(
        &self,
        params: SelectionRangeParams,
    ) -> LspResult<Option<Vec<SelectionRange>>> {
        info!(
            "Selection range requested for {} positions",
            params.positions.len()
        );

        // For each position, create a selection range and notify about the selection
        let mut ranges = Vec::new();

        for position in &params.positions {
            info!("Selection at {}:{}", position.line, position.character);

            // Create a basic selection range (this would normally be more sophisticated)
            let range = Range {
                start: *position,
                end: Position {
                    line: position.line,
                    character: position.character + 1,
                },
            };

            ranges.push(SelectionRange {
                range,
                parent: None,
            });

            // Send selection_changed notification
            let selection_range = Range {
                start: *position,
                end: Position {
                    line: position.line,
                    character: position.character + 1,
                },
            };
            let selected_text =
                read_text_from_range(params.text_document.uri.path(), selection_range);
            let selection_notification = SelectionChangedNotification {
                text: selected_text,
                file_path: params.text_document.uri.path().to_string(),
                file_url: params.text_document.uri.to_string(),
                selection: SelectionInfo {
                    start: *position,
                    end: Position {
                        line: position.line,
                        character: position.character + 1,
                    },
                    is_empty: true,
                },
            };

            self.send_notification(
                "selection_changed",
                serde_json::to_value(selection_notification).unwrap(),
            )
            .await;
        }

        Ok(Some(ranges))
    }
}
