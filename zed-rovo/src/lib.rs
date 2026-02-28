use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

/// Zed editor extension that provides LSP support for Rovo annotations in Rust
/// by locating and launching the `rovo-lsp` language server.
struct RovoExtension {
    cached_binary_path: Option<String>,
}

const SERVER_NAME: &str = "rovo-lsp";
const INSTALL_MESSAGE: &str =
    "rovo-lsp is not installed. Run `cargo install rovo-lsp` to install it.";

/// Resolves the cargo bin directory path in a cross-platform manner.
///
/// Checks `CARGO_HOME` first, then falls back to platform-specific home
/// directories (`HOME` on Unix, `USERPROFILE` on Windows).
fn cargo_bin_path() -> Option<String> {
    if let Ok(cargo_home) = std::env::var("CARGO_HOME") {
        return Some(format!("{cargo_home}/bin/{SERVER_NAME}"));
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;

    Some(format!("{home}/.cargo/bin/{SERVER_NAME}"))
}

impl zed::Extension for RovoExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    /// Resolves the `rovo-lsp` binary and returns the command to launch it.
    ///
    /// Resolution order:
    /// 1. User-configured binary path from Zed LSP settings
    /// 2. System PATH lookup
    /// 3. Previously cached binary path
    /// 4. Common cargo bin location (`CARGO_HOME` or `~/.cargo/bin`)
    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.binary);

        let binary_args = binary_settings
            .as_ref()
            .and_then(|s| s.arguments.clone())
            .unwrap_or_default();

        let env: Vec<(String, String)> = binary_settings
            .as_ref()
            .and_then(|s| s.env.clone())
            .map(|env| env.into_iter().collect())
            .unwrap_or_default();

        let make_cmd = |command: String| zed::Command {
            command,
            args: binary_args.clone(),
            env: env.clone(),
        };

        // 1. Check user-configured binary path
        if let Some(path) = binary_settings.and_then(|s| s.path) {
            return Ok(make_cmd(path));
        }

        // 2. Check system PATH
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(make_cmd(path));
        }

        // 3. Check cached path from previous resolution
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(make_cmd(path.clone()));
            } else {
                self.cached_binary_path = None;
            }
        }

        // 4. Try common cargo bin locations
        if let Some(cargo_bin) = cargo_bin_path() {
            if fs::metadata(&cargo_bin).is_ok_and(|stat| stat.is_file()) {
                self.cached_binary_path = Some(cargo_bin.clone());
                return Ok(make_cmd(cargo_bin));
            }
        }

        // 5. Not found
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Failed(INSTALL_MESSAGE.to_string()),
        );

        Err(INSTALL_MESSAGE.into())
    }

    /// Forwards initialization options from user LSP settings to `rovo-lsp`.
    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.initialization_options))
    }

    /// Forwards workspace configuration from user LSP settings to `rovo-lsp`.
    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.settings))
    }
}

zed::register_extension!(RovoExtension);
