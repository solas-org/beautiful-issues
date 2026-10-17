use crate::render::views::ticket_wizard::parse_query_path_range;
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
                let (p, r) = parse_query_path_range(rest);

                results.push(SnippetRef {
                    path: PathBuf::from(p),
                    range: r,
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
            return vec![Line::from(vec![Span::styled(
                format!("Cannot read file: {}", path.display()),
                Style::default().fg(Color::Yellow),
            )])];
        }
    };

    let all_lines: Vec<&str> = content.lines().collect();
    if all_lines.is_empty() {
        return vec![Line::from(vec![Span::styled(
            "File is empty",
            Style::default().fg(subtext_color),
        )])];
    }

    let (start_idx, end_idx) = match target_line {
        Some(target) => {
            let t = target.saturating_sub(1);
            let s = t.saturating_sub(5);
            let e = (t + 6).min(all_lines.len());
            (s, e)
        }
        None => {
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

        let marker = if is_target {
            Span::styled(
                format!(" {select_symbol} "),
                Style::default()
                    .fg(highlight_color)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            let pad = " ".repeat(select_symbol.chars().count() + 2);
            Span::raw(pad)
        };

        let num_span = Span::styled(
            format!("{line_num:4} │ "),
            if is_target {
                Style::default()
                    .fg(highlight_color)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(subtext_color)
            },
        );

        let normal_content_style = Style::default().fg(text_color);
        let highlight_content_style = Style::default()
            .fg(highlight_color)
            .add_modifier(Modifier::BOLD);

        let mut content_spans = Vec::new();
        let line_str = &all_lines[idx];

        if is_target {
            if let Some((start, end)) = range {
                let char_indices: Vec<usize> =
                    line_str.char_indices().map(|(b_idx, _)| b_idx).collect();
                let len_chars = char_indices.len();

                if start <= end && start <= len_chars && end <= len_chars {
                    let byte_start = if start == len_chars {
                        line_str.len()
                    } else {
                        char_indices[start]
                    };
                    let byte_end = if end == len_chars {
                        line_str.len()
                    } else {
                        char_indices[end]
                    };

                    let before = &line_str[..byte_start];
                    let highlighted = &line_str[byte_start..byte_end];
                    let after = &line_str[byte_end..];

                    if !before.is_empty() {
                        content_spans.push(Span::styled(before.to_string(), normal_content_style));
                    }
                    if !highlighted.is_empty() {
                        content_spans.push(Span::styled(
                            highlighted.to_string(),
                            highlight_content_style,
                        ));
                    }
                    if !after.is_empty() {
                        content_spans.push(Span::styled(after.to_string(), normal_content_style));
                    }
                } else {
                    content_spans.push(Span::styled(line_str.to_string(), highlight_content_style));
                }
            } else {
                content_spans.push(Span::styled(line_str.to_string(), highlight_content_style));
            }
        } else {
            content_spans.push(Span::styled(line_str.to_string(), normal_content_style));
        }

        let mut all_spans = vec![marker, num_span];
        all_spans.extend(content_spans);

        lines.push(Line::from(all_spans));
    }

    lines
}
