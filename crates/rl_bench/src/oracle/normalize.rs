pub fn normalize_lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

pub fn normalize_paths(lines: Vec<String>) -> Vec<String> {
    lines
        .into_iter()
        .map(|line| line.replace('\\', "/"))
        .collect()
}

pub fn sort_stable(mut lines: Vec<String>) -> Vec<String> {
    lines.sort();
    lines
}
