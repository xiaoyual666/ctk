---
name: ctk-cli
description: Use when Codex should prefer the local `ctk` executable over raw shell commands for repository exploration, git inspection, search, JSON inspection, logs, and noisy test or build output. Trigger when working in repositories that ship `ctk` and the task would normally reach for `git`, `ls`, `cat`, `rg`, `find`, test runners, or build tools, but a compact `ctk ...` command would preserve more signal with fewer tokens.
---

# Ctk Cli

## Overview

Use `ctk` as the default command prefix for routine development workflows in repositories that provide it. Prefer the narrowest `ctk` subcommand that fits the task so Codex sees compact, high-signal output instead of raw shell noise.

## Workflow

1. Confirm `ctk` is available with `command -v ctk`.
2. Translate the intended raw command into the nearest `ctk` form.
3. Use `ctk` first for exploration, line-range reads, git inspection, dependency inspection, tests, builds, logs, and JSON structure.
4. Fall back to raw shell only when `ctk` does not cover the workflow cleanly.
5. If you need the exact command mapping or examples, read [references/commands.md](references/commands.md).

## Preferred Commands

- Use `ctk git status`, `ctk git diff`, `ctk git log`, and `ctk git show` before raw `git`.
- Use `ctk ls`, `ctk read`, `ctk sed`, `ctk grep`, `ctk find`, and `ctk deps` for codebase discovery.
- Use `ctk test <cmd...>` for test runners.
- Use `ctk err <cmd...>` for build, lint, and compile commands where errors matter more than full logs.
- Use `ctk log <file>` for log files and `ctk json <file>` for JSON structure.
- Use `ctk run <cmd...>` only as the generic fallback when no narrower `ctk` command exists.

## Decision Rules

- Prefer specialized `ctk` subcommands over `ctk run`.
- Prefer `ctk test` for test commands even when the underlying tool is language-specific.
- Prefer `ctk err` for builds and linters when the main goal is to surface failures or warnings.
- For exact git blob/diff/history forms, still prefer `ctk git ...`; `ctk` passthroughs several precision-sensitive git forms unchanged.
- Fall back to raw shell only when `ctk` truly cannot express the workflow cleanly.

## Extension

- When extending `ctk`, update the Rust command implementation first, then update [references/commands.md](references/commands.md) so Codex keeps the command mapping accurate.
