#!/usr/bin/env bash
# Build the Reforger Script Tools language server from source and install it
# to ~/.local/bin so the Zed extension can find it on $PATH.
#
# Upstream ships no license and no prebuilt binaries, so it is never
# downloaded by the extension itself. This script pins a known-good commit.
set -euo pipefail

UPSTREAM_REPO="https://github.com/burn0ut7/reforger-script-tools.git"
UPSTREAM_REV="${REFORGER_SCRIPT_TOOLS_REV:-59c4e25dc034e7b83254fbd83981f0f7794e397b}"
SRC_DIR="${REFORGER_SCRIPT_TOOLS_DIR:-$(cd "$(dirname "$0")/../.." && pwd)/reforger-script-tools}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY="reforger_language_server"
# Official Wiki Markdown corpus used by the MCP wiki tools; the extension passes
# this directory as --official-wiki-root when it exists.
WIKI_DIR="${WIKI_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/reforger-script-tools/official-wiki}"

if ! command -v cargo >/dev/null 2>&1; then
  if [ -x "$HOME/.cargo/bin/cargo" ]; then
    export PATH="$HOME/.cargo/bin:$PATH"
  else
    echo "cargo not found. Install Rust via https://rustup.rs first." >&2
    exit 1
  fi
fi

if [ ! -d "$SRC_DIR/.git" ]; then
  echo "Cloning $UPSTREAM_REPO into $SRC_DIR"
  git clone "$UPSTREAM_REPO" "$SRC_DIR"
fi

echo "Checking out $UPSTREAM_REV"
git -C "$SRC_DIR" fetch --quiet origin
git -C "$SRC_DIR" checkout --quiet "$UPSTREAM_REV"

echo "Building $BINARY (release)"
cargo build --release --manifest-path "$SRC_DIR/server/Cargo.toml" --bin "$BINARY"

mkdir -p "$INSTALL_DIR"
install -m 0755 "$SRC_DIR/server/target/release/$BINARY" "$INSTALL_DIR/$BINARY"
echo "Installed $INSTALL_DIR/$BINARY"
"$INSTALL_DIR/$BINARY" --help | head -1

echo "Installing Official Wiki corpus to $WIKI_DIR"
rm -rf "$WIKI_DIR"
mkdir -p "$(dirname "$WIKI_DIR")"
cp -r "$SRC_DIR/data/official-wiki" "$WIKI_DIR"
echo "Installed $(find "$WIKI_DIR" -name "*.md" | wc -l) wiki pages"
