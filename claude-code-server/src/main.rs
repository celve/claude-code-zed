use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{error, info};

mod lsp;
mod mcp;
mod websocket;

use lsp::{run_lsp_server, run_lsp_server_with_notifications};
use websocket::{cleanup_lock_file, run_websocket_server, run_websocket_server_full};

#[derive(Parser)]
#[command(name = "claude-code-server")]
#[command(about = "Claude Code Server - WebSocket and LSP server for Claude Code integration")]
struct Cli {
    #[command(subcommand)]
    mode: Option<Mode>,

    /// Enable debug logging
    #[arg(long, short)]
    debug: bool,

    /// Worktree root path (for LSP mode)
    #[arg(long)]
    worktree: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Mode {
    /// Run as LSP server for Zed extension communication
    Lsp {
        /// Worktree root path
        #[arg(long)]
        worktree: Option<PathBuf>,
    },
    /// Run as standalone WebSocket server for Claude Code CLI
    Websocket {
        /// WebSocket server port (default: 59791)
        #[arg(long, short)]
        port: Option<u16>,
    },
    /// Run both LSP and WebSocket servers
    Hybrid {
        /// WebSocket server port (default: 59791)
        #[arg(long, short)]
        port: Option<u16>,
        /// Worktree root path
        #[arg(long)]
        worktree: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging with enhanced formatting for debugging
    let log_level = if cli.debug {
        tracing::Level::DEBUG
    } else {
        // Check environment variable for log level override
        match std::env::var("RUST_LOG").as_deref() {
            Ok("trace") => tracing::Level::TRACE,
            Ok("debug") => tracing::Level::DEBUG,
            Ok("info") => tracing::Level::INFO,
            Ok("warn") => tracing::Level::WARN,
            Ok("error") => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        }
    };

    let subscriber = tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_writer(std::io::stderr) // Force all logs to stderr for LSP compatibility
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Logging initialized at level: {:?}", log_level);

    info!("Claude Code Server starting...");

    match cli.mode {
        Some(Mode::Lsp { worktree }) => {
            let worktree_path = cli.worktree.or(worktree);
            run_lsp_server(worktree_path).await
        }
        Some(Mode::Websocket { port }) => run_websocket_server(port).await,
        Some(Mode::Hybrid { port, worktree }) => {
            let worktree_path = cli.worktree.or(worktree);
            run_hybrid_server(port, worktree_path).await
        }
        None => {
            // Default mode: try to detect what we should run based on arguments
            if cli.worktree.is_some() {
                info!("No mode specified but worktree provided, running LSP mode...");
                run_lsp_server(cli.worktree).await
            } else {
                info!("No mode specified, running in hybrid mode...");
                run_hybrid_server(None, cli.worktree).await
            }
        }
    }
}

async fn run_hybrid_server(port: Option<u16>, worktree: Option<PathBuf>) -> Result<()> {
    info!("Starting hybrid server (LSP + WebSocket)");
    if let Some(path) = &worktree {
        info!("Worktree path: {}", path.display());
    }

    // Create notification channel for LSP -> WebSocket communication
    let (notification_sender, notification_receiver) = tokio::sync::broadcast::channel(100);
    let notification_sender = std::sync::Arc::new(notification_sender);

    // Create channel to receive the actual bound port from WebSocket server
    let (port_sender, port_receiver) = tokio::sync::oneshot::channel::<u16>();

    // In hybrid mode, we run both servers with notification bridge
    let websocket_handle = tokio::spawn(run_websocket_server_full(
        port,
        worktree.clone(),
        Some(notification_receiver),
        Some(port_sender),
    ));
    let lsp_handle = tokio::spawn(run_lsp_server_with_notifications(
        worktree,
        Some(notification_sender),
    ));

    // Wait to receive the actual port from WebSocket server
    let actual_port = port_receiver.await.ok();
    if let Some(p) = actual_port {
        info!("WebSocket server bound to port {}", p);
    }

    // Wait for either to complete (or fail)
    tokio::select! {
        result = websocket_handle => {
            match result {
                Ok(Ok(())) => info!("WebSocket server completed"),
                Ok(Err(e)) => error!("WebSocket server error: {}", e),
                Err(e) => error!("WebSocket server task panicked: {}", e),
            }
        }
        result = lsp_handle => {
            match result {
                Ok(Ok(())) => info!("LSP server completed"),
                Ok(Err(e)) => error!("LSP server error: {}", e),
                Err(e) => error!("LSP server task panicked: {}", e),
            }

            // LSP server exited - clean up the WebSocket server's lock file
            if let Some(p) = actual_port {
                info!("LSP server exited, cleaning up lock file for port {}", p);
                if let Err(e) = cleanup_lock_file(p).await {
                    error!("Failed to cleanup lock file: {}", e);
                }
            }
        }
    }

    Ok(())
}