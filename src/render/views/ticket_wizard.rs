use crate::commands::list::paths_by_extension;
use crate::configs::find_bissue_dir;
use crate::lock::Lock;
use crate::models::crash::CrashFile;
use crate::models::ticket::Ticket;
use crate::render::keys::normalize_char;
use crate::render::snippet::{load_file_snippet, SnippetRef};
use crate::render::theme::Theme;
use crate::render::views::wizard::{
    render_active_text_step, render_wizard_footer, render_wizard_header, WizardHeader, WizardLayout,
};
use chrono::{DateTime, Utc};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Spacing};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::fs::metadata;
use std::path::PathBuf;

pub(crate) enum TicketWizardMsg {
    Char(char),
    Backspace,
    Enter,
    NextStep,
    PrevStep,
    Up,
    Down,
    ToggleSelection,
    ToggleAll,
    Quit,
}

pub(crate) enum TicketWizardEffect {
    None,
    Quit,
    Saved(#[allow(dead_code)] PathBuf),
}

pub(crate) struct TicketWizardModel {
    pub(crate) active_step: usize,
    pub(crate) header: String,
    pub(crate) description: String,
    pub(crate) crash_options: Vec<Option<PathBuf>>,
    pub(crate) crash_cursor: usize,
    pub(crate) crash_scroll: usize,
    pub(crate) loaded_crash: Option<CrashFile>,
    pub(crate) selected_diagnostics: Vec<bool>,
    pub(crate) diag_cursor: usize,
    pub(crate) diag_scroll: usize,
    pub(crate) status: Option<String>,
}

impl TicketWizardModel {
    pub(crate) fn new(preset_crash: Option<PathBuf>) -> Self {
        let mut crash_options: Vec<Option<PathBuf>> = vec![None];
        let crashes_dir = find_bissue_dir().join("crashes");
        let mut found = paths_by_extension(&crashes_dir, "crash").unwrap_or_default();

        found.sort_by(|a, b| {
            let ma = metadata(a).and_then(|m| m.modified()).ok();
            let mb = metadata(b).and_then(|m| m.modified()).ok();
            mb.cmp(&ma)
        });

        for p in found {
            crash_options.push(Some(p));
        }

        let mut initial_cursor = 0;
        let last_crash = Lock::load().last_crash;
        let target_crash = preset_crash.or_else(|| {
            if last_crash.exists() {
                Some(last_crash)
            } else {
                None
            }
        });

        if let Some(target) = target_crash {
            let canon_target = dunce::canonicalize(&target).unwrap_or(target.clone());
            let found_idx = crash_options.iter().position(|opt| {
                if let Some(p) = opt {
                    dunce::canonicalize(p).unwrap_or_else(|_| p.clone()) == canon_target
                } else {
                    false
                }
            });
            if let Some(idx) = found_idx {
                initial_cursor = idx;
            } else {
                crash_options.insert(1, Some(target));
                initial_cursor = 1;
            }
        }

        Self {
            active_step: 0,
            header: String::new(),
            description: String::new(),
            crash_options,
            crash_cursor: initial_cursor,
            crash_scroll: 0,
            loaded_crash: None,
            selected_diagnostics: Vec::new(),
            diag_cursor: 0,
            diag_scroll: 0,
            status: None,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<TicketWizardMsg> {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && let KeyCode::Char(c) = key.code
            && normalize_char(c) == 's'
        {
            return Some(TicketWizardMsg::NextStep);
        }

        match key.code {
            KeyCode::Esc => Some(TicketWizardMsg::Quit),
            KeyCode::Tab => Some(TicketWizardMsg::NextStep),
            KeyCode::BackTab => Some(TicketWizardMsg::PrevStep),
            KeyCode::Enter => Some(TicketWizardMsg::Enter),
            KeyCode::Backspace => Some(TicketWizardMsg::Backspace),
            KeyCode::Up => Some(TicketWizardMsg::Up),
            KeyCode::Down => Some(TicketWizardMsg::Down),
            KeyCode::Char(' ') => {
                if self.active_step == 3 {
                    Some(TicketWizardMsg::ToggleSelection)
                } else {
                    Some(TicketWizardMsg::Char(' '))
                }
            }
            KeyCode::Char(c) => {
                let norm = normalize_char(c);
                match self.active_step {
                    2 => match norm {
                        'k' => Some(TicketWizardMsg::Up),
                        'j' => Some(TicketWizardMsg::Down),
                        _ => None,
                    },
                    3 => match norm {
                        'k' => Some(TicketWizardMsg::Up),
                        'j' => Some(TicketWizardMsg::Down),
                        'a' => Some(TicketWizardMsg::ToggleAll),
                        _ => None,
                    },
                    _ => Some(TicketWizardMsg::Char(c)),
                }
            }
            _ => None,
        }
    }

    pub(crate) fn update(&mut self, msg: TicketWizardMsg) -> TicketWizardEffect {
        match msg {
            TicketWizardMsg::Quit => TicketWizardEffect::Quit,
            TicketWizardMsg::Char(c) => {
                match self.active_step {
                    0 => self.header.push(c),
                    1 => self.description.push(c),
                    _ => {}
                }
                TicketWizardEffect::None
            }
            TicketWizardMsg::Backspace => {
                match self.active_step {
                    0 => { self.header.pop(); }
                    1 => { self.description.pop(); }
                    2 => { self.active_step = 1; }
                    3 => { self.active_step = 2; }
                    _ => {}
                }
                TicketWizardEffect::None
            }
            TicketWizardMsg::PrevStep => {
                self.active_step = self.active_step.saturating_sub(1);
                TicketWizardEffect::None
            }
            TicketWizardMsg::Up => {
                if self.active_step == 2 {
                    self.crash_cursor = self.crash_cursor.saturating_sub(1);
                } else if self.active_step == 3 {
                    self.diag_cursor = self.diag_cursor.saturating_sub(1);
                }
                TicketWizardEffect::None
            }
            TicketWizardMsg::Down => {
                if self.active_step == 2 {
                    if !self.crash_options.is_empty() {
                        self.crash_cursor = (self.crash_cursor + 1).min(self.crash_options.len() - 1);
                    }
                } else if self.active_step == 3
                    && let Some(crash) = &self.loaded_crash
                    && !crash.crashes.is_empty()
                {
                    self.diag_cursor = (self.diag_cursor + 1).min(crash.crashes.len() - 1);
                }
                TicketWizardEffect::None
            }
            TicketWizardMsg::ToggleSelection => {
                if self.active_step == 3 && self.diag_cursor < self.selected_diagnostics.len() {
                    self.selected_diagnostics[self.diag_cursor] = !self.selected_diagnostics[self.diag_cursor];
                }
                TicketWizardEffect::None
            }
            TicketWizardMsg::ToggleAll => {
                if self.active_step == 3 && !self.selected_diagnostics.is_empty() {
                    let any_unselected = self.selected_diagnostics.iter().any(|&s| !s);
                    self.selected_diagnostics.fill(any_unselected);
                }
                TicketWizardEffect::None
            }
            TicketWizardMsg::NextStep | TicketWizardMsg::Enter => {
                match self.active_step {
                    0 => {
                        if !self.header.trim().is_empty() {
                            self.active_step = 1;
                        }
                        TicketWizardEffect::None
                    }
                    1 => {
                        if !self.description.trim().is_empty() {
                            self.active_step = 2;
                        }
                        TicketWizardEffect::None
                    }
                    2 => {
                        self.load_selected_crash();
                        self.active_step = 3;
                        TicketWizardEffect::None
                    }
                    3 => self.save_ticket(),
                    _ => TicketWizardEffect::None,
                }
            }
        }
    }

    fn load_selected_crash(&mut self) {
        if let Some(Some(path)) = self.crash_options.get(self.crash_cursor) {
            match std::fs::read(path) {
                Ok(bytes) => match serde_json::from_slice::<CrashFile>(&bytes) {
                    Ok(crash_file) => {
                        let count = crash_file.crashes.len();
                        self.selected_diagnostics = vec![true; count];
                        self.diag_cursor = 0;
                        self.diag_scroll = 0;
                        self.loaded_crash = Some(crash_file);
                        self.status = None;
                    }
                    Err(e) => {
                        self.status = Some(format!("Error parsing crash JSON: {e}"));
                        self.loaded_crash = None;
                        self.selected_diagnostics.clear();
                    }
                },
                Err(e) => {
                    self.status = Some(format!("Error reading crash file: {e}"));
                    self.loaded_crash = None;
                    self.selected_diagnostics.clear();
                }
            }
        } else {
            self.loaded_crash = None;
            self.selected_diagnostics.clear();
            self.status = None;
        }
    }

    fn save_ticket(&mut self) -> TicketWizardEffect {
        let crash_path = self.crash_options.get(self.crash_cursor).and_then(|opt| opt.clone());
        let diag_indices: Vec<u32> = self.selected_diagnostics
            .iter()
            .enumerate()
            .filter_map(|(idx, &selected)| if selected { Some(idx as u32) } else { None })
            .collect();

        let ticket = Ticket {
            header: self.header.trim().to_string(),
            description: self.description.clone(),
            crash_file: crash_path,
            diagnostics: diag_indices,
        };

        match ticket.save(None) {
            Ok(path) => TicketWizardEffect::Saved(path),
            Err(e) => {
                self.status = Some(format!("Error saving ticket: {e}"));
                TicketWizardEffect::None
            }
        }
    }

    pub(crate) fn view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let layout = WizardLayout::new(area);

        render_wizard_header(
            frame,
            layout.header_area,
            WizardHeader {
                badge_label: "TICKET WIZARD",
                badge_color: theme.ok,
                active_step: self.active_step,
                total_steps: 4,
                title: " New Ticket ",
            },
            theme,
        );

        let form_area = if self.active_step == 1 {
            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .spacing(Spacing::Overlap(1))
                .constraints([
                    Constraint::Percentage(55),
                    Constraint::Fill(1),
                ])
                .split(layout.body_area);

            let snippets = SnippetRef::extract_from_text(&self.description);
            let preview_lines = if let Some(first_ref) = snippets.first() {
                load_file_snippet(
                    &first_ref.path,
                    None,
                    first_ref.range,
                    &theme.select_symbol,
                    theme.ok,
                    theme.text,
                    theme.subtext,
                )
            } else {
                vec![
                    Line::from(vec![
                        theme.badge("SNIPPET PREVIEW", theme.info, Color::Black),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled("Mention any file in your description to preview it live:", Style::default().fg(theme.text))),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Syntax:  ", Style::default().fg(theme.warn)),
                        Span::styled("@path/to/file.rs", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Slice:   ", Style::default().fg(theme.warn)),
                        Span::styled("@path/to/file.rs[10..25]", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled("Lines are dynamically read and rendered with line numbers.", Style::default().fg(theme.subtext))),
                ]
            };

            let preview_title = if let Some(first_ref) = snippets.first() {
                format!(" Code Preview: {} ", first_ref.path.display())
            } else {
                " Live Code Preview ".to_string()
            };

            let preview_widget = Paragraph::new(preview_lines)
                .block(theme.block(&preview_title))
                .wrap(Wrap { trim: false });
            frame.render_widget(preview_widget, horiz_chunks[1]);

            horiz_chunks[0]
        } else {
            layout.body_area
        };

        match self.active_step {
            0 => render_active_text_step(frame, form_area, 0, "Title / Problem Summary", &self.header, theme),
            1 => render_active_text_step(frame, form_area, 1, "Description (supports @file[start..end])", &self.description, theme),
            2 => self.render_crash_step(frame, form_area, theme),
            3 => self.render_diagnostics_step(frame, form_area, theme),
            _ => {}
        }

        let footer_shortcuts = match self.active_step {
            0 => vec![
                ("Enter/Tab", "Next Step"),
                ("Esc", "Cancel"),
            ],
            1 => vec![
                ("Enter/Tab", "Next Step"),
                ("Shift+Tab", "Previous Step"),
                ("Esc", "Cancel"),
            ],
            2 => vec![
                ("↑/k, ↓/j", "Select Crash"),
                ("Enter/Tab", "Next Step"),
                ("Shift+Tab", "Previous Step"),
                ("Esc", "Cancel"),
            ],
            3 => vec![
                ("↑/k, ↓/j", "Navigate"),
                ("Space", "Toggle"),
                ("a", "Toggle All"),
                ("Enter", "Submit Ticket"),
                ("Shift+Tab", "Previous Step"),
                ("Esc", "Cancel"),
            ],
            _ => vec![("Esc", "Cancel")],
        };

        render_wizard_footer(frame, layout.footer_area, &footer_shortcuts, theme);
    }

    fn render_crash_step(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = theme.focused_block(" Step 3: Select Crash File to Attach (↑/↓ to choose, Enter to select) ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let visible_lines = inner.height as usize;
        let scroll = if self.crash_cursor < self.crash_scroll {
            self.crash_cursor
        } else if self.crash_cursor >= self.crash_scroll + visible_lines && visible_lines > 0 {
            self.crash_cursor - visible_lines + 1
        } else {
            self.crash_scroll
        };

        let mut lines = Vec::new();
        for (idx, opt) in self.crash_options.iter().enumerate().skip(scroll).take(visible_lines) {
            let is_cursor = idx == self.crash_cursor;
            let sym = if is_cursor {
                Span::styled(format!(" {} ", theme.select_symbol), Style::default().fg(theme.text_invert).add_modifier(Modifier::BOLD))
            } else {
                Span::raw("    ")
            };

            let item_spans = match opt {
                None => {
                    let text = Span::styled("<None> - Do not attach crash file (skip)", Style::default());
                    vec![sym, text]
                }
                Some(path) => {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let meta = metadata(path).ok();
                    let size = meta.as_ref().map(|m| format!("{} B", m.len())).unwrap_or_default();
                    let modified = meta
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            let dt: DateTime<Utc> = t.into();
                            dt.format("%Y-%m-%d %H:%M").to_string()
                        })
                        .unwrap_or_default();

                    vec![
                        sym,
                        Span::styled(format!("{:<36} ", name), Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{:>8}  ", size), Style::default().bg(theme.info).fg(theme.text_invert)),
                        Span::styled(modified, Style::default()),
                    ]
                }
            };

            let mut line = Line::from(item_spans);
            if is_cursor {
                line = line.style(theme.selected_style());
            }
            lines.push(line);
        }

        let widget = Paragraph::new(lines);
        frame.render_widget(widget, inner);
    }

    fn render_diagnostics_step(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        if let Some(crash) = &self.loaded_crash {
            let total = crash.crashes.len();
            let selected_count = self.selected_diagnostics.iter().filter(|&&s| s).count();
            let title = format!(
                " Step 4: Select Diagnostics ({selected_count}/{total} selected) [Space: Toggle, a: Toggle All, Enter: Save] "
            );
            let block = theme.focused_block(&title);
            let inner = block.inner(area);
            frame.render_widget(block, area);

            if crash.crashes.is_empty() {
                let p = Paragraph::new("Crash file has no recorded diagnostics.\nPress Enter to submit ticket.")
                    .alignment(Alignment::Center);
                frame.render_widget(p, inner);
            } else {
                let visible_lines = inner.height as usize;
                let scroll = if self.diag_cursor < self.diag_scroll {
                    self.diag_cursor
                } else if self.diag_cursor >= self.diag_scroll + visible_lines && visible_lines > 0 {
                    self.diag_cursor - visible_lines + 1
                } else {
                    self.diag_scroll
                };

                let mut lines = Vec::new();
                for (idx, diag) in crash.crashes.iter().enumerate().skip(scroll).take(visible_lines) {
                    let is_cursor = idx == self.diag_cursor;
                    let is_selected = self.selected_diagnostics.get(idx).copied().unwrap_or(false);

                    let cursor_span = if is_cursor {
                        Span::styled(format!("{} ", theme.select_symbol), Style::default().add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("   ")
                    };

                    let check_span = if is_selected {
                        Span::styled(format!("[{}] ", theme.choose_symbol), Style::default().add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled("[ ] ", Style::default().fg(theme.subtext))
                    };

                    let file_name = diag.file_path.file_name().unwrap_or_default().to_string_lossy();
                    let line_spans = vec![
                        cursor_span,
                        check_span,
                        Span::styled(format!("{:<10} ", diag.error_code), Style::default().bg(theme.err).fg(theme.text_invert).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{file_name}:{} ", diag.span.line), Style::default().bg(theme.info).fg(theme.text_invert)),
                        Span::styled(&diag.message, Style::default()),
                    ];

                    let mut line = Line::from(line_spans);
                    if is_cursor {
                        line = line.style(theme.selected_style());
                    }
                    lines.push(line);
                }

                let widget = Paragraph::new(lines);
                frame.render_widget(widget, inner);
            }
        } else {
            let block = theme.focused_block(" Step 4: Confirm Diagnostics ");
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let lines = vec![
                Line::raw(""),
                Line::from(Span::styled("No crash file attached to this ticket.", Style::default().fg(theme.subtext))),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(theme.text)),
                    Span::styled("Enter", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
                    Span::styled(" to create and save the ticket.", Style::default().fg(theme.text)),
                ]),
            ];
            let widget = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(widget, inner);
        }
    }
}
