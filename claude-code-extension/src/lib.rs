use zed_extension_api::{
    current_platform, download_file, latest_github_release, lsp::*, make_file_executable,
    Architecture, DownloadedFileType, GithubReleaseOptions, Os, *,
};

// Development configuration
// Set this to true to always use local development binaries instead of GitHub releases
// This allows using local fixes without waiting for official releases
// DEFAULT: false (production behavior - downloads from GitHub)
const FORCE_DEVELOPMENT_MODE: bool = false;

struct ClaudeCodeExtension;

impl Extension for ClaudeCodeExtension {
    fn new() -> Self {
        eprintln!("🎉 [INIT] Claude Code Extension: Extension loaded!");
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command, String> {
        match language_server_id.as_ref() {
            "claude-code-server" => {
                eprintln!(
                    "🚀 [INFO] Claude Code Extension: Starting claude-code-server for worktree: {}",
                    worktree.root_path()
                );

                // In development, we'll try to find the binary in the workspace
                // In production, this would be a distributed binary
                let server_path = find_server_binary(worktree)?;

                Ok(Command {
                    command: server_path,
                    args: vec![
                        "--debug".to_string(),
                        "--worktree".to_string(),
                        worktree.root_path().to_string(),
                        "hybrid".to_string(),
                    ],
                    env: Default::default(),
                })
            }
            _ => Err(format!("Unknown language server: {}", language_server_id)),
        }
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>, String> {
        match language_server_id.as_ref() {
            "claude-code-server" => {
                eprintln!("🔧 [DEBUG] Setting up initialization options for claude-code-server");

                let options = serde_json::json!({
                    "workspaceFolders": [{
                        "uri": format!("file://{}", worktree.root_path()),
                        "name": worktree.root_path().split('/').last().unwrap_or("workspace")
                    }],
                    "claudeCode": {
                        "enabled": true,
                        "extensionVersion": "0.1.0",
                        "ideName": "Zed"
                    }
                });

                Ok(Some(options))
            }
            _ => Ok(None),
        }
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>, String> {
        match language_server_id.as_ref() {
            "claude-code-server" => {
                let config = serde_json::json!({
                    "claudeCode": {
                        "enabled": true,
                        "debug": true,
                        "websocket": {
                            "host": "127.0.0.1",
                            "portRange": [10000, 65535]
                        },
                        "auth": {
                            "generateTokens": true
                        }
                    }
                });

                Ok(Some(config))
            }
            _ => Ok(None),
        }
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        _completion: Completion,
    ) -> Option<CodeLabel> {
        None
    }

    fn label_for_symbol(
        &self,
        _language_server_id: &LanguageServerId,
        _symbol: Symbol,
    ) -> Option<CodeLabel> {
        None
    }
}

/// Find the claude-code-server binary - downloads from GitHub releases if needed
fn find_server_binary(worktree: &Worktree) -> Result<String, String> {
    let worktree_root = worktree.root_path();

    eprintln!(
        "🔍 [DEBUG] find_server_binary called with worktree_root: {}",
        worktree_root
    );
    eprintln!(
        "🔍 [DEBUG] FORCE_DEVELOPMENT_MODE: {}",
        FORCE_DEVELOPMENT_MODE
    );
    eprintln!(
        "🔍 [DEBUG] Checking if '{}' contains 'claude-code-zed'",
        worktree_root
    );

    // For development: look for manually copied binary in extension work directory
    // Check both the directory name AND the development flag
    if worktree_root.contains("claude-code-zed") || FORCE_DEVELOPMENT_MODE {
        if FORCE_DEVELOPMENT_MODE {
            eprintln!("✅ [DEBUG] Development mode FORCED via FORCE_DEVELOPMENT_MODE flag");
        } else {
            eprintln!("✅ [DEBUG] Detected development environment (claude-code-zed in path)");
        }

        // Check for manually copied development binary in extension work directory
        // This allows developers to use their local build with fixes
        let dev_binary_name =
            get_platform_binary_prefix().unwrap_or("claude-code-server".to_string());
        eprintln!(
            "🔍 [DEBUG] Looking for development binary: {}",
            dev_binary_name
        );

        // The binary should be manually copied to the extension work directory
        // We'll return the expected path and let the download logic handle it
        eprintln!("💡 [INFO] Development mode detected!");
        eprintln!("📋 [INFO] To use your local development build:");
        eprintln!("   1. Build the server: cd claude-code-server && cargo build");
        eprintln!(
            "   2. Copy binary to: ~/.../Zed/extensions/work/claude-code-zed/{}",
            dev_binary_name
        );
        eprintln!("   3. Or let the extension download the GitHub release");

        // Return the expected path - download_server_binary will handle checking if it exists
        return Ok(dev_binary_name);
    } else {
        eprintln!("ℹ️ [INFO] Not in development environment, downloading from GitHub releases");
        eprintln!(
            "🔍 [DEBUG] Worktree path '{}' does not contain 'claude-code-zed'",
            worktree_root
        );
    }

    // For production: download binary from GitHub releases
    download_server_binary()
}

/// Download claude-code-server binary from GitHub releases
/// Binary naming format: claude-code-server-<platform>-<version>
/// e.g., claude-code-server-macos-aarch64-v0.1.0
fn download_server_binary() -> Result<String, String> {
    const GITHUB_REPO: &str = "celve/claude-code-zed";

    // Determine platform-specific binary prefix (without version)
    let binary_prefix = match get_platform_binary_prefix() {
        Ok(name) => {
            eprintln!("🔍 [DEBUG] Platform binary prefix: {}", name);
            name
        }
        Err(e) => {
            eprintln!("❌ [ERROR] Failed to determine platform binary prefix: {}", e);
            return Err(e);
        }
    };

    // Get the latest release from GitHub
    eprintln!("🔍 [DEBUG] Fetching latest release from GitHub repo: {}", GITHUB_REPO);
    let release = latest_github_release(
        GITHUB_REPO,
        GithubReleaseOptions {
            require_assets: true,
            pre_release: false,
        },
    )
    .map_err(|e| {
        eprintln!("❌ [ERROR] Failed to fetch GitHub release: {}", e);
        format!("Failed to get latest release: {}", e)
    })?;

    eprintln!(
        "📥 [INFO] Found release {} with {} assets",
        release.version,
        release.assets.len()
    );

    // Expected binary name with version included
    let versioned_binary_name = format!("{}-{}", binary_prefix, release.version);
    eprintln!("🔍 [DEBUG] Expected versioned binary: {}", versioned_binary_name);

    // Check if we already have this exact version
    if std::path::Path::new(&versioned_binary_name).exists() {
        eprintln!("✅ [INFO] Binary {} is up to date", versioned_binary_name);
        if let Err(e) = make_file_executable(&versioned_binary_name) {
            eprintln!("⚠️ [WARNING] Failed to make binary executable: {}", e);
        }
        return Ok(versioned_binary_name);
    }

    // Check for and clean up old versions (with version suffix)
    if let Some(old_binary) = find_existing_binary(&binary_prefix) {
        eprintln!("🔄 [INFO] Found old version: {}, will update to {}", old_binary, release.version);
        if let Err(e) = std::fs::remove_file(&old_binary) {
            eprintln!("⚠️ [WARNING] Failed to remove old binary {}: {}", old_binary, e);
        } else {
            eprintln!("🗑️ [INFO] Removed old binary: {}", old_binary);
        }
    }

    // Also clean up legacy non-versioned binary (from old code before version embedding)
    if std::path::Path::new(&binary_prefix).exists() {
        eprintln!("🔄 [INFO] Found legacy non-versioned binary: {}", binary_prefix);
        if let Err(e) = std::fs::remove_file(&binary_prefix) {
            eprintln!("⚠️ [WARNING] Failed to remove legacy binary {}: {}", binary_prefix, e);
        } else {
            eprintln!("🗑️ [INFO] Removed legacy binary: {}", binary_prefix);
        }
    }

    // Log all available assets for debugging
    eprintln!("🔍 [DEBUG] Available assets:");
    for asset in &release.assets {
        eprintln!("  - {}", asset.name);
    }

    // Find the asset that matches our platform (GitHub releases use non-versioned names)
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == binary_prefix)
        .ok_or_else(|| {
            eprintln!("❌ [ERROR] Asset {} not found in release", binary_prefix);
            eprintln!("🔍 [DEBUG] Looking for asset matching: {}", binary_prefix);
            format!("Asset {} not found in release", binary_prefix)
        })?;

    eprintln!("✅ [SUCCESS] Found matching asset: {}", asset.name);
    eprintln!("🔍 [DEBUG] Download URL: {}", asset.download_url);

    // Download to versioned filename
    eprintln!("🔍 [DEBUG] Downloading to: {}", versioned_binary_name);

    match download_file(
        &asset.download_url,
        &versioned_binary_name,
        DownloadedFileType::Uncompressed,
    ) {
        Ok(_) => {
            eprintln!("✅ [SUCCESS] Binary downloaded to: {}", versioned_binary_name);

            // Make the binary executable
            eprintln!("🔍 [DEBUG] Making binary executable: {}", versioned_binary_name);
            make_file_executable(&versioned_binary_name).map_err(|e| {
                eprintln!("❌ [ERROR] Failed to make binary executable: {}", e);
                format!("Failed to make binary executable: {}", e)
            })?;

            eprintln!("✅ [SUCCESS] Binary {} is ready", versioned_binary_name);
            Ok(versioned_binary_name)
        }
        Err(e) => {
            eprintln!("❌ [ERROR] Failed to download binary: {}", e);
            eprintln!("🔍 [DEBUG] Download error details: {}", e);

            // Fallback to system PATH
            eprintln!("🔄 [FALLBACK] Using system binary: claude-code-server");
            Ok("claude-code-server".to_string())
        }
    }
}

/// Get platform-specific binary prefix for GitHub releases (without version)
/// e.g., "claude-code-server-macos-aarch64"
fn get_platform_binary_prefix() -> Result<String, String> {
    // Use Zed's platform detection instead of env::consts which returns wasm32
    let (os, arch) = current_platform();

    match (os, arch) {
        (Os::Mac, Architecture::Aarch64) => Ok("claude-code-server-macos-aarch64".to_string()),
        (Os::Mac, Architecture::X8664) => Ok("claude-code-server-macos-x86_64".to_string()),
        (Os::Linux, Architecture::X8664) => Ok("claude-code-server-linux-x86_64".to_string()),
        (Os::Windows, _) => Err("Windows is not currently supported".to_string()),
        (os, arch) => Err(format!("Unsupported platform: {:?}-{:?}", os, arch)),
    }
}

/// Find an existing binary that matches the prefix pattern
/// Returns the filename if found (e.g., "claude-code-server-macos-aarch64-v0.1.0")
fn find_existing_binary(prefix: &str) -> Option<String> {
    // Read current directory entries
    let entries = std::fs::read_dir(".").ok()?;

    for entry in entries.flatten() {
        let filename = entry.file_name().to_string_lossy().to_string();
        // Match files that start with prefix and have a version suffix (e.g., "-v0.1.0")
        if filename.starts_with(prefix) && filename.len() > prefix.len() {
            let suffix = &filename[prefix.len()..];
            // Check if suffix looks like a version (starts with "-v")
            if suffix.starts_with("-v") {
                eprintln!("🔍 [DEBUG] Found existing binary: {}", filename);
                return Some(filename);
            }
        }
    }

    None
}

zed_extension_api::register_extension!(ClaudeCodeExtension);
