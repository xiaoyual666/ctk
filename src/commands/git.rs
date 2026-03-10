use crate::runner::{run_command, run_command_passthrough};
use crate::text::compact_block;
use regex::Regex;
use std::collections::BTreeMap;

pub fn handle(args: Vec<String>) -> i32 {
    if args.is_empty() {
        eprintln!("ctk git: missing arguments");
        return 2;
    }

    let subcommand = args[0].clone();
    let invocation = build_invocation(&args);
    if invocation.passthrough {
        return run_command_passthrough(&invocation.cmd);
    }
    let result = run_command(&invocation.cmd);

    match subcommand.as_str() {
        "status" if invocation.parse_status => handle_status(result),
        "status" if invocation.filter_status_human => handle_status_with_args(result),
        "status" => handle_generic(result),
        "diff" | "show" => handle_diff_like(result),
        "log" => handle_log(result),
        "add" | "commit" | "push" | "pull" | "fetch" | "checkout" => {
            handle_mutation(&subcommand, result)
        }
        _ => handle_generic(result),
    }
}

struct GitInvocation {
    cmd: Vec<String>,
    parse_status: bool,
    filter_status_human: bool,
    passthrough: bool,
}

fn build_invocation(args: &[String]) -> GitInvocation {
    let subcommand = args.first().map(String::as_str).unwrap_or_default();
    let rest = &args[1..];

    if subcommand == "commit" && should_passthrough_commit(rest) {
        let mut cmd = vec!["git".to_string()];
        cmd.extend(args.iter().cloned());
        return GitInvocation {
            cmd,
            parse_status: false,
            filter_status_human: false,
            passthrough: true,
        };
    }

    if subcommand == "status" {
        if rest.is_empty() {
            return GitInvocation {
                cmd: vec![
                    "git".to_string(),
                    "status".to_string(),
                    "--short".to_string(),
                    "--branch".to_string(),
                ],
                parse_status: true,
                filter_status_human: false,
                passthrough: false,
            };
        }

        if status_uses_v2(rest) {
            let mut cmd = vec!["git".to_string()];
            cmd.extend(args.iter().cloned());
            return GitInvocation {
                cmd,
                parse_status: false,
                filter_status_human: false,
                passthrough: false,
            };
        }

        if status_has_machine_flags(rest) {
            let mut cmd = vec!["git".to_string(), "status".to_string()];
            cmd.extend(rest.iter().cloned());
            return GitInvocation {
                cmd,
                parse_status: true,
                filter_status_human: false,
                passthrough: false,
            };
        }

        let mut cmd = vec!["git".to_string(), "status".to_string()];
        cmd.extend(rest.iter().cloned());
        return GitInvocation {
            cmd,
            parse_status: false,
            filter_status_human: true,
            passthrough: false,
        };
    }

    if matches!(subcommand, "diff" | "show" | "log") && git_requires_passthrough(subcommand, rest) {
        let mut cmd = vec!["git".to_string()];
        cmd.extend(args.iter().cloned());
        return GitInvocation {
            cmd,
            parse_status: false,
            filter_status_human: false,
            passthrough: true,
        };
    }

    let mut cmd = vec!["git".to_string()];
    cmd.extend(args.iter().cloned());
    GitInvocation {
        cmd,
        parse_status: false,
        filter_status_human: false,
        passthrough: false,
    }
}

fn should_passthrough_commit(rest: &[String]) -> bool {
    !rest.iter().any(|arg| {
        arg == "-m" || arg.starts_with("-m") || arg == "--message" || arg.starts_with("--message=")
    })
}

fn status_uses_v2(rest: &[String]) -> bool {
    rest.iter().any(|arg| arg == "--porcelain=v2")
}

fn status_has_machine_flags(rest: &[String]) -> bool {
    rest.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "-s" | "-b" | "-sb" | "--short" | "--porcelain" | "--porcelain=v1" | "--branch"
        )
    })
}

