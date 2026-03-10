use std::collections::HashSet;

pub fn tail_lines(text: &str, limit: usize) -> Vec<String> {
    let lines: Vec<String> = text
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();
    if lines.len() <= limit {
        return lines;
    }
    lines[lines.len() - limit..].to_vec()
}

pub fn head_tail(lines: &[String], limit: usize) -> Vec<String> {
    if limit == 0 {
        return Vec::new();
    }
    if lines.len() <= limit {
        return lines.to_vec();
    }
    let head_count = ((limit as f32) * 0.7).ceil() as usize;
    let head_count = head_count.clamp(1, limit.saturating_sub(1));
    let tail_count = limit.saturating_sub(head_count).max(1);
    let omitted = lines.len().saturating_sub(head_count + tail_count);
    let mut result = Vec::with_capacity(limit + 1);
    result.extend(lines[..head_count].iter().cloned());
    result.push(format!("... {omitted} lines omitted ..."));
    result.extend(lines[lines.len() - tail_count..].iter().cloned());
    result
}

pub fn compact_block(text: &str, limit: usize) -> String {
    if limit == 0 {
        return String::new();
    }
    let lines: Vec<String> = text
        .lines()
        .map(|line| line.trim_end().to_string())
        .collect();
    head_tail(&lines, limit).join("\n")
}

pub fn unique_preserve_order(lines: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for line in lines {
        if seen.insert(line.clone()) {
            result.push(line);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn head_tail_inserts_omission_line() {
        let lines = (1..=10).map(|n| format!("line {n}")).collect::<Vec<_>>();
        let compact = head_tail(&lines, 4);
        assert!(compact.iter().any(|line| line.contains("omitted")));
    }

    #[test]
    fn head_tail_zero_limit_is_empty() {
        let lines = vec!["line 1".to_string()];
        assert!(head_tail(&lines, 0).is_empty());
    }
}
