use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::mcp::types::{SelectionState, TextContent};

pub async fn get_current_selection(
    selection_state: &Arc<RwLock<Option<SelectionState>>>,
) -> Vec<TextContent> {
    info!("Getting current selection");

    let state = selection_state.read().await;
    let response = if let Some(selection) = state.as_ref() {
        serde_json::json!({
            "success": true,
            "text": selection.text,
            "filePath": selection.file_path,
            "fileUrl": selection.file_url,
            "selection": selection.selection
        })
    } else {
        serde_json::json!({
            "success": false,
            "message": "No active editor found"
        })
    };

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}

pub async fn get_latest_selection(
    selection_state: &Arc<RwLock<Option<SelectionState>>>,
) -> Vec<TextContent> {
    info!("Getting latest selection");

    let state = selection_state.read().await;
    let response = if let Some(selection) = state.as_ref() {
        serde_json::json!({
            "success": true,
            "text": selection.text,
            "filePath": selection.file_path,
            "fileUrl": selection.file_url,
            "selection": selection.selection
        })
    } else {
        serde_json::json!({
            "success": false,
            "message": "No selection available"
        })
    };

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}
