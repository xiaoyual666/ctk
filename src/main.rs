mod commands;
mod runner;
mod text;

use clap::{ArgAction, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ctk", about = "Compact terminal kit for Codex-first workflows")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Git {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Gh {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    Ls {
        #[arg(short = 'a', long = "all", action = ArgAction::SetTrue)]
        all: bool,
        #[arg(short = 'l', long = "long", action = ArgAction::SetTrue)]
        long: bool,
        #[arg(short = 'R', long = "recursive", action = ArgAction::SetTrue)]
        recursive: bool,
        #[arg(num_args = 0..)]
        paths: Vec<String>,
        #[arg(long, default_value_t = 2)]
        max_depth: usize,
        #[arg(long, default_value_t = 12)]
        max_entries: usize,
    },
    Read {
        path: String,
        #[arg(long, default_value_t = 80)]
        max_lines: usize,
        #[arg(short = 'l', long, default_value = "normal")]
        level: String,
    },
    Sed {
        #[arg(short = 'n', action = ArgAction::SetTrue)]
        quiet: bool,
        script: String,
        #[arg(num_args = 0..)]
        files: Vec<String>,
    },
    Grep {
        #[arg(short = 'n', long = "line-number", action = ArgAction::SetTrue)]
        line_number: bool,
        #[arg(short = 'r', long = "recursive", action = ArgAction::SetTrue)]
        recursive: bool,
        #[arg(short = 'i', long = "ignore-case", action = ArgAction::SetTrue)]
        ignore_case: bool,
        #[arg(short = 'F', long = "fixed-strings", action = ArgAction::SetTrue)]
        fixed_strings: bool,
        pattern: String,
        #[arg(num_args = 0..)]
        paths: Vec<String>,
        #[arg(long, default_value_t = 20)]
        max_files: usize,
        #[arg(long, default_value_t = 3)]
        max_matches_per_file: usize,
    },
    Find {
        pattern: String,
        #[arg(default_value = ".")]
        path: String,
        #[arg(long, default_value_t = 50)]
        max_results: usize,
    },
    Deps {
        #[arg(default_value = ".")]
        path: String,
    },
    Test {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    Err {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
    Log {
        path: String,
        #[arg(long, default_value_t = 120)]
        max_lines: usize,
    },
    Json {
        path: String,
    },
    Run {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        cmd: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        Commands::Git { args } => commands::git::handle(args),
        Commands::Gh { args } => commands::gh::handle(args),
        Commands::Ls {
            all,
            long,
            recursive,
            paths,
            max_depth,
            max_entries,
        } => commands::ls::handle(paths, all, long, recursive, max_depth, max_entries),
        Commands::Read {
            path,
            max_lines,
            level,
        } => commands::read::handle(path, max_lines, level),
        Commands::Sed {
            quiet,
            script,
            files,
        } => commands::sed::handle(quiet, script, files),
        Commands::Grep {
            line_number,
            recursive,
            ignore_case,
            fixed_strings,
            pattern,
            paths,
            max_files,
            max_matches_per_file,
        } => commands::grep::handle(
            pattern,
            paths,
            max_files,
            max_matches_per_file,
            line_number,
            recursive,
            ignore_case,
            fixed_strings,
        ),
        Commands::Find {
            pattern,
            path,
            max_results,
        } => commands::find::handle(pattern, path, max_results),
        Commands::Deps { path } => commands::deps::handle(path),
        Commands::Test { cmd } => commands::test::handle(cmd),
        Commands::Err { cmd } => commands::err::handle(cmd),
        Commands::Log { path, max_lines } => commands::log::handle(path, max_lines),
        Commands::Json { path } => commands::json::handle(path),
        Commands::Run { cmd } => commands::run::handle(cmd),
    };
    std::process::exit(code);
}
