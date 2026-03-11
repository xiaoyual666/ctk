use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir(name: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("ctk-{name}-{suffix}"));
    fs::create_dir_all(&dir).expect("mkdir");
    dir
}

#[test]
fn json_prints_shape() {
    let dir = temp_dir("json");
    let file = dir.join("sample.json");
    fs::write(&file, r#"{"user":{"id":1,"tags":["a"]}}"#).expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["json", file.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("user: object<2>"));
    assert!(stdout.contains("tags: array<1>"));
}

#[test]
fn read_aggressive_keeps_symbols() {
    let dir = temp_dir("read");
    let file = dir.join("sample.py");
    fs::write(
        &file,
        "import os\n\n\ndef alpha():\n    return 1\n\nvalue = 2\n",
    )
    .expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args([
            "read",
            file.to_str().expect("path"),
            "--level",
            "aggressive",
        ])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 | import os"));
    assert!(stdout.contains("4 | def alpha():"));
}

#[test]
fn sed_extracts_line_range_exactly() {
    let dir = temp_dir("sed-range");
    let file = dir.join("sample.txt");
    fs::write(&file, "one\ntwo\nthree\nfour\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["sed", "-n", "2,3p", file.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "two\nthree\n");
}

#[test]
fn sed_requires_read_only_print_shape() {
    let dir = temp_dir("sed-unsupported");
    let file = dir.join("sample.txt");
    fs::write(&file, "one\ntwo\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["sed", "s/one/two/", file.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("only read-only print expressions"));
}

#[test]
fn find_groups_matches() {
    let dir = temp_dir("find");
    let src = dir.join("src");
    fs::create_dir_all(&src).expect("mkdir");
    fs::write(src.join("a.rs"), "").expect("write");
    fs::write(src.join("b.rs"), "").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["find", "*.rs", dir.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("src/ (2)"));
}

#[test]
fn deps_summarizes_cargo_project() {
    let dir = temp_dir("deps-cargo");
    fs::write(
        dir.join("Cargo.toml"),
        r#"[package]
name = "demo"
version = "0.1.0"

[dependencies]
clap = "4"
serde = { version = "1" }

[dev-dependencies]
tempfile = "3"
"#,
    )
    .expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["deps", dir.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rust: demo @ 0.1.0"));
    assert!(stdout.contains("clap 4"));
    assert!(stdout.contains("tempfile 3"));
}

#[test]
fn ls_supports_all_and_long_flags() {
    let dir = temp_dir("ls-all-long");
    fs::write(dir.join(".secret"), "x").expect("write hidden");
    fs::write(dir.join("visible.txt"), "hello").expect("write visible");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["ls", "-la", dir.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(".secret"));
    assert!(stdout.contains("visible.txt 5B"));
}

#[test]
fn git_status_uses_machine_friendly_format() {
    let dir = temp_dir("git-status");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    fs::write(dir.join("a.txt"), "hello\n").expect("write");
    Command::new("git")
        .args(["add", "a.txt"])
        .current_dir(&dir)
        .output()
        .expect("git add");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["git", "status"])
        .current_dir(&dir)
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("added 1"));
    assert!(!stdout.contains("changed 3"));
}

#[test]
fn git_branch_keeps_listing_output() {
    let dir = temp_dir("git-branch");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "ctk@example.com"])
        .current_dir(&dir)
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "CTK Test"])
        .current_dir(&dir)
        .output()
        .expect("git config name");
    fs::write(dir.join("a.txt"), "hello\n").expect("write");
    Command::new("git")
        .args(["add", "a.txt"])
        .current_dir(&dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "init"])
        .current_dir(&dir)
        .output()
        .expect("git commit");
    Command::new("git")
        .args(["checkout", "-q", "-b", "feature/demo"])
        .current_dir(&dir)
        .output()
        .expect("git checkout");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["git", "branch", "-a"])
        .current_dir(&dir)
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("feature/demo"));
    assert!(!stdout.contains("ok branch"));
}

#[test]
fn git_status_with_args_keeps_human_output_without_hints() {
    let dir = temp_dir("git-status-human");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    fs::write(dir.join("a.txt"), "hello\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["git", "status", "--ignored"])
        .current_dir(&dir)
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Untracked files:"));
    assert!(!stdout.contains("(use \"git add"));
}

#[test]
fn git_diff_stat_passthroughs_raw_output() {
    let dir = temp_dir("git-diff-stat");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "ctk@example.com"])
        .current_dir(&dir)
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "CTK Test"])
        .current_dir(&dir)
        .output()
        .expect("git config name");
    fs::write(dir.join("a.txt"), "hello\n").expect("write");
    Command::new("git")
        .args(["add", "a.txt"])
        .current_dir(&dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "init"])
        .current_dir(&dir)
        .output()
        .expect("git commit");
    fs::write(dir.join("a.txt"), "hello\nworld\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["git", "diff", "--stat"])
        .current_dir(&dir)
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("a.txt"));
    assert!(stdout.contains('|'));
}

