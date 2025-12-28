use anyhow::Result;
use serde_json::Value;
use tracing::{debug, info};

use super::tools::dispatch_tool;
use super::types::{
    LoggingCapability, MCPError, MCPRequest, MCPResponse, PromptsCapability, ServerCapabilities,
    ServerInfo, Tool, ToolsCapability,
};
use super::MCPServer;

impl MCPServer {
    pub async fn handle_request(&self, request: MCPRequest) -> Result<MCPResponse> {
        info!("Handling MCP request: {}", request.method);
        debug!("Request params: {:?}", request.params);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(request.params).await?,
            "tools/list" => self.handle_tools_list().await?,
            "tools/call" => self.handle_tools_call(request.params).await?,
            "logging/setLevel" => self.handle_logging_set_level(request.params).await?,
            "prompts/list" => self.handle_prompts_list().await?,
            "prompts/get" => self.handle_prompts_get(request.params).await?,
            _ => {
                return Ok(MCPResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: None,
                    error: Some(MCPError {
                        code: -32601,
                        message: format!("Method not found: {}", request.method),
                        data: None,
                    }),
                });
            }
        };

        Ok(MCPResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(result),
            error: None,
        })
    }

    async fn handle_initialize(&self, params: Option<Value>) -> Result<Value> {
        info!("Initializing MCP session");

        if let Some(params) = params {
            debug!("Initialize params: {}", params);
        }

        Ok(serde_json::json!({
            "protocolVersion": "2025-03-26",
            "capabilities": self.capabilities,
            "serverInfo": ServerInfo {
                name: "claude-code-server".to_string(),
                version: "0.1.0".to_string()
            }
        }))
    }

    async fn handle_tools_list(&self) -> Result<Value> {
        info!("Listing available tools");

        let tools: Vec<Tool> = vec![];

        Ok(serde_json::json!({
            "tools": tools
        }))
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters for tools/call"))?;

        let tool_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tool name"))?;

        let default_args = serde_json::json!({});
        let arguments = params.get("arguments").unwrap_or(&default_args);

        info!("Calling tool: {}", tool_name);
        debug!("Tool arguments: {}", arguments);

        let content =
            dispatch_tool(tool_name, arguments, &self.selection_state, &self.worktree).await?;

        Ok(serde_json::json!({
            "content": content,
            "isError": false
        }))
    }

    async fn handle_logging_set_level(&self, params: Option<Value>) -> Result<Value> {
        if let Some(params) = params {
            let level = params
                .get("level")
                .and_then(|v| v.as_str())
                .unwrap_or("info");
            info!("Setting log level to: {}", level);
        }

        Ok(serde_json::json!({}))
    }

    async fn handle_prompts_list(&self) -> Result<Value> {
        info!("Listing available prompts");

        Ok(serde_json::json!({
            "prompts": []
        }))
    }

    async fn handle_prompts_get(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow::anyhow!("Missing parameters for prompts/get"))?;

        let prompt_name = params
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing prompt name"))?;

        info!("Getting prompt: {}", prompt_name);

        Ok(serde_json::json!({
            "description": format!("Prompt: {}", prompt_name),
            "messages": []
        }))
    }
}

pub fn create_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        prompts: Some(PromptsCapability {
            list_changed: Some(false),
        }),
        logging: Some(LoggingCapability {}),
    }
}
