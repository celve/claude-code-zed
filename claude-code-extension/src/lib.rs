use zed_extension_api::{
    current_platform, download_file, latest_github_release, lsp::*, make_file_executable,
    Architecture, DownloadedFileType, GithubReleaseOptions, Os, *,
};

// Development configuration
// Set this to true to always use local development binaries instead of GitHub releases
// This allows using local fixes without waiting for official releases
// DEFAULT: false (production behavior - downloads from GitHub)
const FORCE_DEVELOPMENT_MODE: bool = false;

// Version file name to track installed binary version
const VERSION_FILE: &str = ".claude-code-server-version";

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
            get_platform_binary_name().unwrap_or("claude-code-server".to_string());
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
fn download_server_binary() -> Result<String, String> {
    const GITHUB_REPO: &str = "celve/claude-code-zed";

    // Determine platform-specific binary name
    let binary_name = match get_platform_binary_name() {
        Ok(name) => {
            eprintln!("🔍 [DEBUG] Platform binary name: {}", name);
            name
        }
        Err(e) => {
            eprintln!("❌ [ERROR] Failed to determine platform binary name: {}", e);
            return Err(e);
        }
    };

    let binary_exists = std::path::Path::new(&binary_name).exists();
    let local_version = read_local_version();

    eprintln!("🔍 [DEBUG] Binary exists: {}", binary_exists);
    eprintln!("🔍 [DEBUG] Local version: {:?}", local_version);

    // Get the latest release from GitHub to check version
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

    // Check if we need to download/update
    let needs_download = if !binary_exists {
        eprintln!("📥 [INFO] Binary not found, will download");
        true
    } else if let Some(ref local_ver) = local_version {
        if is_newer_version(local_ver, &release.version) {
            eprintln!(
                "🔄 [INFO] Update available: {} -> {}",
                local_ver, release.version
            );
            true
        } else {
            eprintln!(
                "✅ [INFO] Binary is up to date (local: {}, remote: {})",
                local_ver, release.version
            );
            false
        }
    } else {
        // Binary exists but no version file - assume it needs update to track version
        eprintln!("⚠️ [INFO] No version file found, will re-download to track version");
        true
    };

    // If binary is up to date, use it
    if !needs_download {
        eprintln!("✅ [SUCCESS] Using existing binary: {}", binary_name);
        if let Err(e) = make_file_executable(&binary_name) {
            eprintln!("⚠️ [WARNING] Failed to make binary executable: {}", e);
        }
        return Ok(binary_name);
    }

    // Log all available assets for debugging
    eprintln!("🔍 [DEBUG] Available assets:");
    for asset in &release.assets {
        eprintln!("  - {}", asset.name);
    }

    // Find the asset that matches our platform
    let asset = release
        .assets
        .iter()
        .find(|asset| asset.name == binary_name)
        .ok_or_else(|| {
            eprintln!("❌ [ERROR] Asset {} not found in release", binary_name);
            eprintln!("🔍 [DEBUG] Looking for asset matching: {}", binary_name);
            format!("Asset {} not found in release", binary_name)
        })?;

    eprintln!("✅ [SUCCESS] Found matching asset: {}", asset.name);
    eprintln!("🔍 [DEBUG] Download URL: {}", asset.download_url);

    // Download the binary to the extension's working directory
    let local_path = binary_name.clone();
    eprintln!("🔍 [DEBUG] Downloading to local path: {}", local_path);

    match download_file(
        &asset.download_url,
        &local_path,
        DownloadedFileType::Uncompressed,
    ) {
        Ok(_) => {
            eprintln!("✅ [SUCCESS] Binary downloaded to: {}", local_path);

            // Make the binary executable
            eprintln!("🔍 [DEBUG] Making binary executable: {}", local_path);
            make_file_executable(&local_path).map_err(|e| {
                eprintln!("❌ [ERROR] Failed to make binary executable: {}", e);
                format!("Failed to make binary executable: {}", e)
            })?;

            // Save the version for future comparisons
            if let Err(e) = write_local_version(&release.version) {
                eprintln!("⚠️ [WARNING] Failed to save version file: {}", e);
            } else {
                eprintln!("✅ [SUCCESS] Version {} saved to {}", release.version, VERSION_FILE);
            }

            eprintln!("✅ [SUCCESS] Binary is now executable");
            Ok(local_path)
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

/// Get platform-specific binary name for GitHub releases
fn get_platform_binary_name() -> Result<String, String> {
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

/// Read the locally stored version from the version file
fn read_local_version() -> Option<String> {
    std::fs::read_to_string(VERSION_FILE)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Write the version to the version file
fn write_local_version(version: &str) -> Result<(), String> {
    std::fs::write(VERSION_FILE, version).map_err(|e| format!("Failed to write version file: {}", e))
}

/// Compare two semver-like version strings (e.g., "v0.1.0" vs "v0.2.0")
/// Returns true if remote_version is newer than local_version
fn is_newer_version(local_version: &str, remote_version: &str) -> bool {
    // Strip 'v' prefix if present for comparison
    let local = local_version.trim_start_matches('v');
    let remote = remote_version.trim_start_matches('v');

    // Parse version components
    let parse_version = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect()
    };

    let local_parts = parse_version(local);
    let remote_parts = parse_version(remote);

    // Compare component by component
    for i in 0..std::cmp::max(local_parts.len(), remote_parts.len()) {
        let local_num = local_parts.get(i).copied().unwrap_or(0);
        let remote_num = remote_parts.get(i).copied().unwrap_or(0);

        if remote_num > local_num {
            return true;
        } else if remote_num < local_num {
            return false;
        }
    }

    false // Versions are equal
}

zed_extension_api::register_extension!(ClaudeCodeExtension);
