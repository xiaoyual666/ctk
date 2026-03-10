use std::fs;
use std::path::{Path, PathBuf};

pub fn handle(path: String, max_depth: usize, max_entries: usize) -> i32 {
    let root = PathBuf::from(&path);
    if !root.exists() {
        eprintln!("ctk ls: path not found: {}", root.display());
        return 2;
    }

    println!("{}", label(&root, &root));
    if root.is_dir() {
        walk(&root, &root, 0, max_depth, max_entries, "");
    }
    0
}

fn walk(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_entries: usize,
    prefix: &str,
) {
    if depth >= max_depth {
        return;
    }

    let entries = match fs::read_dir(current) {
        Ok(entries) => entries,
        Err(_) => {
            println!("{prefix}  [unreadable]");
            return;
        }
    };

    let mut entries = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| {
        (
            !entry.is_dir(),
            entry
                .file_name()
                .map(|name| name.to_string_lossy().to_lowercase()),
        )
    });

    let shown = entries
        .iter()
        .take(max_entries)
        .cloned()
        .collect::<Vec<_>>();
    for (index, entry) in shown.iter().enumerate() {
        let last = index == shown.len().saturating_sub(1) && entries.len() == shown.len();
        let branch = if last { "└─ " } else { "├─ " };
        let child_prefix = if last {
            format!("{prefix}   ")
        } else {
            format!("{prefix}│  ")
        };
        println!("{prefix}{branch}{}", label(entry, root));
        if entry.is_dir() && !entry.is_symlink() {
            walk(
                root,
                entry,
                depth + 1,
                max_depth,
                max_entries,
                &child_prefix,
            );
        }
    }

    if entries.len() > max_entries {
        println!(
            "{prefix}└─ ... {} more entries omitted ...",
            entries.len() - max_entries
        );
    }
}

fn label(path: &Path, root: &Path) -> String {
    if path == root {
        let root_name = path
            .canonicalize()
            .ok()
            .and_then(|resolved| resolved.file_name().map(|name| name.to_os_string()))
            .or_else(|| path.file_name().map(|name| name.to_os_string()))
            .unwrap_or_else(|| ".".into());
        return if path.is_dir() {
            format!("{}/", root_name.to_string_lossy())
        } else {
            root_name.to_string_lossy().into_owned()
        };
    }
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    if path.is_dir() {
        format!("{name}/")
    } else {
        name.into_owned()
    }
}
