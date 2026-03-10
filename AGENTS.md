# CTK Workflow

- Prefer `ctk` for routine repository work instead of raw shell commands.
- Use `ctk git status`, `ctk git diff`, `ctk git log`, and `ctk git show` before raw `git`.
- Use `ctk ls`, `ctk read`, `ctk grep`, and `ctk find` for codebase exploration.
- Use `ctk test <cmd...>` for test runners and `ctk err <cmd...>` for builds and linters.
- Use `ctk log <file>` for logs and `ctk json <file>` for JSON inspection.
- Fall back to raw commands only when `ctk` does not cover the workflow cleanly.
- Use the `$ctk-cli` skill when choosing among `ctk` commands or when extending the command matrix.
