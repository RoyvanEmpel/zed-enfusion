# Enfusion Script for Zed

> Mirrored at <https://git.royvanempel.nl/royvanempel/zed-enfusion>; issues and
> pull requests go through GitHub.

Zed extension for **Enfusion Script** (Enforce Script) as used by Arma
Reforger and the Enfusion Workbench. It is the Zed counterpart of the VS Code
extension [Reforger Script Tools](https://github.com/burn0ut7/reforger-script-tools)
and reuses its Rust language server.

| Feature | Provided by |
| --- | --- |
| Syntax highlighting, bracket matching, outline, indentation, text objects | tree-sitter grammar ([fork of `simonvic/tree-sitter-enforce`](https://github.com/RoyvanEmpel/tree-sitter-enforce), MIT) |
| Completion, snippets, signature help, hover, go to definition, document symbols | `reforger_language_server` (LSP over stdio) |
| Parser diagnostics (errors while typing) | `reforger_language_server` |
| Semantic token coloring (classes, fields, preprocessor, …) | `reforger_language_server` + `semantic_token_rules.json` |
| Range formatting | `reforger_language_server` |
| MCP server for the Zed agent panel (script / game data / official wiki search) | `reforger_language_server mcp` as a Zed context server |
| Workbench compiler validation (NET API, port 5775) | Zed tasks in `examples/reforger-project/.zed/tasks.json` |

Not portable from VS Code: the custom Enter/Tab/Space typing assists, the
Search UI (`Ctrl+Alt+F`), the managed Workbench bridge installer (Windows
registry) and the three bracket-coloring modes. Zed has no debugger for
Enfusion Script and neither has the VS Code extension: "debugging" means the
diagnostics above plus Workbench validation.

## 1. Build the language server (one time)

Upstream publishes **no prebuilt binaries and no license**, so this extension
never downloads the server. Build it locally instead (needs Rust via rustup):

```sh
./scripts/build-server.sh
```

This clones `burn0ut7/reforger-script-tools` next to this repository at a
pinned commit, runs `cargo build --release`, installs
`~/.local/bin/reforger_language_server` and copies the Official Wiki Markdown
corpus (310 pages, 4 MB) to `~/.local/share/reforger-script-tools/official-wiki`
for the MCP wiki tools. The extension finds the binary via, in
order: `lsp.reforger-language-server.binary.path` in Zed settings, the
`REFORGER_LANGUAGE_SERVER` environment variable, or `$PATH`.

## 2. Install the extension in Zed

1. Open the command palette and run `zed: install dev extension`.
2. Select this `zed-enfusion` directory.

Zed compiles `src/lib.rs` to WebAssembly and clones and builds the grammar
from the repository/rev in `extension.toml` (first build downloads the WASI SDK).
Use `zed: reload extensions`/reinstall after changing queries or `lib.rs`.

## 3. `.c` files

Enfusion Script files use `.c`, which Zed also assigns to C. A Zed extension
cannot express "only `.c` under a `Scripts/` folder", so this extension claims
every `.c` file; Zed lets the most recently registered language win an equal
suffix match, and extensions register after the built-in languages. Result:
with the extension installed, `.c` files open as Enfusion Script without any
configuration.

If a project also contains real C code, hand those files back to C with the
`file_types` setting (user settings always beat extension suffixes), globally
or in the project's `.zed/settings.json`:

```jsonc
"file_types": {
  "C": ["src/**/*.c"]
}
```

Optional: enable LSP semantic coloring on top of tree-sitter:

```jsonc
"languages": {
  "Enfusion Script": { "semantic_tokens": "combined" }
}
```

## Language server arguments

Default arguments passed by the extension:

```
--workspace-scripts <worktree root> --external-index-mode none
```

`--external-index-mode none` skips Workbench/PAC-backed base-game indexes,
which the server can only obtain through a running Workbench on Windows.
Override everything with `lsp.reforger-language-server.binary.arguments`
(see `examples/reforger-project/.zed/settings.json`). Run
`reforger_language_server --help` for all options (`--log <path>` is useful
when debugging).

## MCP context server (agent panel)

The extension registers `reforger-script-tools` as a Zed context server: the
same binary in `mcp` mode (87 tools: workspace/game-data symbol and text
search, symbol inspection, official wiki lookup). **It needs no configuration**:
Zed starts context servers in the project root, and the extension passes that
directory as `--workspace-scripts`. The command is always `/bin/sh -c` (or
`cmd /C` on Windows) by absolute path, because Zed resolves an extension's
context server command relative to the extension's work directory; the shell
then finds `reforger_language_server` on `$PATH`. Result: the agent searches the
project you have open. Optional overrides under `context_servers`:

```jsonc
"context_servers": {
  "reforger-script-tools": {
    "settings": {
      "workspace_scripts": ["/home/you/mods/OtherMod"],  // index these instead of the open project
      "external_index_mode": "none"
    }
  }
}
```

`official_wiki_root` overrides the wiki corpus location (default
`~/.local/share/reforger-script-tools/official-wiki`, passed automatically when
the directory exists). Set `command` (`path`, `arguments`) instead to bypass
these settings entirely.

## Workbench integration

Workbench exposes a local NET API (Resource Manager → Options → *Enable net
API*, default `127.0.0.1:5775`). The tasks in
`examples/reforger-project/.zed/tasks.json` call the server's `workbench-api
status|validate|read-logs` sub-commands so compiler results appear in a Zed
terminal. Workbench itself runs on Windows only.

## Layout

```
zed-enfusion/
  extension.toml                     manifest: grammar, language server, context server
  Cargo.toml, src/lib.rs             extension logic (binary discovery, args, MCP)
  languages/enfusion-script/
    config.toml                      comments, brackets, tabs
    highlights.scm brackets.scm indents.scm outline.scm textobjects.scm overrides.scm
    semantic_token_rules.json        maps reforger* LSP token types to theme styles
  scripts/build-server.sh            builds/installs the language server
  examples/reforger-project/          sample mod: .zed/settings.json, .zed/tasks.json, Scripts/
```

## Licensing

- This extension: MIT (see `LICENSE`).
- Grammar fork: MIT (Simone Vicinanza, `tree-sitter-enforce`).
- `reforger_language_server`: no license published upstream — build and use it
  locally; do not redistribute the binary until upstream adds a license.
- Unofficial project, not affiliated with Bohemia Interactive.
