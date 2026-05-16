use zed_extension_api as zed;

use zed::settings::LspSettings;
use zed::{Command, Extension, LanguageServerId, Result, Worktree};

struct OxContentZedExtension;

impl Extension for OxContentZedExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        let binary = settings.binary;

        let command = binary
            .as_ref()
            .and_then(|binary| binary.path.clone())
            .unwrap_or_else(|| "ox-content-lsp".to_string());
        let args = binary.as_ref().and_then(|binary| binary.arguments.clone()).unwrap_or_default();
        let env = binary.and_then(|binary| binary.env).unwrap_or_default().into_iter().collect();

        Ok(Command { command, args, env })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.initialization_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<serde_json::Value>> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        Ok(settings.settings)
    }
}

zed::register_extension!(OxContentZedExtension);
