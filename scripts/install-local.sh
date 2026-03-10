#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

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

mkdir -p "$HOME/.agents/skills"
rm -f "$HOME/.agents/skills/ctk-cli"
ln -sfn "$ROOT/.agents/skills/ctk-cli" "$HOME/.agents/skills/ctk-cli"
echo "linked: $HOME/.agents/skills/ctk-cli -> $ROOT/.agents/skills/ctk-cli"

# Compatibility for local Codex harnesses that still scan ~/.codex/skills.
mkdir -p "$HOME/.codex/skills"
rm -f "$HOME/.codex/skills/ctk-cli"
ln -sfn "$ROOT/.agents/skills/ctk-cli" "$HOME/.codex/skills/ctk-cli"
echo "linked: $HOME/.codex/skills/ctk-cli -> $ROOT/.agents/skills/ctk-cli"
