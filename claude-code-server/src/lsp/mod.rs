mod handlers;
mod notifications;
mod server;
mod utils;
mod watchdog;

// Re-export public items
pub use notifications::NotificationReceiver;
pub use watchdog::{run_lsp_server, run_lsp_server_inner};
