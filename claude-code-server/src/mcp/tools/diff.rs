use serde_json::Value;
use tracing::info;

use crate::mcp::types::TextContent;

pub fn open_diff(arguments: &Value) -> Vec<TextContent> {
    let old_file_path = arguments
        .get("old_file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("No old file path provided");
    let new_file_path = arguments
        .get("new_file_path")
        .and_then(|v| v.as_str())
        .unwrap_or("No new file path provided");
    let new_file_contents = arguments
        .get("new_file_contents")
        .and_then(|v| v.as_str())
        .unwrap_or("No new file contents provided");
    let _tab_name = arguments
        .get("tab_name")
        .and_then(|v| v.as_str())
        .unwrap_or("diff");

    info!("Opening diff for {} vs {}", old_file_path, new_file_path);

    // Always respond with FILE_SAVED to simulate accepting the diff
    vec![
        TextContent {
            type_: "text".to_string(),
            text: "FILE_SAVED".to_string(),
        },
        TextContent {
            type_: "text".to_string(),
            text: new_file_contents.to_string(),
        },
    ]
}