fn git_requires_passthrough(subcommand: &str, rest: &[String]) -> bool {
    let raw_flags = [
        "--stat",
        "--numstat",
        "--shortstat",
        "--name-only",
        "--name-status",
        "--raw",
        "--check",
        "--summary",
        "--patch-with-stat",
        "--word-diff",
    ];
    let format_flags = rest.iter().any(|arg| {
        arg.starts_with("--pretty")
            || arg.starts_with("--format")
            || arg == "--oneline"
            || arg == "--graph"
    });
    let has_raw_flag = rest.iter().any(|arg| raw_flags.contains(&arg.as_str()));
    let has_function_context = rest
        .iter()
        .any(|arg| arg == "-W" || arg == "--function-context");
    let has_unified_context = rest
        .iter()
        .any(|arg| arg.starts_with("-U") || arg == "--unified" || arg.starts_with("--unified="));
    let has_exact_diff_target = subcommand == "diff"
        && rest
            .iter()
            .any(|arg| arg == "--" || arg.contains("..") || arg.contains("..."));
    let has_targeted_history = subcommand == "log"
        && rest.iter().any(|arg| {
            arg == "-L"
                || arg.starts_with("-L")
                || arg == "--follow"
                || arg == "-p"
                || arg == "--patch"
        });
    let is_blob_show = subcommand == "show"
        && rest
            .iter()
            .any(|arg| !arg.starts_with('-') && arg.contains(':'));

    format_flags
        || has_raw_flag
        || has_function_context
        || has_unified_context
        || has_exact_diff_target
        || has_targeted_history
        || is_blob_show
}

fn handle_status(result: crate::runner::CommandResult) -> i32 {
    if result.returncode != 0 {
        eprintln!(
            "{}",
            if result.stderr.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            }
        );
        return result.returncode;
    }

    let mut branch = String::new();
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut items = Vec::new();
    for line in result.stdout.lines().filter(|line| !line.trim().is_empty()) {
        if let Some(rest) = line.strip_prefix("##") {
            branch = rest.trim().to_string();
            continue;
        }
        for label in classify_status_codes(line) {
            *counts.entry(label).or_insert(0) += 1;
        }
        items.push(line.trim_end().to_string());
    }

    if !branch.is_empty() {
        println!("branch: {branch}");
    }
    if counts.is_empty() {
        println!("clean");
        return 0;
    }
    let summary = counts
        .into_iter()
        .map(|(label, count)| format!("{label} {count}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("changes: {summary}");
    for item in items.iter().take(20) {
        println!("{item}");
    }
    if items.len() > 20 {
        println!("... {} more items omitted ...", items.len() - 20);
    }
    0
}

fn classify_status_codes(line: &str) -> Vec<&'static str> {
    let mut labels = Vec::new();
    let mut chars = line.chars();
    let x = chars.next().unwrap_or(' ');
    let y = chars.next().unwrap_or(' ');
    for mark in [x, y] {
        if let Some(label) = classify_status_mark(mark) {
            if !labels.contains(&label) {
                labels.push(label);
            }
        }
    }
    if labels.is_empty() {
        labels.push("changed");
    }
    labels
}

fn classify_status_mark(mark: char) -> Option<&'static str> {
    match mark {
        'M' => Some("modified"),
        'A' => Some("added"),
        'D' => Some("deleted"),
        'R' => Some("renamed"),
        'C' => Some("copied"),
        'U' => Some("unmerged"),
        '?' => Some("untracked"),
        '!' => Some("ignored"),
        ' ' => None,
        _ => Some("changed"),
    }
}

fn handle_diff_like(result: crate::runner::CommandResult) -> i32 {
    if result.returncode != 0 {
        eprintln!(
            "{}",
            if result.stderr.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            }
        );
        return result.returncode;
    }

    if result.stdout.trim().is_empty() {
        println!("no diff");
        return 0;
    }

    let mut files = Vec::new();
    let mut hunks = Vec::new();
    let mut additions = 0usize;
    let mut deletions = 0usize;
    for line in result.stdout.lines() {
        if let Some(rest) = line.strip_prefix("diff --git ") {
            let parts = rest.split_whitespace().collect::<Vec<_>>();
            if parts.len() >= 2 {
                files.push(parts[1].trim_start_matches("b/").to_string());
            }
        } else if line.starts_with("@@") {
            hunks.push(line.trim_end().to_string());
        } else if line.starts_with('+') && !line.starts_with("+++") {
            additions += 1;
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
        }
    }

    if !files.is_empty() {
        println!("files: {} changed", files.len());
        for file in files.iter().take(15) {
            println!("  {file}");
        }
        if files.len() > 15 {
            println!("  ... {} more files omitted ...", files.len() - 15);
        }
    }
    println!("delta: +{additions} -{deletions}");
    for hunk in hunks.iter().take(20) {
        println!("{hunk}");
    }
    if hunks.len() > 20 {
        println!("... {} more hunks omitted ...", hunks.len() - 20);
    }
    0
}

