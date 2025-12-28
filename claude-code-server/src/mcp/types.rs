use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<MCPError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: Option<ToolsCapability>,
    pub prompts: Option<PromptsCapability>,
    pub logging: Option<LoggingCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptsCapability {
    #[serde(rename = "listChanged")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoggingCapability {}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TextContent {
    #[serde(rename = "type")]
    pub type_: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectionState {
    pub text: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "fileUrl")]
    pub file_url: String,
    pub selection: SelectionRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectionRange {
    pub start: SelectionPosition,
    pub end: SelectionPosition,
    #[serde(rename = "isEmpty")]
    pub is_empty: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SelectionPosition {
    pub line: u32,
    pub character: u32,
}
