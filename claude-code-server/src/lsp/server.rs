use std::path::PathBuf;
use std::sync::Arc;
use tower_lsp::Client;
use tracing::debug;

use super::notifications::{JsonRpcNotification, NotificationSender};

#[derive(Debug)]
pub struct ClaudeCodeLanguageServer {
    pub(crate) client: Client,
    #[allow(dead_code)]
    pub(crate) worktree: Option<PathBuf>,
    pub(crate) notification_sender: Option<Arc<NotificationSender>>,
}

impl ClaudeCodeLanguageServer {
    pub fn new(client: Client, worktree: Option<PathBuf>) -> Self {
        Self {
            client,
            worktree,
            notification_sender: None,
        }
    }

    pub fn with_notification_sender(mut self, sender: Arc<NotificationSender>) -> Self {
        self.notification_sender = Some(sender);
        self
    }

    pub(crate) async fn send_notification(&self, method: &str, params: serde_json::Value) {
        if let Some(sender) = &self.notification_sender {
            let notification = JsonRpcNotification {
                jsonrpc: "2.0".to_string(),
                method: method.to_string(),
                params,
            };

            if let Err(e) = sender.send(notification) {
                debug!("Failed to send notification: {}", e);
            }
        }
    }
}
