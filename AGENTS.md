# CTK Workflow

- Use `ctk` as the default command prefix for routine repository work instead of raw shell commands.
- Do not use bare `git` when `ctk git ...` can express the same workflow.
- Use `ctk git status`, `ctk git diff`, `ctk git log`, and `ctk git show` before raw `git`.
- Use `ctk git ...` even for exact blob, diff, range, pathspec, and targeted history forms that `ctk` passthroughs unchanged.
- Use `ctk ls`, `ctk read`, `ctk sed`, `ctk grep`, `ctk find`, and `ctk deps` for codebase exploration.
- Use `ctk test <cmd...>` for test runners and `ctk err <cmd...>` for builds and linters.
- Use `ctk log <file>` for logs and `ctk json <file>` for JSON inspection.
- Fall back to raw commands only when `ctk` truly does not cover the workflow cleanly.
- Use the `$ctk-cli` skill when choosing among `ctk` commands or when extending the command matrix.
