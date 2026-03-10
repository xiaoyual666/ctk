use crate::runner::run_command;
use crate::text::{tail_lines, unique_preserve_order};

const ERROR_PATTERNS: &[&str] = &[
    "error",
    "warning",
    "failed",
    "failure",
    "exception",
    "traceback",
    "panic",
    "undefined",
    "not found",
    "cannot",
];

pub fn handle(cmd: Vec<String>) -> i32 {
    if cmd.is_empty() {
        eprintln!("ctk err: missing command");
        return 2;
    }

    let result = run_command(&cmd);
    let stderr_lines = result
        .stderr
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();
    let stdout_lines = result
        .stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| matches_error(line))
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();
    let mut merged = stderr_lines;
    merged.extend(stdout_lines);
    let merged = unique_preserve_order(merged);

    if !merged.is_empty() {
        println!(
            "{}",
            merged.into_iter().take(80).collect::<Vec<_>>().join("\n")
        );
    } else if result.returncode == 0 {
        let summary = tail_lines(
            if result.stdout.is_empty() {
                &result.stderr
            } else {
                &result.stdout
            },
            5,
        );
        if summary.is_empty() {
            println!("ok");
        } else {
            println!("{}", summary.join("\n"));
        }
    } else {
        let fallback = tail_lines(
            if result.stderr.is_empty() {
                &result.stdout
            } else {
                &result.stderr
            },
            20,
        );
        eprintln!(
            "{}",
            if fallback.is_empty() {
                "command failed".to_string()
            } else {
                fallback.join("\n")
            }
        );
    }

    result.returncode
}

fn matches_error(line: &str) -> bool {
    let lowered = line.to_lowercase();
    ERROR_PATTERNS
        .iter()
        .any(|pattern| lowered.contains(pattern))
}
