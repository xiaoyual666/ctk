#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

warn() {
  echo "ctk uninstall: warning: $*" >&2
}

remove_link() {
  local target="$1"

  if [ ! -e "$target" ] && [ ! -L "$target" ]; then
    warn "target not present, skipping: $target"
    return 0
  fi

  if [ -L "$target" ]; then
    if rm -f "$target"; then
      echo "removed link: $target"
    else
      warn "failed to remove link: $target"
    fi
    return 0
  fi

  warn "target exists but is not a symlink, leaving it alone: $target"
}

maybe_source_cargo_env() {
  if command -v cargo >/dev/null 2>&1; then
    return
  fi

  if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    . "$HOME/.cargo/env"
  fi
}

remove_ctk_binary() {
  maybe_source_cargo_env

  if command -v cargo >/dev/null 2>&1; then
    if cargo uninstall ctk >/dev/null 2>&1; then
      echo "removed cargo package: ctk"
      return 0
    fi
    warn "cargo uninstall ctk did not remove anything"
  fi

  if [ -f "$HOME/.cargo/bin/ctk" ]; then
    if rm -f "$HOME/.cargo/bin/ctk"; then
      echo "removed binary: $HOME/.cargo/bin/ctk"
    else
      warn "failed to remove binary: $HOME/.cargo/bin/ctk"
    fi
  else
    warn "binary not present, skipping: $HOME/.cargo/bin/ctk"
  fi
}

remove_ctk_binary
remove_link "$HOME/.agents/skills/ctk-cli"
remove_link "$HOME/.codex/skills/ctk-cli"
remove_link "$HOME/.codex/rules/ctk.rules"

echo "ctk uninstall: repository sources remain in $ROOT"
