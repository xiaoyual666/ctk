#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

warn() {
  echo "ctk install: warning: $*" >&2
}

link_path() {
  local source="$1"
  local target="$2"

  if [ ! -e "$source" ] && [ ! -L "$source" ]; then
    warn "source not found, skipping link: $source"
    return 0
  fi

  if ! mkdir -p "$(dirname "$target")"; then
    warn "failed to create parent directory for $target, skipping link"
    return 0
  fi

  if [ -d "$target" ] && [ ! -L "$target" ]; then
    warn "target exists as a directory, refusing to replace it: $target"
    return 0
  fi

  if ! rm -f "$target"; then
    warn "failed to remove existing target, skipping link: $target"
    return 0
  fi
  if ln -s "$source" "$target"; then
    echo "linked: $target -> $source"
  else
    warn "failed to create link: $target -> $source"
  fi
}

ensure_rust_toolchain() {
  if command -v cargo >/dev/null 2>&1; then
    return
  fi

  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
  fi

  if command -v cargo >/dev/null 2>&1; then
    return
  fi

  if ! command -v curl >/dev/null 2>&1; then
    echo "ctk install: cargo is missing and curl is not available to bootstrap Rust via rustup" >&2
    exit 1
  fi

  echo "ctk install: cargo not found, installing Rust toolchain via rustup"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable

  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
  fi

  if ! command -v cargo >/dev/null 2>&1; then
    echo "ctk install: rustup completed but cargo is still unavailable in this shell" >&2
    exit 1
  fi
}

ensure_rust_toolchain
cargo install --path "$ROOT" --force
echo "installed: ctk via cargo"

link_path "$ROOT/.agents/skills/ctk-cli" "$HOME/.agents/skills/ctk-cli"

# Compatibility for local Codex harnesses that still scan ~/.codex/skills.
link_path "$ROOT/.agents/skills/ctk-cli" "$HOME/.codex/skills/ctk-cli"

# Install the project-local Codex execpolicy rule that nudges bare git usage to ctk git.
link_path "$ROOT/.codex/rules/ctk.rules" "$HOME/.codex/rules/ctk.rules"
