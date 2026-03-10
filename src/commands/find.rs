use glob::Pattern;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn handle(pattern: String, path: String, max_results: usize) -> i32 {
    let root = PathBuf::from(&path);
    if !root.exists() {
        eprintln!("ctk find: path not found: {}", root.display());
        return 2;
    }

    let matcher = match Pattern::new(&pattern) {
        Ok(matcher) => matcher,
        Err(error) => {
            eprintln!("ctk find: invalid pattern: {error}");
            return 2;
        }
    };

    let mut matches = WalkDir::new(&root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|entry| matcher.matches_path(entry))
        .collect::<Vec<_>>();
    matches.sort();

    if matches.is_empty() {
        println!("0 matches");
        return 1;
    }

    let limited = matches
        .iter()
        .take(max_results)
        .cloned()
        .collect::<Vec<_>>();
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for item in limited {
        let parent = item.parent().unwrap_or(Path::new("."));
        let rel_parent = parent
            .strip_prefix(&root)
            .unwrap_or(parent)
            .display()
            .to_string();
        grouped
            .entry(if rel_parent.is_empty() {
                ".".to_string()
            } else {
                rel_parent
            })
            .or_default()
            .push(
                item.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
            );
    }

    for (parent, items) in grouped {
        println!("{parent}/ ({})", items.len());
        for item in items {
            println!("  {item}");
        }
    }

    if matches.len() > max_results {
        println!(
            "... {} more matches omitted ...",
            matches.len() - max_results
        );
    }

    0
}
