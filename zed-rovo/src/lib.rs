use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result};

struct RovoExtension;

const SERVER_NAME: &str = "rovo-lsp";

impl zed::Extension for RovoExtension {
    fn new() -> Self {
        Self
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

        // Check user-configured binary path
        if let Some(path) = binary_settings.and_then(|s| s.path) {
            return Ok(make_cmd(path));
        }

        // Check system PATH (includes ~/.cargo/bin)
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(make_cmd(path));
        }

        let message = "rovo-lsp is not installed. Run `cargo install rovo-lsp` to install it.";

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::Failed(message.to_string()),
        );

        Err(message.into())
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
