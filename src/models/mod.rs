pub(crate) mod crash;
pub(crate) mod issue;
pub(crate) mod package;
pub(crate) mod ticket;

pub(crate) fn sanitize_filename(name: &str) -> String {
    let first_line = name.lines().next().unwrap_or("").trim();
    let sanitized: String = first_line
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0'..='\x1f' => '_',
            _ => c,
        })
        .collect();
    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    }
}
