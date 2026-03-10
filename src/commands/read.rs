use crate::text::head_tail;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

pub fn handle(path: String, max_lines: usize, level: String) -> i32 {
    let content = if path == "-" {
        match read_stdin() {
            Ok(content) => content,
            Err(error) => {
                eprintln!("ctk read: failed to read stdin: {error}");
                return 1;
            }
        }
    } else {
        let file = PathBuf::from(&path);
        if !file.exists() {
            eprintln!("ctk read: file not found: {}", file.display());
            return 2;
        }
        if !file.is_file() {
            eprintln!("ctk read: not a file: {}", file.display());
            return 2;
        }

        match read_file(&file) {
            Ok(content) => content,
            Err(ReadError::Binary(size)) => {
                println!("binary file: {} ({} bytes)", file.display(), size);
                return 0;
            }
            Err(ReadError::Io(error)) => {
                eprintln!("ctk read: failed to read {}: {error}", file.display());
                return 1;
            }
        }
    };
    let lines = content.lines().collect::<Vec<_>>();
    let selected = if level == "aggressive" {
        let matches = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| is_symbol_line(line))
            .map(|(idx, line)| format!("{} | {}", idx + 1, line.trim_end()))
            .collect::<Vec<_>>();
        if matches.is_empty() {
            lines
                .iter()
                .enumerate()
                .map(|(idx, line)| format!("{} | {}", idx + 1, line.trim_end()))
                .collect::<Vec<_>>()
        } else {
            matches
        }
    } else {
        lines
            .iter()
            .enumerate()
            .map(|(idx, line)| format!("{} | {}", idx + 1, line.trim_end()))
            .collect::<Vec<_>>()
    };

    if selected.is_empty() {
        println!("empty file");
        return 0;
    }

    let compact = head_tail(&selected, max_lines);
    if !compact.is_empty() {
        println!("{}", compact.join("\n"));
    }
    0
}

enum ReadError {
    Binary(usize),
    Io(io::Error),
}

fn read_file(path: &PathBuf) -> Result<String, ReadError> {
    let bytes = fs::read(path).map_err(ReadError::Io)?;
    if is_binary(&bytes) {
        return Err(ReadError::Binary(bytes.len()));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_stdin() -> io::Result<String> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn is_binary(bytes: &[u8]) -> bool {
    bytes.contains(&0)
}

fn is_symbol_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    const PREFIXES: &[&str] = &[
        "use ",
        "mod ",
        "pub mod ",
        "import ",
        "from ",
        "class ",
        "def ",
        "async def ",
        "export ",
        "function ",
        "interface ",
        "type ",
        "struct ",
        "enum ",
        "trait ",
        "impl ",
        "fn ",
        "pub fn ",
        "pub struct ",
        "pub enum ",
        "pub trait ",
        "#include ",
        "package ",
    ];
    PREFIXES.iter().any(|prefix| trimmed.starts_with(prefix))
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("var ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_symbol_lines() {
        assert!(is_symbol_line("def alpha():"));
        assert!(is_symbol_line("pub fn handle() {}"));
        assert!(!is_symbol_line("return value"));
    }

    #[test]
    fn detects_binary_content() {
        assert!(is_binary(&[0, 1, 2]));
        assert!(!is_binary(b"hello"));
    }
}
