use crate::commands::list::paths_by_extension;
use crate::configs::find_bissue_dir;
use crate::lock::Lock;
use crate::models::crash::CrashFile;
use crate::models::ticket::Ticket;
use crate::render::snippet::{SnippetRef, load_file_snippet};
use crate::render::theme::Theme;
use crate::render::views::helpers::{nav_key, time};
use crate::render::views::wizard::{
    TextInput, WizardEffect, WizardHeader, WizardLayout, render_active_text_step,
    render_wizard_footer, render_wizard_header,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Spacing};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(crate) enum TicketWizardMsg {
    InputChanged,
    NextStep,
    PrevStep,
    Up,
    Down,
    ToggleSelection,
    ToggleAll,
    ScrollPreviewUp,
    ScrollPreviewDown,
    ConfirmSave,
    Quit,
}

pub(crate) struct TicketWizardModel {
    pub(crate) active_step: usize,
    pub(crate) header: TextInput,
    pub(crate) description: TextInput,
    pub(crate) crash_options: Vec<Option<PathBuf>>,
    pub(crate) crash_cursor: usize,
    pub(crate) crash_scroll: usize,
    pub(crate) loaded_crash: Option<CrashFile>,
    pub(crate) selected_diagnostics: Vec<bool>,
    pub(crate) diag_cursor: usize,
    pub(crate) diag_scroll: usize,
    pub(crate) preview_scroll: usize,
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
            header: TextInput::new_single_line(""),
            description: TextInput::new_multiline(""),
            crash_options,
            crash_cursor: initial_cursor,
            crash_scroll: 0,
            loaded_crash: None,
            selected_diagnostics: Vec::new(),
            diag_cursor: 0,
            diag_scroll: 0,
            preview_scroll: 0,
            status: None,
        }
    }

    fn active_input_mut(&mut self) -> Option<&mut TextInput> {
        match self.active_step {
            0 => Some(&mut self.header),
            1 => Some(&mut self.description),
            _ => None,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<TicketWizardMsg> {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && let KeyCode::Char(c) = key.code
            && nav_key(c) == Some('s')
        {
            return Some(TicketWizardMsg::NextStep);
        }

        if self.active_step <= 1 {
            if let Some(input) = self.active_input_mut()
                && input.handle_key(&key)
            {
                return Some(TicketWizardMsg::InputChanged);
            }
        } else if self.active_step == 2 {
            match key.code {
                KeyCode::Up => return Some(TicketWizardMsg::Up),
                KeyCode::Down => return Some(TicketWizardMsg::Down),
                KeyCode::Char(c) => match nav_key(c) {
                    Some('k') => return Some(TicketWizardMsg::Up),
                    Some('j') => return Some(TicketWizardMsg::Down),
                    _ => {}
                },
                _ => {}
            }
        } else if self.active_step == 3 {
            match key.code {
                KeyCode::Up => return Some(TicketWizardMsg::Up),
                KeyCode::Down => return Some(TicketWizardMsg::Down),
                KeyCode::Char(' ') => return Some(TicketWizardMsg::ToggleSelection),
                KeyCode::Char(c) => match nav_key(c) {
                    Some('k') => return Some(TicketWizardMsg::Up),
                    Some('j') => return Some(TicketWizardMsg::Down),
                    Some('a') => return Some(TicketWizardMsg::ToggleAll),
                    _ => {}
                },
                _ => {}
            }
        } else if self.active_step == 4 {
            match key.code {
                KeyCode::Up | KeyCode::PageUp => return Some(TicketWizardMsg::ScrollPreviewUp),
                KeyCode::Down | KeyCode::PageDown => {
                    return Some(TicketWizardMsg::ScrollPreviewDown);
                }
                KeyCode::Char(c) => match nav_key(c) {
                    Some('k') => return Some(TicketWizardMsg::ScrollPreviewUp),
                    Some('j') => return Some(TicketWizardMsg::ScrollPreviewDown),
                    Some('y') => return Some(TicketWizardMsg::ConfirmSave),
                    _ => {}
                },
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => Some(TicketWizardMsg::Quit),
            KeyCode::Enter => {
                if self.active_step == 4 {
                    Some(TicketWizardMsg::ConfirmSave)
                } else {
                    Some(TicketWizardMsg::NextStep)
                }
            }
            KeyCode::Tab => Some(TicketWizardMsg::NextStep),
            KeyCode::BackTab => Some(TicketWizardMsg::PrevStep),
            _ => None,
        }
    }

    pub(crate) fn update(&mut self, msg: TicketWizardMsg) -> WizardEffect {
        match msg {
            TicketWizardMsg::Quit => WizardEffect::Quit,
            TicketWizardMsg::InputChanged => WizardEffect::None,
            TicketWizardMsg::PrevStep => {
                if self.active_step == 4 && self.loaded_crash.is_none() {
                    self.active_step = 2;
                } else {
                    self.active_step = self.active_step.saturating_sub(1);
                }
                WizardEffect::None
            }
            TicketWizardMsg::ScrollPreviewUp => {
                self.preview_scroll = self.preview_scroll.saturating_sub(2);
                WizardEffect::None
            }
            TicketWizardMsg::ScrollPreviewDown => {
                self.preview_scroll = self.preview_scroll.saturating_add(2);
                WizardEffect::None
            }
            TicketWizardMsg::Up => {
                if self.active_step == 2 {
                    self.crash_cursor = self.crash_cursor.saturating_sub(1);
                    if self.crash_cursor < self.crash_scroll {
                        self.crash_scroll = self.crash_cursor;
                    }
                } else if self.active_step == 3 {
                    self.diag_cursor = self.diag_cursor.saturating_sub(1);
                    if self.diag_cursor < self.diag_scroll {
                        self.diag_scroll = self.diag_cursor;
                    }
                }
                WizardEffect::None
            }
            TicketWizardMsg::Down => {
                if self.active_step == 2 {
                    if !self.crash_options.is_empty() {
                        self.crash_cursor =
                            (self.crash_cursor + 1).min(self.crash_options.len() - 1);
                    }
                } else if self.active_step == 3
                    && let Some(crash) = &self.loaded_crash
                    && !crash.crashes.is_empty()
                {
                    self.diag_cursor = (self.diag_cursor + 1).min(crash.crashes.len() - 1);
                }
                WizardEffect::None
            }

            TicketWizardMsg::ToggleSelection => {
                if self.active_step == 3 && self.diag_cursor < self.selected_diagnostics.len() {
                    self.selected_diagnostics[self.diag_cursor] =
                        !self.selected_diagnostics[self.diag_cursor];
                }
                WizardEffect::None
            }
            TicketWizardMsg::ToggleAll => {
                if self.active_step == 3 && !self.selected_diagnostics.is_empty() {
                    let any_unselected = self.selected_diagnostics.iter().any(|&s| !s);
                    self.selected_diagnostics.fill(any_unselected);
                }
                WizardEffect::None
            }
            TicketWizardMsg::NextStep => {
                match self.active_step {
                    0 => {
                        if !self.header.trim().is_empty() {
                            self.active_step = 1;
                        }
                    }
                    1 => {
                        if !self.description.trim().is_empty() {
                            self.active_step = 2;
                        }
                    }
                    2 => {
                        self.load_selected_crash();
                        if self.loaded_crash.is_some() {
                            self.active_step = 3;
                        } else {
                            self.preview_scroll = 0;
                            self.active_step = 4;
                        }
                    }
                    3 => {
                        self.preview_scroll = 0;
                        self.active_step = 4;
                    }
                    _ => {}
                }
                WizardEffect::None
            }
            TicketWizardMsg::ConfirmSave => self.save_ticket(),
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

    fn save_ticket(&mut self) -> WizardEffect {
        let crash_path = self
            .crash_options
            .get(self.crash_cursor)
            .and_then(|opt| opt.clone());
        let diag_indices: Vec<u32> = self
            .selected_diagnostics
            .iter()
            .enumerate()
            .filter_map(|(idx, &selected)| if selected { Some(idx as u32) } else { None })
            .collect();

        let final_description = embed_code_snippets(self.description.as_str());

        let ticket = Ticket {
            header: self.header.trim().to_string(),
            description: final_description,
            crash_file: crash_path,
            diagnostics: diag_indices,
        };

        match ticket.save(None) {
            Ok(..) => WizardEffect::Saved,
            Err(e) => {
                self.status = Some(format!("Error saving ticket: {e}"));
                WizardEffect::None
            }
        }
    }

    pub(crate) fn view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let layout = WizardLayout::new(area);

        let (display_step, total_steps) = if self.loaded_crash.is_none() {
            if self.active_step == 4 {
                (3, 4)
            } else {
                (self.active_step, 4)
            }
        } else {
            (self.active_step, 5)
        };

        render_wizard_header(
            frame,
            layout.header_area,
            WizardHeader {
                badge_label: "TICKET WIZARD",
                badge_color: theme.ok,
                active_step: display_step,
                total_steps,
                title: " New Ticket ",
            },
            theme,
        );

        let form_area = if self.active_step == 1 {
            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .spacing(Spacing::Overlap(1))
                .constraints([Constraint::Percentage(55), Constraint::Fill(1)])
                .split(layout.body_area);

            self.render_step_1_right_panel(frame, horiz_chunks[1], theme);
            horiz_chunks[0]
        } else {
            layout.body_area
        };

        match self.active_step {
            0 => render_active_text_step(
                frame,
                form_area,
                0,
                "Title / Problem Summary",
                &self.header,
                theme,
            ),
            1 => render_active_text_step(
                frame,
                form_area,
                1,
                "Description (supports @file[start..end])",
                &self.description,
                theme,
            ),
            2 => self.render_crash_step(frame, form_area, theme),
            3 => self.render_diagnostics_step(frame, form_area, theme),
            4 => self.render_preview_step(frame, form_area, theme),
            _ => {}
        }

        let footer_shortcuts = match self.active_step {
            0 => vec![("Enter/Tab", "Next Step"), ("Esc", "Cancel")],
            1 => vec![
                ("Enter/Tab", "Next Step"),
                ("Shift+Enter", "Newline"),
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
                ("Enter", "Preview Ticket"),
                ("Shift+Tab", "Previous Step"),
                ("Esc", "Cancel"),
            ],
            4 => {
                let back_hint = if self.loaded_crash.is_none() {
                    "Back to Crash Select"
                } else {
                    "Back to Diagnostics"
                };
                vec![
                    ("Enter", "Confirm"),
                    ("↑/k, ↓/j", "Scroll Preview"),
                    ("Shift+Tab", back_hint),
                    ("Esc", "Cancel"),
                ]
            }
            _ => vec![("Esc", "Cancel")],
        };

        render_wizard_footer(frame, layout.footer_area, &footer_shortcuts, theme);
    }

    fn render_step_1_right_panel(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let active_mention =
            active_mention_query(self.description.as_str(), self.description.cursor);

        if let Some(query_raw) = active_mention {
            let (path_str, range) = parse_query_path_range(&query_raw);
            let path = Path::new(path_str);

            if path.is_file() {
                let preview_lines = load_file_snippet(
                    path,
                    None,
                    range,
                    &theme.select_symbol,
                    theme.ok,
                    theme.text,
                    theme.subtext,
                );
                let title = format!(" Code Snippet: {path_str} ");
                let widget = Paragraph::new(preview_lines)
                    .block(theme.block(&title))
                    .wrap(Wrap { trim: false });
                frame.render_widget(widget, area);
            } else {
                let matches = scan_project_paths(path_str);
                let title = format!(" Files & Folders (@{path_str}) ");
                let block = theme.block(&title);
                let inner = block.inner(area);
                frame.render_widget(block, area);

                let visible = inner.height as usize;
                let mut lines = Vec::new();
                if matches.is_empty() {
                    lines.push(Line::raw(""));
                    lines.push(Line::from(Span::styled(
                        "  No matching files or directories found.",
                        Style::default().fg(theme.subtext),
                    )));
                } else {
                    for entry in matches.iter().take(visible) {
                        if entry.is_dir {
                            lines.push(Line::from(vec![
                                Span::styled("  📁 ", Style::default().fg(theme.warn)),
                                Span::styled(
                                    &entry.path,
                                    Style::default().add_modifier(Modifier::BOLD),
                                ),
                            ]));
                        } else {
                            lines.push(Line::from(vec![
                                Span::styled("  📄 ", Style::default().fg(theme.info)),
                                Span::styled(&entry.path, Style::default()),
                            ]));
                        }
                    }
                }
                let widget = Paragraph::new(lines);
                frame.render_widget(widget, inner);
            }
        } else {
            let snippets = SnippetRef::extract_from_text(self.description.as_str());
            if let Some(first_ref) = snippets.first() {
                let preview_lines = load_file_snippet(
                    &first_ref.path,
                    None,
                    first_ref.range,
                    &theme.select_symbol,
                    theme.ok,
                    theme.text,
                    theme.subtext,
                );
                let title = format!(" Code Preview: {} ", first_ref.path.display());
                let widget = Paragraph::new(preview_lines)
                    .block(theme.block(&title))
                    .wrap(Wrap { trim: false });
                frame.render_widget(widget, area);
            } else {
                let instructions = vec![
                    Line::from(vec![theme.badge(
                        "SNIPPET PREVIEW",
                        theme.info,
                        theme.text_invert,
                    )]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Type '@' to autocomplete files & folders:",
                        Style::default().fg(theme.text),
                    )),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Syntax:  ", Style::default().fg(theme.warn)),
                        Span::styled(
                            "@path/to/file.rs",
                            Style::default().fg(theme.ok).add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Slice:   ", Style::default().fg(theme.warn)),
                        Span::styled(
                            "@path/to/file.rs[10..25]",
                            Style::default().fg(theme.ok).add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(Span::styled(
                        "Code blocks will be embedded directly into ticket description.",
                        Style::default().fg(theme.subtext),
                    )),
                ];
                let widget = Paragraph::new(instructions)
                    .block(theme.block(" Live Code Preview "))
                    .wrap(Wrap { trim: false });
                frame.render_widget(widget, area);
            }
        }
    }

    fn render_crash_step(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = theme.focused_block(
            " Step 3: Select Crash File to Attach (↑/↓ to choose, Enter to select) ",
        );
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
        for (idx, opt) in self
            .crash_options
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_lines)
        {
            let is_cursor = idx == self.crash_cursor;
            let sym = if is_cursor {
                Span::styled(
                    format!(" {} ", theme.select_symbol),
                    Style::default()
                        .fg(theme.text_invert)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("    ")
            };

            let item_spans = match opt {
                None => {
                    let text =
                        Span::styled("<None> - Do not attach crash file (skip)", Style::default());
                    vec![sym, text]
                }
                Some(path) => {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    let meta = metadata(path).ok();
                    let size = meta
                        .as_ref()
                        .map(|m| format!("{} B", m.len()))
                        .unwrap_or_default();
                    let modified = time(meta.as_ref());

                    vec![
                        sym,
                        Span::styled(
                            format!("{:<35} ", name),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!("{:<10}  ", size), Style::default()),
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
                " Step 4: Select Diagnostics ({selected_count}/{total} selected) [Space: Toggle, a: Toggle All, Enter: Preview] "
            );
            let block = theme.focused_block(&title);
            let inner = block.inner(area);
            frame.render_widget(block, area);

            if crash.crashes.is_empty() {
                let p = Paragraph::new(
                    "Crash file has no recorded diagnostics.\nPress Enter to continue to preview.",
                )
                .alignment(Alignment::Center);
                frame.render_widget(p, inner);
            } else {
                let visible_lines = inner.height as usize;
                let scroll = if self.diag_cursor < self.diag_scroll {
                    self.diag_cursor
                } else if self.diag_cursor >= self.diag_scroll + visible_lines && visible_lines > 0
                {
                    self.diag_cursor - visible_lines + 1
                } else {
                    self.diag_scroll
                };

                let mut lines = Vec::new();
                for (idx, diag) in crash
                    .crashes
                    .iter()
                    .enumerate()
                    .skip(scroll)
                    .take(visible_lines)
                {
                    let is_cursor = idx == self.diag_cursor;
                    let is_selected = self.selected_diagnostics.get(idx).copied().unwrap_or(false);

                    let cursor_span = if is_cursor {
                        Span::styled(
                            format!("{} ", theme.select_symbol),
                            Style::default().add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw("   ")
                    };

                    let check_span = if is_selected {
                        Span::styled(
                            format!("[{}] ", theme.choose_symbol),
                            Style::default().add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::styled("[ ] ", Style::default().fg(theme.subtext))
                    };

                    let file_name = diag
                        .file_path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();

                    let issue = &diag.get_issue().unwrap();
                    //TODO unwrap better
                    let line_spans = vec![
                        cursor_span,
                        check_span,
                        Span::styled(
                            format!("{:<20} ", diag.issue_code),
                            Style::default().add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{file_name}:{:<10} ", diag.span.line),
                            Style::default(),
                        ),
                        Span::styled(
                            issue.message.clone().unwrap_or(String::new()),
                            Style::default(),
                        ),
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
                Line::from(Span::styled(
                    "No crash file attached to this ticket.",
                    Style::default().fg(theme.subtext),
                )),
                Line::raw(""),
                Line::from(vec![
                    Span::styled("Press ", Style::default().fg(theme.text)),
                    Span::styled(
                        "Enter",
                        Style::default().fg(theme.ok).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " to continue to ticket preview.",
                        Style::default().fg(theme.text),
                    ),
                ]),
            ];
            let widget = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(widget, inner);
        }
    }

    fn render_preview_step(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let md_text = build_ticket_markdown(
            self.header.as_str(),
            self.description.as_str(),
            self.loaded_crash.as_ref(),
            &self.selected_diagnostics,
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Length(3)])
            .split(area);

        let preview_title = if self.loaded_crash.is_none() {
            " Step 4: Ticket Preview (Markdown) "
        } else {
            " Step 5: Ticket Preview (Markdown) "
        };
        let preview_block = theme.focused_block(preview_title);
        let preview_inner = preview_block.inner(chunks[0]);
        frame.render_widget(preview_block, chunks[0]);

        let lines = format_markdown_preview_lines(&md_text, theme);
        let preview_widget = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .scroll((self.preview_scroll as u16, 0));
        frame.render_widget(preview_widget, preview_inner);

        let button_line = Line::from(vec![
            Span::styled(
                " [Enter] Confirm ",
                Style::default()
                    .bg(theme.ok)
                    .fg(theme.text_invert)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(
                " [Esc] Cancel ",
                Style::default().bg(theme.border).fg(theme.text),
            ),
        ]);
        let buttons_widget = Paragraph::new(button_line).alignment(Alignment::Center);
        frame.render_widget(buttons_widget, chunks[1]);
    }
}

#[derive(Clone, Debug)]
struct PathEntry {
    path: String,
    is_dir: bool,
}

fn active_mention_query(text: &str, cursor_chars: usize) -> Option<String> {
    let byte_idx = text
        .char_indices()
        .nth(cursor_chars)
        .map(|(i, _)| i)
        .unwrap_or(text.len());
    let prefix = &text[..byte_idx];
    if let Some(at_idx) = prefix.rfind('@') {
        let query = &prefix[at_idx + 1..];
        if !query.contains(char::is_whitespace) {
            return Some(query.to_string());
        }
    }
    None
}

pub(crate) fn parse_query_path_range(query: &str) -> (&str, Option<(usize, usize)>) {
    let cleaned = query.trim_matches(|c: char| c == ',' || c == ';' || c == ')' || c == '(');
    if let Some(b_start) = cleaned.find('[')
        && let Some(b_end) = cleaned.find(']')
    {
        let p = &cleaned[..b_start];
        let r_str = &cleaned[b_start + 1..b_end];
        let r = if let Some((s_str, e_str)) = r_str.split_once("..") {
            let s = s_str.trim().parse::<usize>().ok();
            let e = e_str.trim().parse::<usize>().ok();
            match (s, e) {
                (Some(start), Some(end)) => Some((start, end)),
                (Some(start), None) => Some((start, start + 15)),
                (None, Some(end)) => Some((1, end)),
                _ => None,
            }
        } else {
            None
        };
        (p, r)
    } else {
        (cleaned, None)
    }
}

fn scan_project_paths(query: &str) -> Vec<PathEntry> {
    let q = query.replace('\\', "/").to_lowercase();
    let walker = WalkDir::new(".")
        .min_depth(1)
        .max_depth(6)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !name.starts_with('.') && name != "target" && name != "node_modules"
        });

    let mut entries = Vec::new();
    for entry in walker.filter_map(|e| e.ok()) {
        let p = entry.path();
        let rel = p.strip_prefix(".").unwrap_or(p);
        let norm = rel.to_string_lossy().replace('\\', "/");
        let is_dir = entry.file_type().is_dir();
        let display = if is_dir { format!("{norm}/") } else { norm };
        entries.push(PathEntry {
            path: display,
            is_dir,
        });
    }

    if q.is_empty() {
        entries.sort_by(|a, b| {
            let a_depth = a.path.matches('/').count();
            let b_depth = b.path.matches('/').count();
            a_depth
                .cmp(&b_depth)
                .then(b.is_dir.cmp(&a.is_dir))
                .then(a.path.cmp(&b.path))
        });
        return entries;
    }

    let mut prefix_matches = Vec::new();
    let mut contains_matches = Vec::new();

    for entry in entries {
        let lower = entry.path.to_lowercase();
        if lower.starts_with(&q) {
            prefix_matches.push(entry);
        } else if lower.contains(&q) {
            contains_matches.push(entry);
        }
    }

    prefix_matches.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.path.cmp(&b.path)));
    contains_matches.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.path.cmp(&b.path)));

    prefix_matches.extend(contains_matches);
    prefix_matches
}

pub(crate) fn embed_code_snippets(text: &str) -> String {
    let mut result = Vec::new();

    for line in text.lines() {
        let mut line_tokens = Vec::new();
        for word in line.split_whitespace() {
            if let Some(rest) = word.strip_prefix('@') {
                let cleaned =
                    rest.trim_matches(|c: char| c == ',' || c == ';' || c == ')' || c == '(');
                let (path_str, range) = parse_query_path_range(cleaned);
                let path = Path::new(path_str);
                if path.is_file()
                    && let Ok(content) = std::fs::read_to_string(path)
                {
                    let all_lines: Vec<&str> = content.lines().collect();
                    let (s, e) = match range {
                        Some((start, end)) => {
                            let s = start.saturating_sub(1).min(all_lines.len());
                            let e = end.min(all_lines.len());
                            (s, e.max(s + 1))
                        }
                        None => (0, all_lines.len().min(30)),
                    };
                    let slice = &all_lines[s..e.min(all_lines.len())];
                    let code_block = format!("\n```\n{}\n```\n", slice.join("\n"));
                    line_tokens.push(code_block);
                    continue;
                }
            }
            line_tokens.push(word.to_string());
        }
        result.push(line_tokens.join(" "));
    }

    result.join("\n").trim().to_string()
}

pub(crate) fn build_ticket_markdown(
    title: &str,
    description: &str,
    crash: Option<&CrashFile>,
    selected_diagnostics: &[bool],
) -> String {
    let mut md = String::new();
    md.push_str(&format!("# {}\n\n", title.trim()));
    md.push_str(&embed_code_snippets(description));

    if let Some(cf) = crash {
        let mut diag_lines = Vec::new();
        for (idx, diag) in cf.crashes.iter().enumerate() {
            if selected_diagnostics.get(idx).copied().unwrap_or(false) {
                let file_name = diag
                    .file_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy();
                diag_lines.push(format!(
                    "- `{}` ({}:{})",
                    diag.issue_code, file_name, diag.span.line
                ));
            }
        }
        if !diag_lines.is_empty() {
            md.push_str("\n\n### Diagnostics\n");
            for line in diag_lines {
                md.push_str(&line);
                md.push('\n');
            }
        }
    }

    md
}

fn format_markdown_preview_lines<'a>(text: &'a str, theme: &Theme) -> Vec<Line<'a>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;

    for l in text.lines() {
        if l.starts_with("```") {
            in_code_block = !in_code_block;
            lines.push(Line::from(Span::styled(
                l,
                Style::default().fg(theme.subtext),
            )));
        } else if in_code_block {
            lines.push(Line::from(vec![
                Span::styled("  │ ", Style::default().fg(theme.border)),
                Span::styled(l, Style::default()),
            ]));
        } else if l.starts_with('#') {
            lines.push(Line::from(Span::styled(
                l,
                Style::default().add_modifier(Modifier::BOLD),
            )));
        } else {
            lines.push(Line::from(Span::styled(l, Style::default())));
        }
    }

    lines
}
