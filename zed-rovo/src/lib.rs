use std::fs;
use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct RovoExtension {
    cached_binary_path: Option<String>,
}

const SERVER_NAME: &str = "rovo-lsp";

impl zed::Extension for RovoExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

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

        let env = binary_settings
            .as_ref()
            .and_then(|s| s.env.clone())
            .map(|env| env.into_iter().collect())
            .unwrap_or_default();

        // 1. Check user-configured binary path
        if let Some(path) = binary_settings.and_then(|s| s.path) {
            return Ok(zed::Command {
                command: path,
                args: binary_args,
                env,
            });
        }

        // 2. Check system PATH
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(zed::Command {
                command: path,
                args: binary_args,
                env,
            });
        }

        // 3. Check cached path from previous resolution
        if let Some(path) = &self.cached_binary_path {
            if fs::metadata(path).is_ok_and(|stat| stat.is_file()) {
                return Ok(zed::Command {
                    command: path.clone(),
                    args: binary_args,
                    env,
                });
            }
        }

        // 4. Try common cargo bin locations
        let home = std::env::var("HOME").unwrap_or_default();
        let cargo_bin = format!("{home}/.cargo/bin/{SERVER_NAME}");

        if fs::metadata(&cargo_bin).is_ok_and(|stat| stat.is_file()) {
            self.cached_binary_path = Some(cargo_bin.clone());
            return Ok(zed::Command {
                command: cargo_bin,
                args: binary_args,
                env,
            });
        }

        // 5. Not found — report installation error
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Failed(
                "rovo-lsp not found. Install it with: cargo install rovo-lsp".to_string(),
            ),
        );

        Err("rovo-lsp is not installed. Run `cargo install rovo-lsp` to install it.".into())
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        Ok(LspSettings::for_worktree(SERVER_NAME, worktree)
            .ok()
            .and_then(|s| s.initialization_options))
    }

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
