use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub(crate) struct SnippetRef {
    pub(crate) path: PathBuf,
    pub(crate) range: Option<(usize, usize)>,
}

impl SnippetRef {
    pub(crate) fn extract_from_text(text: &str) -> Vec<SnippetRef> {
        let mut results = Vec::new();
        for word in text.split_whitespace() {
            if let Some(rest) = word.strip_prefix('@') {
                let cleaned = rest.trim_matches(|c: char| c == ',' || c == ';' || c == ')' || c == '(');
                if cleaned.is_empty() {
                    continue;
                }

                if let Some(bracket_start) = cleaned.find('[')
                    && let Some(bracket_end) = cleaned.find(']')
                {
                    let path_str = &cleaned[..bracket_start];
                    let range_str = &cleaned[bracket_start + 1..bracket_end];

                    let range = if let Some((s_str, e_str)) = range_str.split_once("..") {
                        let start = s_str.trim().parse::<usize>().ok();
                        let end = e_str.trim().parse::<usize>().ok();
                        match (start, end) {
                            (Some(s), Some(e)) => Some((s, e)),
                            (Some(s), None) => Some((s, s + 15)),
                            (None, Some(e)) => Some((1, e)),
                            _ => None,
                        }
                    } else {
                        None
                    };

                    results.push(SnippetRef {
                        path: PathBuf::from(path_str),
                        range,
                    });
                    continue;
                }

                results.push(SnippetRef {
                    path: PathBuf::from(cleaned),
                    range: None,
                });
            }
        }
        results
    }
}

pub(crate) fn load_file_snippet<'a>(
    path: &Path,
    target_line: Option<usize>,
    range: Option<(usize, usize)>,
    select_symbol: &str,
    highlight_color: Color,
    text_color: Color,
    subtext_color: Color,
) -> Vec<Line<'a>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            return vec![Line::from(vec![
                Span::styled(
                    format!("Cannot read file: {}", path.display()),
                    Style::default().fg(Color::Yellow),
                )
            ])];
        }
    };

    let all_lines: Vec<&str> = content.lines().collect();
    if all_lines.is_empty() {
        return vec![Line::from(vec![Span::styled(
            "File is empty",
            Style::default().fg(subtext_color),
        )])];
    }

    let (start_idx, end_idx) = match (range, target_line) {
        (Some((start, end)), _) => {
            let s = start.saturating_sub(1).min(all_lines.len());
            let e = end.min(all_lines.len());
            (s, e.max(s + 1))
        }
        (None, Some(target)) => {
            let t = target.saturating_sub(1);
            let s = t.saturating_sub(5);
            let e = (t + 6).min(all_lines.len());
            (s, e)
        }
        (None, None) => {
            let e = all_lines.len().min(30);
            (0, e)
        }
    };

    let mut lines = Vec::new();
    for idx in start_idx..end_idx {
        if idx >= all_lines.len() {
            break;
        }
        let line_num = idx + 1;
        let is_target = target_line.map(|t| t == line_num).unwrap_or(false);

        let num_span = Span::styled(
            format!("{line_num:4} │ "),
            if is_target {
                Style::default().fg(highlight_color).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(subtext_color)
            },
        );

        let content_span = Span::styled(
            all_lines[idx].to_string(),
            if is_target {
                Style::default()
                    .fg(highlight_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(text_color)
            },
        );

        let marker = if is_target {
            Span::styled(
                format!(" {select_symbol} "),
                Style::default().fg(highlight_color).add_modifier(Modifier::BOLD),
            )
        } else {
            let pad = " ".repeat(select_symbol.chars().count() + 2);
            Span::raw(pad)
        };

        lines.push(Line::from(vec![marker, num_span, content_span]));
    }

    lines
}
