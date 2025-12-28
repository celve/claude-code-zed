mod diff;
mod document;
mod editor;
mod misc;
mod selection;
mod workspace;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::types::{SelectionState, TextContent};

/// Dispatch a tool call to the appropriate handler
pub async fn dispatch_tool(
    tool_name: &str,
    arguments: &serde_json::Value,
    selection_state: &Arc<RwLock<Option<SelectionState>>>,
    worktree: &Option<PathBuf>,
) -> Result<Vec<TextContent>, anyhow::Error> {
    let content = match tool_name {
        "echo" => misc::echo(arguments),
        "get_workspace_info" => workspace::get_workspace_info(worktree),
        "getWorkspaceFolders" => workspace::get_workspace_folders(worktree),
        "getCurrentSelection" => selection::get_current_selection(selection_state).await,
        "getLatestSelection" => selection::get_latest_selection(selection_state).await,
        "openFile" => editor::open_file(arguments),
        "getOpenEditors" => editor::get_open_editors(),
        "closeAllDiffTabs" => editor::close_all_diff_tabs(),
        "close_tab" => editor::close_tab(arguments),
        "openDiff" => diff::open_diff(arguments),
        "checkDocumentDirty" => document::check_document_dirty(arguments),
        "saveDocument" => document::save_document(arguments),
        "getDiagnostics" => document::get_diagnostics(arguments),
        "executeCode" => misc::execute_code(arguments),
        _ => return Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
    };

    Ok(content)
}
