//! Zed extension for Enfusion Script (Arma Reforger / Enforce Script).
//!
//! Syntax highlighting comes from the bundled tree-sitter grammar; language
//! intelligence comes from the `reforger_language_server` binary of the
//! Reforger Script Tools project. That project ships no license and no
//! prebuilt releases, so this extension never downloads it: the user builds
//! it once (see `scripts/build-server.sh`) and the extension locates it via
//! Zed's `lsp.reforger-language-server.binary.path` setting, the
//! `REFORGER_LANGUAGE_SERVER` environment variable, or `$PATH`.

use zed_extension_api::{
    self as zed,
    serde_json::Value,
    settings::{ContextServerSettings, LspSettings},
    ContextServerConfiguration, ContextServerId, LanguageServerId, Project, Result, Worktree,
};

/// Binary name of the upstream language server (also used for MCP mode).
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

    /// Expose the same binary in its `mcp` mode as a Zed context server, so
    /// the agent panel can search scripts, game data and the official wiki.
    ///
    /// Zed resolves an extension's context server command relative to the
    /// extension's work directory, so a bare binary name would never be found
    /// on `$PATH`. The command is therefore always the platform shell by
    /// absolute path; the shell resolves `reforger_language_server` itself and,
    /// when no explicit roots are configured, passes Zed's working directory
    /// (the project root) as the script root.
    fn context_server_command(
        &mut self,
        context_server_id: &ContextServerId,
        project: &Project,
    ) -> Result<zed::Command> {
        let (command_settings, server_settings) =
            match ContextServerSettings::for_project(context_server_id.as_ref(), project) {
                Ok(settings) => (settings.command, settings.settings),
                Err(_) => (None, None),
            };

        let binary = command_settings
            .as_ref()
            .and_then(|settings| settings.path.clone())
            .unwrap_or_else(|| SERVER_BINARY.to_string());
        let (has_roots, args) = match command_settings.and_then(|settings| settings.arguments) {
            Some(args) => (true, args),
            None => Self::mcp_args(server_settings.as_ref()),
        };

        Ok(Self::shell_command(binary, args, !has_roots))
    }

    fn context_server_configuration(
        &mut self,
        _context_server_id: &ContextServerId,
        _project: &Project,
    ) -> Result<Option<ContextServerConfiguration>> {
        Ok(Some(ContextServerConfiguration {
            installation_instructions: MCP_INSTALLATION_INSTRUCTIONS.to_string(),
            default_settings: MCP_DEFAULT_SETTINGS.to_string(),
            settings_schema: MCP_SETTINGS_SCHEMA.to_string(),
        }))
    }
}

impl EnfusionScriptExtension {
    /// Build `reforger_language_server mcp ...` arguments from the context
    /// server settings object (see `MCP_SETTINGS_SCHEMA`). Returns whether
    /// explicit `workspace_scripts` roots were configured.
    fn mcp_args(settings: Option<&Value>) -> (bool, Vec<String>) {
        let mut args = vec!["mcp".to_string()];

        let mode = settings
            .and_then(|settings| settings.get("external_index_mode"))
            .and_then(Value::as_str)
            .unwrap_or("none");
        args.push("--external-index-mode".to_string());
        args.push(mode.to_string());

        let mut has_roots = false;
        if let Some(roots) = settings
            .and_then(|settings| settings.get("workspace_scripts"))
            .and_then(Value::as_array)
        {
            for root in roots.iter().filter_map(Value::as_str) {
                has_roots = true;
                args.push("--workspace-scripts".to_string());
                args.push(root.to_string());
            }
        }

        if let Some(root) = settings
            .and_then(|settings| settings.get("official_wiki_root"))
            .and_then(Value::as_str)
        {
            args.push("--official-wiki-root".to_string());
            args.push(root.to_string());
        }

        (has_roots, args)
    }

