use crate::runner::{command_exists, run_command, run_command_passthrough};
use crate::text::compact_block;

pub fn handle(args: Vec<String>) -> i32 {
    if !command_exists("gh") {
        println!("{}", unavailable_message());
        return 0;
    }
    if args.is_empty() {
        eprintln!("ctk gh: missing arguments");
        return 2;
    }

    let mut cmd = vec!["gh".to_string()];
    cmd.extend(args);
    if should_passthrough(&cmd[1..]) {
        return run_command_passthrough(&cmd);
    }
    let result = run_command(&cmd);
    let stream = if result.stdout.trim().is_empty() {
        &result.stderr
    } else {
        &result.stdout
    };
    if !stream.trim().is_empty() {
        println!("{}", compact_block(stream, 80));
    } else if result.returncode == 0 {
        println!("ok");
    }
    result.returncode
}

fn unavailable_message() -> String {
    "unavailable: gh is not installed on this machine".to_string()
}

fn should_passthrough(args: &[String]) -> bool {
    if args.is_empty() {
        return false;
    }

    let structured_flags = [
        "--json",
        "--jq",
        "--template",
        "--web",
        "--log",
        "--log-failed",
    ];
    if args
        .iter()
        .any(|arg| structured_flags.contains(&arg.as_str()))
    {
        return true;
    }

    matches!(
        args.first().map(String::as_str),
        Some("auth" | "browse" | "alias" | "config" | "extension")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_message_is_llm_friendly() {
        assert_eq!(
            unavailable_message(),
            "unavailable: gh is not installed on this machine"
        );
    }

    #[test]
    fn json_flags_passthrough() {
        assert!(should_passthrough(&[
            "pr".to_string(),
            "view".to_string(),
            "--json".to_string(),
        ]));
    }

    #[test]
    fn auth_passthroughs() {
        assert!(should_passthrough(&[
            "auth".to_string(),
            "login".to_string()
        ]));
    }
}
