use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

pub fn handle(
    paths: Vec<String>,
    show_all: bool,
    long: bool,
    recursive: bool,
    max_depth: usize,
    max_entries: usize,
) -> i32 {
    let roots = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.into_iter().map(PathBuf::from).collect::<Vec<_>>()
    };

    let depth_limit = if recursive { usize::MAX } else { max_depth };

    for (index, root) in roots.iter().enumerate() {
        if !root.exists() {
            eprintln!("ctk ls: path not found: {}", root.display());
            return 2;
        }

        if index > 0 {
            println!();
        }
        print_root(root, show_all, long, depth_limit, max_entries);
    }
    0
}

fn print_root(root: &Path, show_all: bool, long: bool, max_depth: usize, max_entries: usize) {
    println!("{}", label(root, root, long));
    if root.is_dir() {
        walk(root, root, 0, max_depth, max_entries, show_all, long, "");
    }
}

fn walk(
    root: &Path,
    current: &Path,
    depth: usize,
    max_depth: usize,
    max_entries: usize,
    show_all: bool,
    long: bool,
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
        .filter(|entry| should_show(entry, show_all))
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
        println!("{prefix}{branch}{}", label(entry, root, long));
        if entry.is_dir() && !entry.is_symlink() {
            walk(
                root,
                entry,
                depth + 1,
                max_depth,
                max_entries,
                show_all,
                long,
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

fn should_show(path: &Path, show_all: bool) -> bool {
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or_default();
    if show_all {
        return true;
    }
    !name.starts_with('.')
}

fn label(path: &Path, root: &Path, long: bool) -> String {
    let base = if path == root {
        let root_name = path
            .canonicalize()
            .ok()
            .and_then(|resolved| resolved.file_name().map(|name| name.to_os_string()))
            .or_else(|| path.file_name().map(|name| name.to_os_string()))
            .unwrap_or_else(|| ".".into());
        if path.is_dir() {
            format!("{}/", root_name.to_string_lossy())
        } else {
            root_name.to_string_lossy().into_owned()
        }
    } else {
        let name = path.file_name().unwrap_or_default().to_string_lossy();
        if path.is_dir() {
            format!("{name}/")
        } else {
            name.into_owned()
        }
    };

    if !long {
        return base;
    }

    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                format!("{base} [dir]")
            } else if metadata.file_type().is_symlink() {
                format!("{base} [link]")
            } else {
                format!("{base} {}", human_size(metadata.len()))
            }
        }
        Err(_) => format!("{base} [unknown]"),
    }
}

fn human_size(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1}M", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else {
        format!("{bytes}B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hidden_files_are_filtered_by_default() {
        assert!(!should_show(Path::new(".gitignore"), false));
        assert!(should_show(Path::new(".gitignore"), true));
        assert!(should_show(Path::new("src"), false));
    }
}
