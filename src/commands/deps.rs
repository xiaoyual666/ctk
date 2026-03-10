use std::fs;
use std::path::{Path, PathBuf};

use toml::Value as TomlValue;

pub fn handle(path: String) -> i32 {
    let root = PathBuf::from(&path);
    let dir = if root.is_file() {
        root.parent().unwrap_or(Path::new(".")).to_path_buf()
    } else {
        root
    };

    if !dir.exists() {
        eprintln!("ctk deps: path not found: {}", dir.display());
        return 2;
    }

    let mut sections = Vec::new();

    if let Some(section) = summarize_cargo(&dir.join("Cargo.toml")) {
        sections.push(section);
    }
    if let Some(section) = summarize_package_json(&dir.join("package.json")) {
        sections.push(section);
    }
    if let Some(section) = summarize_requirements(&dir.join("requirements.txt")) {
        sections.push(section);
    }
    if let Some(section) = summarize_pyproject(&dir.join("pyproject.toml")) {
        sections.push(section);
    }
    if let Some(section) = summarize_go_mod(&dir.join("go.mod")) {
        sections.push(section);
    }

    if sections.is_empty() {
        println!("no dependency files found in {}", dir.display());
        return 1;
    }

    println!("{}", sections.join("\n\n"));
    0
}

fn summarize_cargo(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let value = toml::from_str::<TomlValue>(&content).ok()?;
    let package_name = value
        .get("package")
        .and_then(|pkg| pkg.get("name"))
        .and_then(TomlValue::as_str)
        .unwrap_or("cargo-project");
    let version = value
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(TomlValue::as_str)
        .unwrap_or("?");

    let mut out = vec![format!("Rust: {package_name} @ {version}")];
    render_toml_dep_table(&mut out, "dependencies", value.get("dependencies"));
    render_toml_dep_table(&mut out, "dev-dependencies", value.get("dev-dependencies"));
    Some(out.join("\n"))
}

fn summarize_package_json(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let json = serde_json::from_str::<serde_json::Value>(&content).ok()?;
    let name = json
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or("node-project");
    let version = json
        .get("version")
        .and_then(|value| value.as_str())
        .unwrap_or("?");
    let mut out = vec![format!("Node: {name} @ {version}")];
    render_json_dep_object(&mut out, "dependencies", json.get("dependencies"));
    render_json_dep_object(&mut out, "devDependencies", json.get("devDependencies"));
    Some(out.join("\n"))
}

fn summarize_requirements(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let deps = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect::<Vec<_>>();
    let mut out = vec![format!("Python requirements.txt: {} packages", deps.len())];
    out.extend(render_name_list(&deps, 12));
    Some(out.join("\n"))
}

fn summarize_pyproject(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let value = toml::from_str::<TomlValue>(&content).ok()?;
    let project_name = value
        .get("project")
        .and_then(|project| project.get("name"))
        .and_then(TomlValue::as_str)
        .or_else(|| {
            value
                .get("tool")
                .and_then(|tool| tool.get("poetry"))
                .and_then(|poetry| poetry.get("name"))
                .and_then(TomlValue::as_str)
        })
        .unwrap_or("python-project");
    let version = value
        .get("project")
        .and_then(|project| project.get("version"))
        .and_then(TomlValue::as_str)
        .or_else(|| {
            value
                .get("tool")
                .and_then(|tool| tool.get("poetry"))
                .and_then(|poetry| poetry.get("version"))
                .and_then(TomlValue::as_str)
        })
        .unwrap_or("?");

    let mut out = vec![format!("Python pyproject: {project_name} @ {version}")];

    let project_deps = value
        .get("project")
        .and_then(|project| project.get("dependencies"))
        .and_then(TomlValue::as_array)
        .map(|deps| {
            deps.iter()
                .filter_map(TomlValue::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !project_deps.is_empty() {
        out.push(format!("  dependencies ({})", project_deps.len()));
        out.extend(render_name_list(&project_deps, 12));
    }

    let poetry_deps = value
        .get("tool")
        .and_then(|tool| tool.get("poetry"))
        .and_then(|poetry| poetry.get("dependencies"))
        .and_then(TomlValue::as_table)
        .map(|table| {
            table
                .iter()
                .filter(|(name, _)| name.as_str() != "python")
                .map(|(name, spec)| format!("{name} {}", short_toml_dep_spec(spec)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !poetry_deps.is_empty() {
        out.push(format!("  poetry dependencies ({})", poetry_deps.len()));
        out.extend(render_name_list(&poetry_deps, 12));
    }

    Some(out.join("\n"))
}

fn summarize_go_mod(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    let mut deps = Vec::new();
    let mut in_require_block = false;

    for line in content.lines().map(str::trim) {
        if line == "require (" {
            in_require_block = true;
            continue;
        }
        if in_require_block && line == ")" {
            in_require_block = false;
            continue;
        }
        if in_require_block {
            if !line.is_empty() {
                deps.push(line.to_string());
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("require ") {
            deps.push(rest.to_string());
        }
    }

    let mut out = vec![format!("Go: {} dependencies", deps.len())];
    out.extend(render_name_list(&deps, 12));
    Some(out.join("\n"))
}

fn render_toml_dep_table(out: &mut Vec<String>, label: &str, value: Option<&TomlValue>) {
    let Some(table) = value.and_then(TomlValue::as_table) else {
        return;
    };
    let deps = table
        .iter()
        .map(|(name, spec)| format!("{name} {}", short_toml_dep_spec(spec)))
        .collect::<Vec<_>>();
    out.push(format!("  {label} ({})", deps.len()));
    out.extend(render_name_list(&deps, 12));
}

fn render_json_dep_object(out: &mut Vec<String>, label: &str, value: Option<&serde_json::Value>) {
    let Some(table) = value.and_then(serde_json::Value::as_object) else {
        return;
    };
    let deps = table
        .iter()
        .map(|(name, spec)| format!("{name} {}", spec.as_str().unwrap_or("*")))
        .collect::<Vec<_>>();
    out.push(format!("  {label} ({})", deps.len()));
    out.extend(render_name_list(&deps, 12));
}

fn render_name_list(items: &[String], limit: usize) -> Vec<String> {
    let mut out = items
        .iter()
        .take(limit)
        .map(|item| format!("    {item}"))
        .collect::<Vec<_>>();
    if items.len() > limit {
        out.push(format!("    ... {} more", items.len() - limit));
    }
    out
}

fn short_toml_dep_spec(value: &TomlValue) -> String {
    match value {
        TomlValue::String(version) => version.clone(),
        TomlValue::Table(table) => table
            .get("version")
            .and_then(TomlValue::as_str)
            .or_else(|| table.get("path").and_then(TomlValue::as_str))
            .or_else(|| table.get("git").and_then(TomlValue::as_str))
            .unwrap_or("*")
            .to_string(),
        _ => "*".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_toml_dep_spec_prefers_version() {
        let value = toml::from_str::<TomlValue>("dep = { version = \"1\", path = \"../dep\" }")
            .expect("toml");
        let dep = value.get("dep").expect("dep");
        assert_eq!(short_toml_dep_spec(dep), "1");
    }
}
