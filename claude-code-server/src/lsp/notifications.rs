use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use tower_lsp::lsp_types::Position;

/// Notification sent when the user's selection changes in the editor
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectionChangedNotification {
    pub text: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "fileUrl")]
    pub file_url: String,
    pub selection: SelectionInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SelectionInfo {
    pub start: Position,
    pub end: Position,
    #[serde(rename = "isEmpty")]
    pub is_empty: bool,
}

/// Notification sent when the user @mentions a file or code range
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AtMentionedNotification {
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "lineStart")]
    pub line_start: u32,
    #[serde(rename = "lineEnd")]
    pub line_end: u32,
}

/// JSON-RPC notification structure for IDE to Claude communication
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// Channel for sending notifications from LSP to MCP
pub type NotificationSender = broadcast::Sender<JsonRpcNotification>;
pub type NotificationReceiver = broadcast::Receiver<JsonRpcNotification>;
