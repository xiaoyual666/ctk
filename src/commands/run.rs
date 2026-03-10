use crate::runner::run_command;
use crate::text::compact_block;

pub fn handle(cmd: Vec<String>) -> i32 {
    if cmd.is_empty() {
        eprintln!("ctk run: missing command");
        return 2;
    }

    let result = run_command(&cmd);
    if result.returncode == 0 {
        if !result.stdout.trim().is_empty() {
            println!("{}", compact_block(&result.stdout, 120));
        }
        if !result.stderr.trim().is_empty() {
            eprintln!("{}", compact_block(&result.stderr, 120));
        }
        if result.stdout.trim().is_empty() && result.stderr.trim().is_empty() {
            println!("ok");
        }
    } else {
        if !result.stderr.trim().is_empty() {
            eprintln!("{}", compact_block(&result.stderr, 120));
        }
        if !result.stdout.trim().is_empty() {
            eprintln!("{}", compact_block(&result.stdout, 120));
        }
        if result.stdout.trim().is_empty() && result.stderr.trim().is_empty() {
            eprintln!("command failed");
        }
    }
    result.returncode
}
