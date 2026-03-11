use std::fs;
use std::io::{self, Read};
use std::path::Path;

pub fn handle(quiet: bool, script: String, files: Vec<String>) -> i32 {
    if !quiet {
        eprintln!("ctk sed: only read-only print expressions are supported; use -n");
        return 2;
    }

    if files.len() > 1 {
        eprintln!("ctk sed: only a single input file or stdin is currently supported");
        return 2;
    }

    let clauses = match parse_script(&script) {
        Ok(clauses) => clauses,
        Err(error) => {
            eprintln!("ctk sed: {error}");
            return 2;
        }
    };

    let input = files.first().cloned().unwrap_or_else(|| "-".to_string());
    let content = if input == "-" {
        match read_stdin() {
            Ok(content) => content,
            Err(error) => {
                eprintln!("ctk sed: failed to read stdin: {error}");
                return 1;
            }
        }
    } else {
        match read_file(Path::new(&input)) {
            Ok(content) => content,
            Err(ReadError::Binary(size)) => {
                println!("binary file: {} ({} bytes)", input, size);
                return 0;
            }
            Err(ReadError::Io(error)) => {
                eprintln!("ctk sed: failed to read {}: {error}", input);
                return 1;
            }
        }
    };

    let lines = split_lines_preserve_newlines(&content);
    let rendered = render_clauses(&clauses, &lines);
    print!("{rendered}");
    0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Address {
    Line(usize),
    Last,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Clause {
    All,
    Single(Address),
    Range(Address, Address),
}

fn parse_script(script: &str) -> Result<Vec<Clause>, String> {
    let mut clauses = Vec::new();
    for raw_clause in script
        .split(';')
        .map(str::trim)
        .filter(|clause| !clause.is_empty())
    {
        if !raw_clause.ends_with('p') {
            return Err(format!(
                "unsupported script `{raw_clause}`; only print expressions like `1,200p` are supported"
            ));
        }
        let body = raw_clause[..raw_clause.len() - 1].trim();
        if body.is_empty() {
            clauses.push(Clause::All);
            continue;
        }
        if let Some((start, end)) = body.split_once(',') {
            clauses.push(Clause::Range(
                parse_address(start.trim())?,
                parse_address(end.trim())?,
            ));
        } else {
            clauses.push(Clause::Single(parse_address(body)?));
        }
    }

    if clauses.is_empty() {
        return Err("missing print expression".to_string());
    }
    Ok(clauses)
}

fn parse_address(raw: &str) -> Result<Address, String> {
    if raw == "$" {
        return Ok(Address::Last);
    }
    let line = raw
        .parse::<usize>()
        .map_err(|_| format!("unsupported address `{raw}`"))?;
    if line == 0 {
        return Err("line numbers are 1-based".to_string());
    }
    Ok(Address::Line(line))
}

fn split_lines_preserve_newlines(content: &str) -> Vec<String> {
    if content.is_empty() {
        return Vec::new();
    }
    content
        .split_inclusive('\n')
        .map(|line| line.to_string())
        .collect::<Vec<_>>()
}

fn render_clauses(clauses: &[Clause], lines: &[String]) -> String {
    let total_lines = lines.len();
    let mut out = String::new();
    for (index, line) in lines.iter().enumerate() {
        let line_number = index + 1;
        for clause in clauses {
            if clause_matches(*clause, line_number, total_lines) {
                out.push_str(line);
            }
        }
    }
    out
}

fn clause_matches(clause: Clause, line_number: usize, total_lines: usize) -> bool {
    match clause {
        Clause::All => true,
        Clause::Single(address) => resolve_address(address, total_lines) == line_number,
        Clause::Range(start, end) => {
            let start = resolve_address(start, total_lines);
            let end = resolve_address(end, total_lines);
            line_number >= start && line_number <= end
        }
    }
}

fn resolve_address(address: Address, total_lines: usize) -> usize {
    match address {
        Address::Line(line) => line,
        Address::Last => total_lines.max(1),
    }
}

enum ReadError {
    Binary(usize),
    Io(io::Error),
}

fn read_file(path: &Path) -> Result<String, ReadError> {
    let bytes = fs::read(path).map_err(ReadError::Io)?;
    if bytes.contains(&0) {
        return Err(ReadError::Binary(bytes.len()));
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn read_stdin() -> io::Result<String> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_common_range_expression() {
        assert_eq!(
            parse_script("1,20p").expect("parse"),
            vec![Clause::Range(Address::Line(1), Address::Line(20))]
        );
    }

    #[test]
    fn parse_last_line_expression() {
        assert_eq!(
            parse_script("5,$p").expect("parse"),
            vec![Clause::Range(Address::Line(5), Address::Last)]
        );
    }

    #[test]
    fn render_exact_lines() {
        let lines = split_lines_preserve_newlines("a\nb\nc\n");
        let clauses = parse_script("2,3p").expect("parse");
        assert_eq!(render_clauses(&clauses, &lines), "b\nc\n");
    }

    #[test]
    fn duplicate_prints_are_preserved() {
        let lines = split_lines_preserve_newlines("a\nb\n");
        let clauses = parse_script("2p;2p").expect("parse");
        assert_eq!(render_clauses(&clauses, &lines), "b\nb\n");
    }
}
