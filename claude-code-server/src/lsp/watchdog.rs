use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tower_lsp::{LspService, Server};
use tracing::{error, info, warn};

#[cfg(unix)]
use std::os::unix::process::parent_id;

use super::notifications::NotificationSender;
use super::server::ClaudeCodeLanguageServer;

pub async fn run_lsp_server(worktree: Option<PathBuf>) -> Result<()> {
    run_lsp_server_with_notifications(worktree, None).await
}

/// Spawn a watchdog task that monitors the parent process.
/// If the parent process dies (we get reparented to init/launchd), signals shutdown
/// via the provided `tokio::sync::watch` sender.
///
/// Returns a JoinHandle that completes when parent death is detected.
#[cfg(unix)]
pub fn spawn_parent_watchdog(
    shutdown_sender: Option<tokio::sync::watch::Sender<bool>>,
) -> tokio::task::JoinHandle<()> {
    let initial_ppid = parent_id();
    info!(
        "Starting parent process watchdog (initial PPID: {})",
        initial_ppid
    );

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            let current_ppid = parent_id();

            // If parent PID changed, our original parent (Zed) has died
            // On Unix, orphaned processes are reparented to init (PID 1) or launchd
            if current_ppid != initial_ppid || current_ppid == 1 {
                if current_ppid == 1 {
                    error!("Reparented to init (PPID=1) - parent died, signaling shutdown");
                } else {
                    error!(
                        "Parent process changed from {} to {} - parent likely died, signaling shutdown",
                        initial_ppid, current_ppid
                    );
                }

                // Signal shutdown so that cleanup can happen
                if let Some(sender) = &shutdown_sender {
                    let _ = sender.send(true);
                    // Give a moment for cleanup to happen
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }

                // If cleanup didn't cause exit, force exit
                warn!("Graceful shutdown timed out, forcing exit");
                std::process::exit(0);
            }
        }
    })
}

#[cfg(not(unix))]
pub fn spawn_parent_watchdog(
    _shutdown_sender: Option<tokio::sync::watch::Sender<bool>>,
) -> tokio::task::JoinHandle<()> {
    // On non-Unix platforms, just return a no-op task
    tokio::spawn(async {
        // No parent monitoring on Windows
        std::future::pending::<()>().await;
    })
}

pub async fn run_lsp_server_with_notifications(
    worktree: Option<PathBuf>,
    notification_sender: Option<Arc<NotificationSender>>,
) -> Result<()> {
    run_lsp_server_inner(worktree, notification_sender, None).await
}

/// Run the LSP server with optional shutdown signaling for coordinated cleanup.
///
/// When `shutdown_sender` is provided, the watchdog will signal through it
/// instead of calling `process::exit(0)`, allowing the hybrid server to
/// clean up lock files before exiting.
pub async fn run_lsp_server_inner(
    worktree: Option<PathBuf>,
    notification_sender: Option<Arc<NotificationSender>>,
    shutdown_sender: Option<tokio::sync::watch::Sender<bool>>,
) -> Result<()> {
    info!("Starting LSP server mode");
    if let Some(path) = &worktree {
        info!("Worktree path: {}", path.display());
    }

    // Spawn watchdog to detect parent process death (e.g., after Mac sleep/wake)
    // Pass the shutdown sender so it can signal graceful shutdown
    let _watchdog = spawn_parent_watchdog(shutdown_sender);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| {
        let mut server = ClaudeCodeLanguageServer::new(client, worktree.clone());
        if let Some(sender) = notification_sender.clone() {
            server = server.with_notification_sender(sender);
        }
        server
    });
    Server::new(stdin, stdout, socket).serve(service).await;

    info!("LSP server stopped");
    Ok(())
}
