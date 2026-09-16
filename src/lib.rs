//! Zed extension for Enfusion Script (Arma Reforger / Enforce Script).
//!
//! Syntax highlighting comes from the bundled tree-sitter grammar; language
//! intelligence comes from the `reforger_language_server` binary of the
//! Reforger Script Tools project. That project ships no license and no
//! prebuilt releases, so this extension never downloads it: the user builds
//! it once (see `scripts/build-server.sh`) and the extension locates it via
//! Zed's `lsp.reforger-language-server.binary.path` setting, the
//! `REFORGER_LANGUAGE_SERVER` environment variable, or `$PATH`.

use zed_extension_api::{self as zed, settings::LspSettings, LanguageServerId, Result, Worktree};

/// Binary name of the upstream language server.
const SERVER_BINARY: &str = "reforger_language_server";
/// Environment variable that may point at a specific server binary.
const SERVER_ENV_VAR: &str = "REFORGER_LANGUAGE_SERVER";

struct EnfusionScriptExtension;

impl EnfusionScriptExtension {
    /// Locate the language server binary without downloading anything.
    fn find_server_binary(&self, worktree: &Worktree) -> Result<String> {
        if let Some((_, path)) = worktree
            .shell_env()
            .into_iter()
            .find(|(key, value)| key == SERVER_ENV_VAR && !value.is_empty())
        {
            return Ok(path);
        }

        if let Some(path) = worktree.which(SERVER_BINARY) {
            return Ok(path);
        }

        Err(format!(
            "`{SERVER_BINARY}` was not found. Build it with `scripts/build-server.sh` from the \
             zed-enfusion repository (installs to ~/.local/bin), or set \
             `lsp.reforger-language-server.binary.path` in your Zed settings, or export \
             `{SERVER_ENV_VAR}=/path/to/{SERVER_BINARY}`."
        ))
    }

    /// Default LSP arguments: index the opened worktree as the script root and
    /// skip Workbench-dependent external indexes (Workbench is Windows-only).
    fn default_lsp_args(worktree: &Worktree) -> Vec<String> {
        vec![
            "--workspace-scripts".to_string(),
            worktree.root_path(),
            "--external-index-mode".to_string(),
            "none".to_string(),
        ]
    }
}

impl zed::Extension for EnfusionScriptExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree).ok();
        let binary = settings.and_then(|settings| settings.binary);

        let command = match binary.as_ref().and_then(|binary| binary.path.clone()) {
            Some(path) => path,
            None => self.find_server_binary(worktree)?,
        };
        let args = binary
            .and_then(|binary| binary.arguments)
            .unwrap_or_else(|| Self::default_lsp_args(worktree));

        Ok(zed::Command {
            command,
            args,
            env: vec![],
        })
    }

    fn language_server_initialization_options(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map(|settings| settings.initialization_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        LspSettings::for_worktree(language_server_id.as_ref(), worktree)
            .map(|settings| settings.settings)
    }
}

zed::register_extension!(EnfusionScriptExtension);
