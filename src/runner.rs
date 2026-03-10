use std::env;
use std::path::Path;
use std::process::{Command, Stdio};

pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub returncode: i32,
}

pub fn command_exists(name: &str) -> bool {
    if name.contains('/') {
        return Path::new(name).is_file();
    }

    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

pub fn run_command(parts: &[String]) -> CommandResult {
    if parts.is_empty() {
        return CommandResult {
            stdout: String::new(),
            stderr: "missing command".to_string(),
            returncode: 2,
        };
    }

    match Command::new(&parts[0])
        .args(&parts[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
    {
        Ok(output) => CommandResult {
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            returncode: output.status.code().unwrap_or(1),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => CommandResult {
            stdout: String::new(),
            stderr: format!("command not found: {}", parts[0]),
            returncode: 127,
        },
        Err(error) => CommandResult {
            stdout: String::new(),
            stderr: format!("failed to execute {}: {error}", parts[0]),
            returncode: 1,
        },
    }
}

pub fn run_command_passthrough(parts: &[String]) -> i32 {
    if parts.is_empty() {
        eprintln!("missing command");
        return 2;
    }

    match Command::new(&parts[0])
        .args(&parts[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => status.code().unwrap_or(1),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!("command not found: {}", parts[0]);
            127
        }
        Err(error) => {
            eprintln!("failed to execute {}: {error}", parts[0]);
            1
        }
    }
}
