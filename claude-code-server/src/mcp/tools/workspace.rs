use std::path::PathBuf;
use tracing::info;

use crate::mcp::types::TextContent;

pub fn get_workspace_info(worktree: &Option<PathBuf>) -> Vec<TextContent> {
    let workspace_info = worktree
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "Unknown workspace".to_string());

    vec![TextContent {
        type_: "text".to_string(),
        text: format!("Current workspace: {}", workspace_info),
    }]
}

pub fn get_workspace_folders(worktree: &Option<PathBuf>) -> Vec<TextContent> {
    let workspace_info = worktree
        .as_ref()
        .map(|p| p.to_string_lossy().to_string())
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| "Unknown workspace".to_string());

    info!("Getting workspace folders");

    let response = serde_json::json!({
        "success": true,
        "folders": [{
            "name": std::path::Path::new(&workspace_info)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("workspace"),
            "uri": format!("file://{}", workspace_info),
            "path": workspace_info
        }],
        "rootPath": workspace_info
    });

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}