#[test]
fn git_show_blob_passthroughs_exact_file_content() {
    let dir = temp_dir("git-show-blob");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "ctk@example.com"])
        .current_dir(&dir)
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "CTK Test"])
        .current_dir(&dir)
        .output()
        .expect("git config name");
    fs::write(dir.join("a.txt"), "hello\n").expect("write");
    Command::new("git")
        .args(["add", "a.txt"])
        .current_dir(&dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "init"])
        .current_dir(&dir)
        .output()
        .expect("git commit");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["git", "show", "HEAD:a.txt"])
        .current_dir(&dir)
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "hello\n");
}

#[test]
fn git_diff_range_and_pathspec_passthroughs_patch() {
    let dir = temp_dir("git-diff-range");
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(&dir)
        .output()
        .expect("git init");
    Command::new("git")
        .args(["config", "user.email", "ctk@example.com"])
        .current_dir(&dir)
        .output()
        .expect("git config email");
    Command::new("git")
        .args(["config", "user.name", "CTK Test"])
        .current_dir(&dir)
        .output()
        .expect("git config name");
    fs::create_dir_all(dir.join("src")).expect("mkdir");
    fs::write(dir.join("src").join("a.txt"), "hello\n").expect("write");
    Command::new("git")
        .args(["add", "."])
        .current_dir(&dir)
        .output()
        .expect("git add");
    Command::new("git")
        .args(["commit", "-qm", "init"])
        .current_dir(&dir)
        .output()
        .expect("git commit");
    fs::write(dir.join("src").join("a.txt"), "hello\nworld\n").expect("write");
    Command::new("git")
        .args(["add", "src/a.txt"])
        .current_dir(&dir)
        .output()
        .expect("git add second");
    Command::new("git")
        .args(["commit", "-qm", "update"])
        .current_dir(&dir)
        .output()
        .expect("git commit second");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["git", "diff", "HEAD~1..HEAD", "--", "src/a.txt"])
        .current_dir(&dir)
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("diff --git"));
    assert!(stdout.contains("src/a.txt"));
}

#[test]
fn run_preserves_stderr_on_failure() {
    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args([
            "run",
            "bash",
            "-lc",
            "echo stdout-line; echo stderr-line >&2; exit 1",
        ])
        .output()
        .expect("run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("stderr-line"));
    assert!(stderr.contains("stdout-line"));
}

#[test]
fn grep_handles_colon_in_filenames() {
    let dir = temp_dir("grep-colon");
    fs::write(dir.join("a:b.txt"), "match\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["grep", "match", dir.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("a:b.txt"));
}

#[test]
fn grep_accepts_line_number_flag_shape() {
    let dir = temp_dir("grep-flag-shape");
    fs::write(dir.join("sample.txt"), "match\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["grep", "-n", "match", dir.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sample.txt"));
    assert!(stdout.contains("1: match"));
}

#[test]
fn grep_supports_case_insensitive_fixed_string_search() {
    let dir = temp_dir("grep-fixed-ignore");
    fs::write(dir.join("sample.txt"), "EndGame\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["grep", "-i", "-F", "endgame", dir.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sample.txt"));
    assert!(stdout.contains("1: EndGame"));
}

#[test]
fn grep_accepts_multiple_paths() {
    let dir = temp_dir("grep-multi-path");
    let one = dir.join("one");
    let two = dir.join("two");
    fs::create_dir_all(&one).expect("mkdir one");
    fs::create_dir_all(&two).expect("mkdir two");
    fs::write(one.join("a.txt"), "match\n").expect("write one");
    fs::write(two.join("b.txt"), "match\n").expect("write two");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args([
            "grep",
            "match",
            one.to_str().expect("path one"),
            two.to_str().expect("path two"),
        ])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("a.txt"));
    assert!(stdout.contains("b.txt"));
}

#[test]
fn test_does_not_treat_smoke_words_as_success_markers() {
    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args([
            "test",
            "bash",
            "-lc",
            "echo smoke-finished; echo broker-ready; exit 0",
        ])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("smoke-finished"));
    assert!(!stdout.contains("broker-ready"));
    assert!(stdout.contains("ok"));
}

#[test]
fn read_zero_lines_does_not_panic() {
    let dir = temp_dir("read-zero");
    let file = dir.join("sample.txt");
    fs::write(&file, "hello\n").expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["read", file.to_str().expect("path"), "--max-lines", "0"])
        .output()
        .expect("run");

    assert!(output.status.success());
}

#[test]
fn read_binary_reports_binary_file() {
    let dir = temp_dir("read-binary");
    let file = dir.join("sample.bin");
    fs::write(&file, [0_u8, 1, 2, 3]).expect("write");

    let output = Command::new(env!("CARGO_BIN_EXE_ctk"))
        .args(["read", file.to_str().expect("path")])
        .output()
        .expect("run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("binary file:"));
}
