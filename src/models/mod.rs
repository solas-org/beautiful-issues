pub(crate) mod ticket;
pub(crate) mod crash;
pub(crate) mod issue;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Simple Title"), "Simple Title");
        assert_eq!(sanitize_filename("First Line\nSecond Line"), "First Line");
        assert_eq!(sanitize_filename("Bad: <file> /path\\name?*"), "Bad_ _file_ _path_name__");
        assert_eq!(sanitize_filename(""), "unnamed");
        assert_eq!(sanitize_filename("\n\r"), "unnamed");
    }
}