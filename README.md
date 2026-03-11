# CTK

`ctk` is a compact command wrapper intended for Codex sessions. Every recommended workflow starts with `ctk`, for example:

```bash
ctk git status
ctk git diff
ctk ls -la .
ctk read src/main.rs
ctk sed -n '1,80p' src/main.rs
ctk grep "handler" src
ctk deps
ctk test cargo test
ctk err cargo build
```

## Command Surface

- `ctk git ...`: compact git status, diff, log, show, and terse mutation confirmations
- `ctk gh ...`: compact `gh` passthrough when GitHub CLI is installed
- `ctk ls [path...]`: compact tree-style listing with `-a`, `-l`, and `-R` compatibility
- `ctk read <file>`: line-numbered file view, with `--level aggressive` for symbol-heavy views
- `ctk sed -n 'start,endp' <file>`: read-only sed compatibility for exact line-range extraction
- `ctk grep <pattern> [path]`: grouped recursive search
- `ctk find <pattern> [path]`: grouped glob-style file discovery
- `ctk deps [path]`: compact dependency summary for Cargo, Node, Python, and Go projects
- `ctk test <cmd...>`: failure-focused wrapper for noisy test commands
- `ctk err <cmd...>`: error- and warning-focused wrapper for builds and linters
- `ctk log <file>`: deduplicated log viewer
- `ctk json <file>`: JSON structure without values
- `ctk run <cmd...>`: generic fallback wrapper that still keeps the `ctk` prefix

## Install

```bash
./scripts/install-local.sh
```

That script:

- installs the Rust toolchain via `rustup` if `cargo` is missing
- runs `cargo install --path . --force`
- symlinks the local skill to `~/.agents/skills/ctk-cli`
- also creates a compatibility link at `~/.codex/skills/ctk-cli` for older local Codex harnesses
- symlinks the project execpolicy rule to `~/.codex/rules/ctk.rules`
- fails with a clear error if an expected source path is missing or a target path is an unexpected directory

## Uninstall

```bash
./scripts/uninstall-local.sh
```

That script:

- removes the cargo-installed `ctk` binary when present
- removes the installed skill links from `~/.agents/skills/ctk-cli` and `~/.codex/skills/ctk-cli`
- removes the installed rule link from `~/.codex/rules/ctk.rules`
- leaves the repository source files untouched

## Verify

```bash
ctk ls .
ctk read README.md
ctk sed -n '1,40p' README.md
ctk deps
cargo test
```
