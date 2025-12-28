use serde_json::Value;
use tracing::info;

use crate::mcp::types::TextContent;

pub fn open_file(arguments: &Value) -> Vec<TextContent> {
    let file_path = arguments
        .get("filePath")
        .and_then(|v| v.as_str())
        .unwrap_or("No file path provided");
    let preview = arguments
        .get("preview")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let _start_text = arguments.get("startText").and_then(|v| v.as_str());
    let _end_text = arguments.get("endText").and_then(|v| v.as_str());
    let make_frontmost = arguments
        .get("makeFrontmost")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    info!("Opening file: {} (preview: {})", file_path, preview);

    if make_frontmost {
        vec![TextContent {
            type_: "text".to_string(),
            text: format!("Opened file: {}", file_path),
        }]
    } else {
        let response = serde_json::json!({
            "success": true,
            "filePath": std::path::Path::new(file_path).canonicalize()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| file_path.to_string()),
            "languageId": "text",
            "lineCount": 0
        });

        vec![TextContent {
            type_: "text".to_string(),
            text: response.to_string(),
        }]
    }
}

pub fn get_open_editors() -> Vec<TextContent> {
    info!("Getting open editors");

    let response = serde_json::json!({
        "tabs": []
    });

    vec![TextContent {
        type_: "text".to_string(),
        text: response.to_string(),
    }]
}

pub fn close_all_diff_tabs() -> Vec<TextContent> {
    info!("Closing all diff tabs");

    let closed_count = 0; // Simulate no diff tabs to close

    vec![TextContent {
        type_: "text".to_string(),
        text: format!("CLOSED_{}_DIFF_TABS", closed_count),
    }]
}

pub fn close_tab(arguments: &Value) -> Vec<TextContent> {
    let tab_name = arguments
        .get("tab_name")
        .and_then(|v| v.as_str())
        .unwrap_or("No tab name provided");

    info!("Closing tab: {}", tab_name);

    vec![TextContent {
        type_: "text".to_string(),
        text: "TAB_CLOSED".to_string(),
    }]
}
