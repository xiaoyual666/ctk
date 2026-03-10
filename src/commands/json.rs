use serde_json::Value;
use std::fs;
use std::path::PathBuf;

pub fn handle(path: String) -> i32 {
    let file = PathBuf::from(&path);
    if !file.exists() {
        eprintln!("ctk json: file not found: {}", file.display());
        return 2;
    }

    let text = match fs::read_to_string(&file) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("ctk json: failed to read {}: {error}", file.display());
            return 1;
        }
    };

    let value = match serde_json::from_str::<Value>(&text) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("ctk json: invalid JSON: {error}");
            return 1;
        }
    };

    let mut lines = Vec::new();
    let mut budget = 200usize;
    render("root", &value, 0, 6, &mut budget, &mut lines);
    println!("{}", lines.join("\n"));
    0
}

fn render(
    name: &str,
    value: &Value,
    depth: usize,
    max_depth: usize,
    budget: &mut usize,
    lines: &mut Vec<String>,
) {
    if *budget == 0 {
        lines.push(format!(
            "{}... node budget exhausted ...",
            "  ".repeat(depth)
        ));
        return;
    }
    *budget -= 1;

    let indent = "  ".repeat(depth);
    match value {
        Value::Object(map) => {
            lines.push(format!("{indent}{name}: object<{}>", map.len()));
            if depth >= max_depth {
                return;
            }
            for (key, child) in map.iter().take(20) {
                render(key, child, depth + 1, max_depth, budget, lines);
            }
            if map.len() > 20 {
                lines.push(format!(
                    "{indent}  ... {} more keys omitted ...",
                    map.len() - 20
                ));
            }
        }
        Value::Array(items) => {
            lines.push(format!("{indent}{name}: array<{}>", items.len()));
            if depth < max_depth {
                if let Some(first) = items.first() {
                    render("[0]", first, depth + 1, max_depth, budget, lines);
                }
            }
        }
        Value::Null => lines.push(format!("{indent}{name}: null")),
        Value::Bool(_) => lines.push(format!("{indent}{name}: bool")),
        Value::Number(_) => lines.push(format!("{indent}{name}: number")),
        Value::String(_) => lines.push(format!("{indent}{name}: string")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_nested_object_shape() {
        let value = serde_json::json!({ "user": { "id": 1, "tags": ["a"] } });
        let mut lines = Vec::new();
        let mut budget = 200usize;
        render("root", &value, 0, 6, &mut budget, &mut lines);
        assert!(lines.iter().any(|line| line.contains("user: object<2>")));
        assert!(lines.iter().any(|line| line.contains("tags: array<1>")));
    }
}