fn handle_log(result: crate::runner::CommandResult) -> i32 {
    if result.returncode != 0 {
        eprintln!(
            "{}",
            if result.stderr.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            }
        );
        return result.returncode;
    }
    let lines = result
        .stdout
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        println!("no commits");
        return 0;
    }
    for line in lines.iter().take(20) {
        println!("{line}");
    }
    if lines.len() > 20 {
        println!("... {} more commits omitted ...", lines.len() - 20);
    }
    0
}

fn handle_mutation(subcommand: &str, result: crate::runner::CommandResult) -> i32 {
    if result.returncode != 0 {
        eprintln!(
            "{}",
            if result.stderr.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            }
        );
        return result.returncode;
    }

    let text = format!("{}\n{}", result.stdout, result.stderr);
    if subcommand == "commit" {
        let regex = Regex::new(r"\[.*? ([0-9a-f]{7,})\]").expect("valid regex");
        if let Some(captures) = regex.captures(&text) {
            println!("ok commit {}", &captures[1]);
            return 0;
        }
    }
    if subcommand == "push" {
        println!("ok push");
        return 0;
    }
    if subcommand == "pull" {
        let last = text
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("ok pull");
        println!("{last}");
        return 0;
    }
    println!("ok {subcommand}");
    0
}

fn handle_generic(result: crate::runner::CommandResult) -> i32 {
    if result.returncode != 0 {
        eprintln!(
            "{}",
            if result.stderr.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            }
        );
        return result.returncode;
    }
    let stream = if result.stdout.trim().is_empty() {
        &result.stderr
    } else {
        &result.stdout
    };
    if stream.trim().is_empty() {
        println!("ok");
    } else {
        println!("{}", compact_block(stream, 80));
    }
    0
}

fn handle_status_with_args(result: crate::runner::CommandResult) -> i32 {
    if result.returncode != 0 {
        eprintln!(
            "{}",
            if result.stderr.trim().is_empty() {
                result.stdout.trim()
            } else {
                result.stderr.trim()
            }
        );
        return result.returncode;
    }

    let filtered = result
        .stdout
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            if trimmed.starts_with("(use \"git")
                || trimmed.starts_with("(create/copy files")
                || trimmed.contains("(use \"git add")
                || trimmed.contains("(use \"git restore")
            {
                return None;
            }
            Some(line.trim_end().to_string())
        })
        .collect::<Vec<_>>();

    if filtered.is_empty() {
        println!("ok");
    } else {
        println!("{}", filtered.join("\n"));
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_without_message_passthroughs() {
        let invocation = build_invocation(&["commit".to_string()]);
        assert!(invocation.passthrough);
    }

    #[test]
    fn status_adds_short_branch_flags() {
        let invocation = build_invocation(&["status".to_string()]);
        assert_eq!(
            invocation.cmd,
            vec![
                "git".to_string(),
                "status".to_string(),
                "--short".to_string(),
                "--branch".to_string()
            ]
        );
        assert!(invocation.parse_status);
    }

    #[test]
    fn status_with_human_flags_does_not_force_short_mode() {
        let invocation = build_invocation(&["status".to_string(), "--ignored".to_string()]);
        assert_eq!(
            invocation.cmd,
            vec![
                "git".to_string(),
                "status".to_string(),
                "--ignored".to_string()
            ]
        );
        assert!(invocation.filter_status_human);
        assert!(!invocation.parse_status);
    }

    #[test]
    fn classify_status_tracks_index_and_worktree() {
        let labels = classify_status_codes("AM file.txt");
        assert!(labels.contains(&"added"));
        assert!(labels.contains(&"modified"));
    }

    #[test]
    fn diff_with_stat_passthroughs() {
        let invocation = build_invocation(&[
            "diff".to_string(),
            "--stat".to_string(),
            "HEAD~1".to_string(),
        ]);
        assert!(invocation.passthrough);
    }

    #[test]
    fn diff_with_range_and_pathspec_passthroughs() {
        let invocation = build_invocation(&[
            "diff".to_string(),
            "origin/main..HEAD".to_string(),
            "--".to_string(),
            "src/main.rs".to_string(),
        ]);
        assert!(invocation.passthrough);
    }

    #[test]
    fn diff_with_function_context_passthroughs() {
        let invocation =
            build_invocation(&["diff".to_string(), "-W".to_string(), "HEAD~1".to_string()]);
        assert!(invocation.passthrough);
    }

    #[test]
    fn log_with_line_history_passthroughs() {
        let invocation = build_invocation(&[
            "log".to_string(),
            "-L".to_string(),
            ":handle:src/main.rs".to_string(),
        ]);
        assert!(invocation.passthrough);
    }
}
