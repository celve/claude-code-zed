use serde_json::Value;
use tracing::info;

use crate::mcp::types::TextContent;

pub fn check_document_dirty(arguments: &Value) -> Vec<TextContent> {
    let file_path = arguments
        .get("filePath")
        .and_then(|v| v.as_str())
        .unwrap_or("No file path provided");

    info!("Checking if document is dirty: {}", file_path);

    let response = serde_json::json!({
        "success": true,
        "filePath": file_path,
        "isDirty": false,
        "isUntitled": false
    });

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}

pub fn save_document(arguments: &Value) -> Vec<TextContent> {
    let file_path = arguments
        .get("filePath")
        .and_then(|v| v.as_str())
        .unwrap_or("No file path provided");

    info!("Saving document: {}", file_path);

    let response = serde_json::json!({
        "success": true,
        "filePath": file_path,
        "saved": true,
        "message": "Document saved successfully"
    });

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}

pub fn get_diagnostics(arguments: &Value) -> Vec<TextContent> {
    let uri = arguments.get("uri").and_then(|v| v.as_str());

    info!("Getting diagnostics for: {:?}", uri);

    let response = if let Some(uri) = uri {
        serde_json::json!([{
            "uri": uri,
            "diagnostics": []
        }])
    } else {
        serde_json::json!([])
    };

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}
