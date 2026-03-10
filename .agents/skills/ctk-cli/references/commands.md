# CTK Command Matrix

## Core rule

Every recommended command starts with `ctk`.

## Common mappings

- `git status` -> `ctk git status`
- `git diff` -> `ctk git diff`
- `git diff origin/feature/mmap..HEAD -- src/file.rs` -> `ctk git diff origin/feature/mmap..HEAD -- src/file.rs`
- `git diff -W HEAD~1 -- src/file.rs` -> `ctk git diff -W HEAD~1 -- src/file.rs`
- `git log --oneline -10` -> `ctk git log --oneline -10`
- `git log -L :handle:src/main.rs` -> `ctk git log -L :handle:src/main.rs`
- `git show HEAD~1` -> `ctk git show HEAD~1`
- `git show HEAD:src/main.rs` -> `ctk git show HEAD:src/main.rs`
- `ls -la` -> `ctk ls -la .`
- `cat path/to/file` -> `ctk read path/to/file`
- `rg "pattern" src` -> `ctk grep "pattern" src`
- `find . -name "*.rs"` -> `ctk find "*.rs" .`
- `cat Cargo.toml` or `cat package.json` for dependency inspection -> `ctk deps`
- `cargo test` -> `ctk test cargo test`
- `pytest -q` -> `ctk test pytest -q`
- `npm test` -> `ctk test npm test`
- `cargo build` -> `ctk err cargo build`
- `ruff check .` -> `ctk err ruff check .`
- `cat app.log` -> `ctk log app.log`
- `cat config.json` -> `ctk json config.json`

## Subcommand guidance

- Use `ctk git ...` for git inspection and terse mutation confirmations.
- Use `ctk git ...` even for exact blob/diff/history forms when the command is representable; some precision-sensitive forms passthrough raw output unchanged.
- Use `ctk ls`, `ctk read`, `ctk grep`, and `ctk find` for repository exploration.
- Use `ctk test <cmd...>` when the raw command is mainly useful for pass/fail and failure context.
- Use `ctk err <cmd...>` when warnings and errors matter more than full progress logs.
- Use `ctk run <cmd...>` only as a fallback wrapper when no narrower `ctk` subcommand exists.

## Fallback rule

If `ctk` cannot express the workflow cleanly or the task explicitly needs raw output, use the underlying command directly and explain why.
