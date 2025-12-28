use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::lsp::NotificationReceiver;

use super::handlers::create_capabilities;
use super::types::{SelectionState, ServerCapabilities};

pub struct MCPServer {
    pub(crate) capabilities: ServerCapabilities,
    pub(crate) selection_state: Arc<RwLock<Option<SelectionState>>>,
    pub(crate) worktree: Option<PathBuf>,
}

impl MCPServer {
    pub fn new() -> Self {
        Self::with_notifications(None, None)
    }

    pub fn with_notifications(
        receiver: Option<NotificationReceiver>,
        worktree: Option<PathBuf>,
    ) -> Self {
        let capabilities = create_capabilities();
        let selection_state = Arc::new(RwLock::new(None));

        // Spawn background task to listen for notifications
        if let Some(mut rx) = receiver {
            let state = selection_state.clone();
            tokio::spawn(async move {
                while let Ok(notification) = rx.recv().await {
                    if notification.method == "selection_changed" {
                        if let Ok(selection) =
                            serde_json::from_value::<SelectionState>(notification.params.clone())
                        {
                            *state.write().await = Some(selection);
                        }
                    }
                }
            });
        }

        Self {
            capabilities,
            selection_state,
            worktree,
        }
    }
}

impl Default for MCPServer {
    fn default() -> Self {
        Self::new()
    }
}
