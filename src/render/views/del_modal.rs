use crate::render::keys::normalize_char;
use crate::render::theme::Theme;
use crate::render::views::inspect::centered_rect;
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Alignment;
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::fs::remove_file;
use std::path::PathBuf;

pub(crate) enum DelModalMsg {
    Confirm,
    Cancel,
}

pub(crate) enum DelModalEffect {
    Confirmed,
    Cancelled,
}

pub(crate) struct DelModalModel {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) title: String,
}

impl DelModalModel {
    pub(crate) fn new(files: Vec<PathBuf>, title: impl Into<String>) -> Self {
        Self {
            files,
            title: title.into(),
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<DelModalMsg> {
        match key.code {
            KeyCode::Enter => Some(DelModalMsg::Confirm),
            KeyCode::Esc => Some(DelModalMsg::Cancel),
            KeyCode::Char(c) => match normalize_char(c) {
                'y' => Some(DelModalMsg::Confirm),
                'n' | 'q' => Some(DelModalMsg::Cancel),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn update(&mut self, msg: DelModalMsg) -> DelModalEffect {
        match msg {
            DelModalMsg::Confirm => {
                for file in &self.files {
                    let _ = remove_file(file);
                }
                DelModalEffect::Confirmed
            }
            DelModalMsg::Cancel => DelModalEffect::Cancelled,
        }
    }

    pub(crate) fn view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        frame.render_widget(Clear, area);
        let backdrop = Block::default();
        frame.render_widget(backdrop, area);

        let popup_area = centered_rect(64, 55, area);
        let modal_title = format!(" Confirm Delete: {} ", self.title);
        let modal_block = theme.alert_block(&modal_title);
        let inner = modal_block.inner(popup_area);
        frame.render_widget(modal_block, popup_area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(4),
                Constraint::Length(2),
            ])
            .split(inner);

        let count = self.files.len();
        let warn_text = Line::from(vec![
            theme.badge("DANGER", theme.err, Color::White),
            Span::raw(" "),
            Span::styled(
                format!("Permanently delete {count} file(s)? This cannot be undone."),
                Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
            ),
        ]);
        let warn_widget = Paragraph::new(vec![Line::raw(""), warn_text]).alignment(Alignment::Center);
        frame.render_widget(warn_widget, chunks[0]);

        let file_items: Vec<ListItem> = self
            .files
            .iter()
            .map(|p| {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                ListItem::new(Line::from(vec![
                    Span::styled("  ✕ ", Style::default().fg(theme.err).add_modifier(Modifier::BOLD)),
                    Span::styled(name, Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
                    Span::styled(format!(" ({})", p.display()), Style::default().fg(theme.subtext)),
                ]))
            })
            .collect();

        let list = List::new(file_items);
        frame.render_widget(list, chunks[1]);

        let button_line = Line::from(vec![
            Span::styled(" [y / Enter] Confirm Delete ", Style::default().bg(theme.err).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::raw("    "),
            Span::styled(" [n / Esc] Cancel ", Style::default().bg(theme.border).fg(theme.text)),
        ]);
        let buttons = Paragraph::new(button_line).alignment(Alignment::Center);
        frame.render_widget(buttons, chunks[2]);
    }
}