    /// Run `binary args...` through the platform shell (by absolute path).
    /// Optionally appends `--workspace-scripts <cwd>`, and passes the locally
    /// installed Official Wiki corpus (see `scripts/build-server.sh`) as
    /// `--official-wiki-root` when that directory exists.
    fn shell_command(binary: String, args: Vec<String>, add_cwd_root: bool) -> zed::Command {
        let (platform, _) = zed::current_platform();
        match platform {
            zed::Os::Windows => {
                let mut line = vec![quote_cmd(&binary)];
                line.extend(args.iter().map(|arg| quote_cmd(arg)));
                if add_cwd_root {
                    line.push("--workspace-scripts".to_string());
                    line.push("\"%CD%\"".to_string());
                }
                let base = line.join(" ");
                let wiki = "%LOCALAPPDATA%\\reforger-script-tools\\official-wiki";
                let script = format!(
                    "if exist \"{wiki}\\\" ({base} --official-wiki-root \"{wiki}\") else ({base})"
                );
                zed::Command {
                    command: "C:\\Windows\\System32\\cmd.exe".to_string(),
                    args: vec!["/C".to_string(), script],
                    env: vec![],
                }
            }
            zed::Os::Mac | zed::Os::Linux => {
                let mut line = vec!["exec".to_string(), quote_sh(&binary)];
                line.extend(args.iter().map(|arg| quote_sh(arg)));
                if add_cwd_root {
                    line.push("--workspace-scripts".to_string());
                    line.push("\"$PWD\"".to_string());
                }
                // `set --` turns the optional wiki flag into "$@" so the exec
                // line stays a single statement.
                let script = format!(
                    "W=\"${{XDG_DATA_HOME:-$HOME/.local/share}}/reforger-script-tools/official-wiki\"; \
                     if [ -d \"$W\" ]; then set -- --official-wiki-root \"$W\"; fi; {} \"$@\"",
                    line.join(" ")
                );
                zed::Command {
                    command: "/bin/sh".to_string(),
                    args: vec!["-c".to_string(), script],
                    env: vec![],
                }
            }
        }
    }
}

/// Single-quote a value for POSIX `sh`.
fn quote_sh(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

/// Double-quote a value for `cmd.exe`.
fn quote_cmd(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

const MCP_INSTALLATION_INSTRUCTIONS: &str = r#"The **Reforger Script Tools** MCP server is the same `reforger_language_server` binary that powers Enfusion Script language support, started in `mcp` mode. It gives the agent panel workspace and game-data symbol/text search, symbol inspection and official-wiki lookup.

**No configuration is required.** By default the server indexes the project you have open in Zed. The settings below are optional:

- `workspace_scripts`: index these script roots instead of the open project (absolute paths).
- `external_index_mode`: keep `"none"` unless you have Workbench-generated add-on indexes (Windows only).
- `official_wiki_root`: only if you keep the wiki corpus somewhere other than `~/.local/share/reforger-script-tools/official-wiki` (where `scripts/build-server.sh` installs it).

The binary must be on your `$PATH` (build it once with `scripts/build-server.sh` from the zed-enfusion repository; it installs to `~/.local/bin`). To use a specific binary or raw arguments, set `context_servers.reforger-script-tools.command` (`path`, `arguments`) instead."#;

const MCP_DEFAULT_SETTINGS: &str = r#"{
  // Optional: index these roots instead of the project open in Zed.
  // "workspace_scripts": ["/home/you/mods/MyMod"],
  // "none" (default), "all" or "loaded" — cached add-on indexes need Workbench (Windows).
  "external_index_mode": "none"
}"#;

const MCP_SETTINGS_SCHEMA: &str = r#"{
  "type": "object",
  "properties": {
    "workspace_scripts": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Optional: absolute paths of Enfusion Script roots to index instead of the project open in Zed."
    },
    "external_index_mode": {
      "type": "string",
      "enum": ["none", "loaded", "all"],
      "default": "none",
      "description": "Which cached external add-on indexes to load."
    },
    "official_wiki_root": {
      "type": "string",
      "description": "Optional: directory with the Official Wiki Markdown corpus. Defaults to ~/.local/share/reforger-script-tools/official-wiki (installed by scripts/build-server.sh)."
    }
  }
}"#;

zed::register_extension!(EnfusionScriptExtension);
