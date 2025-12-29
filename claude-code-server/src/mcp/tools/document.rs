use std::path::PathBuf;
use tracing::info;

use crate::mcp::types::TextContent;

pub fn get_diagnostics(worktree: &Option<PathBuf>) -> Vec<TextContent> {
    info!("Getting diagnostics for workspace: {:?}", worktree);

    // Return empty diagnostics for now
    // TODO: This could be enhanced to collect diagnostics from the LSP
    let response = serde_json::json!({
        "diagnostics": []
    });

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}