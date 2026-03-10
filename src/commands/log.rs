use std::fs;
use std::path::PathBuf;

pub fn handle(path: String, max_lines: usize) -> i32 {
    let file = PathBuf::from(&path);
    if !file.exists() {
        eprintln!("ctk log: file not found: {}", file.display());
        return 2;
    }

    let content = match fs::read_to_string(&file) {
        Ok(content) => content,
        Err(error) => {
            eprintln!("ctk log: failed to read {}: {error}", file.display());
            return 1;
        }
    };
    let mut lines = content
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        println!("empty log");
        return 0;
    }
    if lines.len() > max_lines {
        lines = lines[lines.len() - max_lines..].to_vec();
    }
    for line in dedupe_runs(&lines) {
        println!("{line}");
    }
    0
}

fn dedupe_runs(lines: &[String]) -> Vec<String> {
    if lines.is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut current = lines[0].clone();
    let mut count = 1usize;
    for line in &lines[1..] {
        if line == &current {
            count += 1;
        } else {
            result.push(format_run(&current, count));
            current = line.clone();
            count = 1;
        }
    }
    result.push(format_run(&current, count));
    result
}

fn format_run(line: &str, count: usize) -> String {
    if count == 1 {
        return line.to_string();
    }
    format!("[x{count}] {line}")
}
