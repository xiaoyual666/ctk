use crate::runner::run_command;
use crate::text::{tail_lines, unique_preserve_order};
use regex::Regex;
use std::sync::OnceLock;

const FAILURE_PATTERNS: &[&str] = &[
    "failed",
    "error",
    "failures",
    "traceback",
    "assertionerror",
    "panic",
    "test result: failed",
    "not ok",
    "failure",
];
const SUCCESS_PATTERNS: &[&str] = &["test result: ok", "success", "0 failed", "failed=0"];

pub fn handle(cmd: Vec<String>) -> i32 {
    if cmd.is_empty() {
        eprintln!("ctk test: missing command");
        return 2;
    }

    let result = run_command(&cmd);
    let combined = format!("{}\n{}", result.stdout, result.stderr);
    let lines = combined
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();

    if result.returncode == 0 {
        let summary = lines
            .iter()
            .filter(|line| matches_success(line))
            .cloned()
            .collect::<Vec<_>>();
        let summary = if summary.is_empty() {
            Vec::new()
        } else {
            summary
                .into_iter()
                .rev()
                .take(5)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect()
        };
        if summary.is_empty() {
            println!("ok");
        } else {
            println!("{}", summary.join("\n"));
        }
        return 0;
    }

    let extracted = extract_failure_windows(&lines);
    if extracted.is_empty() {
        let fallback = tail_lines(
            if result.stderr.is_empty() {
                &result.stdout
            } else {
                &result.stderr
            },
            40,
        );
        eprintln!("{}", fallback.join("\n"));
    } else {
        eprintln!(
            "{}",
            extracted
                .into_iter()
                .take(120)
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    result.returncode
}

fn extract_failure_windows(lines: &[String]) -> Vec<String> {
    let mut kept = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !matches_any(line, FAILURE_PATTERNS) {
            continue;
        }
        let start = index.saturating_sub(2);
        let end = (index + 3).min(lines.len());
        for candidate in &lines[start..end] {
            if !candidate.trim().is_empty() {
                kept.push(candidate.clone());
            }
        }
    }
    unique_preserve_order(kept)
}

fn matches_any(line: &str, patterns: &[&str]) -> bool {
    let lowered = line.to_lowercase();
    patterns.iter().any(|pattern| lowered.contains(pattern))
}

fn matches_success(line: &str) -> bool {
    matches_any(line, SUCCESS_PATTERNS)
        || passed_regex().is_match(line)
        || tap_ok_regex().is_match(line)
}

fn passed_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?i)(^|\b)\d+\s+passed\b").expect("valid regex"))
}

fn tap_ok_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"(?i)^\s*ok(\b|$)").expect("valid regex"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_words_are_not_false_successes() {
        assert!(!matches_success("smoke-finished"));
        assert!(!matches_success("broker-ready"));
    }
}
