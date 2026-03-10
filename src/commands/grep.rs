use crate::runner::{command_exists, run_command};
use regex::{Regex, RegexBuilder};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

pub fn handle(
    pattern: String,
    paths: Vec<String>,
    max_files: usize,
    max_matches_per_file: usize,
    _line_number: bool,
    _recursive: bool,
    ignore_case: bool,
    fixed_strings: bool,
) -> i32 {
    let paths = if paths.is_empty() {
        vec![".".to_string()]
    } else {
        paths
    };
    if command_exists("rg") {
        handle_with_rg(
            pattern,
            paths,
            max_files,
            max_matches_per_file,
            ignore_case,
            fixed_strings,
        )
    } else {
        handle_with_walk(
            pattern,
            paths,
            max_files,
            max_matches_per_file,
            ignore_case,
            fixed_strings,
        )
    }
}

fn handle_with_rg(
    pattern: String,
    paths: Vec<String>,
    max_files: usize,
    max_matches_per_file: usize,
    ignore_case: bool,
    fixed_strings: bool,
) -> i32 {
    let mut cmd = vec![
        "rg".to_string(),
        "--json".to_string(),
        "--color".to_string(),
        "never".to_string(),
    ];
    if ignore_case {
        cmd.push("-i".to_string());
    }
    if fixed_strings {
        cmd.push("-F".to_string());
    }
    cmd.push(pattern);
    cmd.extend(paths);
    let result = run_command(&cmd);
    if !matches!(result.returncode, 0 | 1) {
        eprintln!("{}", result.stderr.trim());
        return result.returncode;
    }

    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in result.stdout.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(value) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if value.get("type").and_then(Value::as_str) != Some("match") {
            continue;
        }
        let data = &value["data"];
        let file_name = data["path"]["text"]
            .as_str()
            .unwrap_or("<stdin>")
            .to_string();
        let line_no = data["line_number"].as_u64().unwrap_or(0);
        let raw_text = data["lines"]["text"].as_str().unwrap_or_default();
        let snippet = first_display_line(raw_text);
        let rendered = if line_no > 0 {
            format!("{line_no}: {snippet}")
        } else {
            snippet
        };
        grouped.entry(file_name).or_default().push(rendered);
    }
    print_grouped(grouped, max_files, max_matches_per_file)
}

fn handle_with_walk(
    pattern: String,
    paths: Vec<String>,
    max_files: usize,
    max_matches_per_file: usize,
    ignore_case: bool,
    fixed_strings: bool,
) -> i32 {
    let matcher = if fixed_strings {
        None
    } else {
        match RegexBuilder::new(&pattern)
            .case_insensitive(ignore_case)
            .build()
        {
            Ok(matcher) => Some(matcher),
            Err(error) => {
                eprintln!("ctk grep: invalid regex: {error}");
                return 2;
            }
        }
    };
    let pattern_cmp = if ignore_case {
        pattern.to_lowercase()
    } else {
        pattern.clone()
    };
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for path in paths {
        let root = PathBuf::from(path);
        if !root.exists() {
            eprintln!("ctk grep: path not found: {}", root.display());
            return 2;
        }
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(_) => continue,
            };
            for (idx, line) in content.lines().enumerate() {
                if matches_line(
                    line,
                    matcher.as_ref(),
                    &pattern_cmp,
                    ignore_case,
                    fixed_strings,
                ) {
                    grouped
                        .entry(entry.path().display().to_string())
                        .or_default()
                        .push(format!("{}: {}", idx + 1, line.trim_end()));
                }
            }
        }
    }
    print_grouped(grouped, max_files, max_matches_per_file)
}

fn print_grouped(
    grouped: BTreeMap<String, Vec<String>>,
    max_files: usize,
    max_matches_per_file: usize,
) -> i32 {
    if grouped.is_empty() {
        println!("0 matches");
        return 1;
    }

    let total_matches = grouped.values().map(|matches| matches.len()).sum::<usize>();
    let file_names = grouped.keys().cloned().collect::<Vec<_>>();
    for file_name in file_names.iter().take(max_files) {
        let matches = &grouped[file_name];
        println!("{file_name} ({} matches)", matches.len());
        for item in matches.iter().take(max_matches_per_file) {
            println!("  {item}");
        }
        if matches.len() > max_matches_per_file {
            println!(
                "  ... {} more matches omitted ...",
                matches.len() - max_matches_per_file
            );
        }
    }
    if file_names.len() > max_files {
        println!(
            "... {} more files omitted ...",
            file_names.len() - max_files
        );
    }
    println!(
        "summary: {} files, {total_matches} matches",
        file_names.len()
    );
    0
}

fn first_display_line(text: &str) -> String {
    let mut lines = text.lines();
    let first = lines.next().unwrap_or_default().trim_end().to_string();
    if lines.next().is_some() {
        format!("{first} ...")
    } else {
        first
    }
}

fn matches_line(
    line: &str,
    matcher: Option<&Regex>,
    pattern: &str,
    ignore_case: bool,
    fixed_strings: bool,
) -> bool {
    if fixed_strings {
        if ignore_case {
            line.to_lowercase().contains(pattern)
        } else {
            line.contains(pattern)
        }
    } else {
        matcher
            .map(|matcher| matcher.is_match(line))
            .unwrap_or(false)
    }
}
