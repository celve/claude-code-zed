mod document;
mod selection;
mod workspace;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{SelectionState, TextContent};

/// Dispatch a tool call to the appropriate handler
pub async fn dispatch_tool(
    tool_name: &str,
    _arguments: &serde_json::Value,
    selection_state: &Arc<RwLock<Option<SelectionState>>>,
    worktree: &Option<PathBuf>,
) -> Result<Vec<TextContent>, anyhow::Error> {
    let content = match tool_name {
        // Working tools
        "getWorkspaceFolders" => workspace::get_workspace_folders(worktree),
        "getCurrentSelection" => selection::get_current_selection(selection_state).await,
        "getLatestSelection" => selection::get_latest_selection(selection_state).await,
        "getDiagnostics" => document::get_diagnostics(worktree),

        // IDE tools not supported in Zed - return graceful response
        "openDiff" | "openFile" | "getOpenEditors" | "closeAllDiffTabs" | "close_tab"
        | "checkDocumentDirty" | "saveDocument" | "echo" | "get_workspace_info"
        | "executeCode" => {
            not_supported_response(tool_name)
        }

        // Unknown tools
        _ => not_supported_response(tool_name),
    };

    Ok(content)
}

fn not_supported_response(tool_name: &str) -> Vec<TextContent> {
    vec![TextContent {
        type_: "text".to_string(),
        text: format!("NOT_SUPPORTED: Tool '{}' is not available in Zed integration. File operations should be performed directly.", tool_name),
    }]
}
