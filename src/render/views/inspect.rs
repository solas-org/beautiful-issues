use crate::lock::Lock;
use crate::models::crash::CrashFile;
use crate::render::keys::normalize_char;
use crate::render::snippet::load_file_snippet;
use crate::render::theme::Theme;
use anyhow::{bail, Context, Result};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Spacing};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::fs::read;
use std::path::{Path, PathBuf};

pub(crate) enum InspectMsg {
    NextDiagnostic,
    PrevDiagnostic,
    ToggleTrace,
    CreateTicket,
    Quit,
}

pub(crate) enum InspectEffect {
    None,
    Quit,
    CreateTicketForCrash(PathBuf),
}

pub(crate) struct InspectModel {
    pub(crate) crash_path: PathBuf,
    pub(crate) crash: CrashFile,
    pub(crate) selected_index: usize,
    pub(crate) show_trace: bool,
    pub(crate) trace_scroll: usize,
}

impl InspectModel {
    pub(crate) fn load(custom_path: Option<&Path>) -> Result<Self> {
        let path = if let Some(p) = custom_path {
            p.to_path_buf()
        } else {
            let lock = Lock::load();
            if lock.last_crash.as_os_str().is_empty() || !lock.last_crash.exists() {
                bail!("No crash file found. Run a crash or provide a file path.");
            }
            lock.last_crash
        };

        let bytes = read(&path)
            .with_context(|| format!("Cannot read crash file at {}", path.display()))?;
        let crash: CrashFile = serde_json::from_slice(&bytes)
            .context("Cannot parse crash file JSON")?;

        let selected_index = if !crash.crashes.is_empty() {
            crash.crashes.len() - 1
        } else {
            0
        };

        Ok(Self {
            crash_path: path,
            crash,
            selected_index,
            show_trace: false,
            trace_scroll: 0,
        })
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<InspectMsg> {
        match key.code {
            KeyCode::Esc => Some(InspectMsg::Quit),
            KeyCode::Down => Some(InspectMsg::NextDiagnostic),
            KeyCode::Up => Some(InspectMsg::PrevDiagnostic),
            KeyCode::Char(c) => match normalize_char(c) {
                'j' => Some(InspectMsg::NextDiagnostic),
                'k' => Some(InspectMsg::PrevDiagnostic),
                't' => Some(InspectMsg::ToggleTrace),
                'c' => Some(InspectMsg::CreateTicket),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn update(&mut self, msg: InspectMsg) -> InspectEffect {
        match msg {
            InspectMsg::Quit => InspectEffect::Quit,
            InspectMsg::NextDiagnostic => {
                if !self.crash.crashes.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.crash.crashes.len() - 1);
                    self.trace_scroll = 0;
                }
                InspectEffect::None
            }
            InspectMsg::PrevDiagnostic => {
                self.selected_index = self.selected_index.saturating_sub(1);
                self.trace_scroll = 0;
                InspectEffect::None
            }
            InspectMsg::ToggleTrace => {
                self.show_trace = !self.show_trace;
                InspectEffect::None
            }
            InspectMsg::CreateTicket => {
                InspectEffect::CreateTicketForCrash(self.crash_path.clone())
            }
        }
    }

    pub(crate) fn view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let main_vert = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),
                Constraint::Length(3),
            ])
            .split(area);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .spacing(Spacing::Overlap(1))
            .constraints([
                Constraint::Length(3),
                Constraint::Min(7),
            ])
            .split(main_vert[0]);

        let diag_count = self.crash.crashes.len();
        let current_diag_num = if diag_count > 0 { self.selected_index + 1 } else { 0 };
        let crash_filename = self.crash_path.file_name().unwrap_or_default().to_string_lossy();
        let header_title = format!(" CRASH INSPECTOR: {crash_filename} ");

        let issue_str = format!("ISSUE {current_diag_num}/{diag_count}");
        let header_text = Line::from(vec![
            theme.badge(&issue_str, theme.warn, Color::Black),
            Span::raw("  "),
            theme.badge(&self.crash.context.os, theme.info, Color::Black),
            Span::raw("  "),
            theme.badge(&self.crash.context.target_framework, theme.ok, Color::Black),
        ]);
        let header_block = Paragraph::new(header_text)
            .alignment(Alignment::Center)
            .block(theme.block(&header_title));
        frame.render_widget(header_block, chunks[0]);

        if self.crash.crashes.is_empty() {
            let empty_msg = Paragraph::new("No diagnostics recorded in this crash file.")
                .alignment(Alignment::Center)
                .block(theme.block("Empty Crash"));
            frame.render_widget(empty_msg, chunks[1]);
        } else {
            let current_diag = &self.crash.crashes[self.selected_index];

            let content_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .spacing(Spacing::Overlap(1))
                .constraints([
                    Constraint::Percentage(62),
                    Constraint::Fill(1),
                ])
                .split(chunks[1]);

            let snippet_title = format!(
                " Source: {}:{} ",
                current_diag.file_path.display(),
                current_diag.span.line
            );
            let snippet_lines = load_file_snippet(
                &current_diag.file_path,
                Some(current_diag.span.line),
                None,
                &theme.select_symbol,
                theme.err,
                theme.text,
                theme.subtext,
            );
            let snippet_widget = Paragraph::new(snippet_lines).block(theme.block(&snippet_title));
            frame.render_widget(snippet_widget, content_chunks[0]);

            let detail_lines = vec![
                Line::from(vec![
                    theme.badge(&current_diag.error_code, theme.warn, Color::Black),
                    Span::raw(" "),
                    Span::styled(
                        format!("{}:{}", current_diag.span.line, current_diag.span.column),
                        Style::default().fg(theme.info).add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled("Error Description:", Style::default().fg(theme.subtext))),
                Line::from(Span::styled(
                    &current_diag.message,
                    Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Trace Frames: ", Style::default().fg(theme.subtext)),
                    Span::styled(
                        format!("{} available (press 't' to expand)", current_diag.trace.len()),
                        Style::default().fg(theme.info),
                    ),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("File Location: ", Style::default().fg(theme.subtext)),
                    Span::styled(current_diag.file_path.display().to_string(), Style::default().fg(theme.text)),
                ]),
            ];

            let detail_widget = Paragraph::new(detail_lines)
                .wrap(Wrap { trim: true })
                .block(theme.block(" Diagnostic Details "));
            frame.render_widget(detail_widget, content_chunks[1]);
        }

        let footer_shortcuts = [
            ("↑/k", "Prev"),
            ("↓/j", "Next"),
            ("t", if self.show_trace { "Hide Trace" } else { "Show Trace" }),
            ("c", "Create Ticket"),
            ("Esc", "Quit"),
        ];
        let footer_block = Paragraph::new(theme.shortcut_bar(&footer_shortcuts))
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(footer_block, main_vert[1]);

        if self.show_trace && !self.crash.crashes.is_empty() {
            let current_diag = &self.crash.crashes[self.selected_index];
            let popup_area = centered_rect(70, 65, area);
            frame.render_widget(Clear, popup_area);

            let trace_items: Vec<ListItem> = if current_diag.trace.is_empty() {
                vec![ListItem::new("No stack trace frames recorded for this diagnostic.")]
            } else {
                current_diag
                    .trace
                    .iter()
                    .enumerate()
                    .map(|(idx, frame_str)| {
                        ListItem::new(vec![
                            Line::from(vec![
                                Span::styled(format!("{idx:2}. "), Style::default().fg(theme.warn).add_modifier(Modifier::BOLD)),
                                Span::styled(frame_str, Style::default().fg(theme.text)),
                            ]),
                        ])
                    })
                    .collect()
            };

            let trace_title = format!(" Call Stack Trace ({} frames) ", current_diag.trace.len());
            let trace_list = List::new(trace_items).block(
                theme.focused_block(&trace_title)
            );
            frame.render_widget(trace_list, popup_area);
        }
    }
}

pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
