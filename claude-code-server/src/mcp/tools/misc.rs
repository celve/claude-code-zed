use serde_json::Value;
use tracing::info;

use crate::mcp::types::TextContent;

pub fn echo(arguments: &Value) -> Vec<TextContent> {
    let text = arguments
        .get("text")
        .and_then(|v| v.as_str())
        .unwrap_or("No text provided");

    vec![TextContent {
        type_: "text".to_string(),
        text: format!("Echo: {}", text),
    }]
}

pub fn execute_code(arguments: &Value) -> Vec<TextContent> {
    let code = arguments
        .get("code")
        .and_then(|v| v.as_str())
        .unwrap_or("No code provided");

    info!(
        "Executing code: {}",
        code.chars().take(50).collect::<String>()
    );

    vec![TextContent {
        type_: "text".to_string(),
        text: format!(
            "Code executed successfully. Output: (simulated execution of {} characters)",
            code.len()
        ),
    }]
}
