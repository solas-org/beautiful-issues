use crate::models::issue::{Issue, IssueType};
use crate::render::keys::normalize_char;
use crate::render::theme::Theme;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Spacing};
use ratatui::prelude::*;
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::*;
use std::path::PathBuf;

pub(crate) enum IssueWizardMsg {
    Char(char),
    Backspace,
    Enter,
    NextStep,
    PrevStep,
    SelectType(IssueType),
    NextType,
    PrevType,
    Quit,
}

pub(crate) enum IssueWizardEffect {
    None,
    Quit,
    Saved(#[allow(dead_code)] PathBuf),
}

pub(crate) struct IssueWizardModel {
    pub(crate) active_step: usize,
    pub(crate) issue_type: IssueType,
    pub(crate) code: String,
    pub(crate) message: String,
    pub(crate) note: String,
    pub(crate) advice: String,
    pub(crate) status: Option<String>,
}

impl IssueWizardModel {
    pub(crate) fn new() -> Self {
        Self {
            active_step: 0,
            issue_type: IssueType::Error,
            code: String::new(),
            message: String::new(),
            note: String::new(),
            advice: String::new(),
            status: None,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<IssueWizardMsg> {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && let KeyCode::Char(c) = key.code
            && normalize_char(c) == 's'
        {
            return Some(IssueWizardMsg::NextStep);
        }

        if self.active_step == 0 {
            match key.code {
                KeyCode::Left => return Some(IssueWizardMsg::PrevType),
                KeyCode::Right => return Some(IssueWizardMsg::NextType),
                KeyCode::Char('1') => return Some(IssueWizardMsg::SelectType(IssueType::Error)),
                KeyCode::Char('2') => return Some(IssueWizardMsg::SelectType(IssueType::Warning)),
                KeyCode::Char('3') => return Some(IssueWizardMsg::SelectType(IssueType::Fatal)),
                KeyCode::Char(c) => match normalize_char(c) {
                    'h' => return Some(IssueWizardMsg::PrevType),
                    'l' => return Some(IssueWizardMsg::NextType),
                    _ => {}
                },
                _ => {}
            }
        }

        match key.code {
            KeyCode::Esc => Some(IssueWizardMsg::Quit),
            KeyCode::Tab => Some(IssueWizardMsg::NextStep),
            KeyCode::BackTab => Some(IssueWizardMsg::PrevStep),
            KeyCode::Enter => Some(IssueWizardMsg::Enter),
            KeyCode::Backspace => Some(IssueWizardMsg::Backspace),
            KeyCode::Char(c) => Some(IssueWizardMsg::Char(c)),
            _ => None,
        }
    }

    pub(crate) fn update(&mut self, msg: IssueWizardMsg) -> IssueWizardEffect {
        match msg {
            IssueWizardMsg::Quit => IssueWizardEffect::Quit,
            IssueWizardMsg::SelectType(t) => {
                self.issue_type = t;
                IssueWizardEffect::None
            }
            IssueWizardMsg::NextType => {
                self.issue_type = match self.issue_type {
                    IssueType::Error => IssueType::Warning,
                    IssueType::Warning => IssueType::Fatal,
                    IssueType::Fatal => IssueType::Error,
                };
                IssueWizardEffect::None
            }
            IssueWizardMsg::PrevType => {
                self.issue_type = match self.issue_type {
                    IssueType::Error => IssueType::Fatal,
                    IssueType::Warning => IssueType::Error,
                    IssueType::Fatal => IssueType::Warning,
                };
                IssueWizardEffect::None
            }
            IssueWizardMsg::Char(c) => {
                match self.active_step {
                    1 => self.code.push(c),
                    2 => self.message.push(c),
                    3 => self.note.push(c),
                    4 => self.advice.push(c),
                    _ => {}
                }
                IssueWizardEffect::None
            }
            IssueWizardMsg::Backspace => {
                match self.active_step {
                    1 => { self.code.pop(); }
                    2 => { self.message.pop(); }
                    3 => { self.note.pop(); }
                    4 => { self.advice.pop(); }
                    _ => {}
                }
                IssueWizardEffect::None
            }
            IssueWizardMsg::PrevStep => {
                self.active_step = self.active_step.saturating_sub(1);
                IssueWizardEffect::None
            }
            IssueWizardMsg::NextStep | IssueWizardMsg::Enter => {
                match self.active_step {
                    0 => {
                        self.active_step = 1;
                        IssueWizardEffect::None
                    }
                    1 => {
                        if !self.code.trim().is_empty() {
                            self.active_step = 2;
                        }
                        IssueWizardEffect::None
                    }
                    2 => {
                        if !self.message.trim().is_empty() {
                            self.active_step = 3;
                        }
                        IssueWizardEffect::None
                    }
                    3 => {
                        self.active_step = 4;
                        IssueWizardEffect::None
                    }
                    4 => self.save_issue(),
                    _ => IssueWizardEffect::None,
                }
            }
        }
    }

    fn save_issue(&mut self) -> IssueWizardEffect {
        let note_opt = if !self.note.trim().is_empty() {
            Some(self.note.trim().to_string())
        } else {
            None
        };

        let advice_opt = if !self.advice.trim().is_empty() {
            Some(self.advice.trim().to_string())
        } else {
            None
        };

        let issue = Issue::new(
            Some(self.issue_type.clone()),
            Some(self.code.trim().to_string()),
            Some(self.message.trim().to_string()),
            note_opt,
            advice_opt,
        );

        match issue.save(None) {
            Ok(path) => IssueWizardEffect::Saved(path),
            Err(e) => {
                self.status = Some(format!("Error saving issue: {e}"));
                IssueWizardEffect::None
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

        let vert_chunks = Layout::default()
            .direction(Direction::Vertical)
            .spacing(Spacing::Overlap(1))
            .constraints([
                Constraint::Length(3),
                Constraint::Min(7),
            ])
            .split(main_vert[0]);

        let header_line = Line::from(vec![
            theme.badge("ISSUE WIZARD", theme.info, Color::Black),
            Span::raw("  "),
            Span::styled(format!("Step {} of 5", self.active_step + 1), Style::default().fg(theme.text).add_modifier(Modifier::BOLD)),
        ]);
        let header_widget = Paragraph::new(header_line)
            .alignment(Alignment::Center)
            .block(theme.block(" New Issue Definition "));
        frame.render_widget(header_widget, vert_chunks[0]);

        let visible_steps = self.active_step + 1;
        let mut constraints = Vec::new();
        for step in 0..visible_steps {
            if step == self.active_step {
                constraints.push(Constraint::Min(5));
            } else {
                constraints.push(Constraint::Length(3));
            }
        }

        let form_area = if self.active_step >= 1 {
            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .spacing(Spacing::Overlap(1))
                .constraints([
                    Constraint::Percentage(60),
                    Constraint::Fill(1),
                ])
                .split(vert_chunks[1]);

            self.render_live_card_preview(frame, horiz_chunks[1], theme);
            horiz_chunks[0]
        } else {
            vert_chunks[1]
        };

        let form_chunks = Layout::default()
            .direction(Direction::Vertical)
            .spacing(Spacing::Overlap(1))
            .constraints(constraints)
            .split(form_area);

        for step in 0..visible_steps {
            match step {
                0 => self.render_type_step(frame, form_chunks[0], theme),
                1 => self.render_text_step(frame, form_chunks[1], 1, "Issue Code (e.g. ERR001)", &self.code, theme),
                2 => self.render_text_step(frame, form_chunks[2], 2, "Short Description / Message", &self.message, theme),
                3 => self.render_text_step(frame, form_chunks[3], 3, "Additional Note / Context (optional)", &self.note, theme),
                4 => self.render_text_step(frame, form_chunks[4], 4, "Advice / Recommended Solution (optional)", &self.advice, theme),
                _ => {}
            }
        }

        let footer_shortcuts = [
            ("Enter/Tab", if self.active_step == 4 { "Save Issue" } else { "Next Step" }),
            ("Shift+Tab", "Previous Step"),
            ("1/2/3 or ←/→", "Change Type (Step 1)"),
            ("Esc", "Cancel"),
        ];
        let footer_widget = Paragraph::new(theme.shortcut_bar(&footer_shortcuts))
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(footer_widget, main_vert[1]);
    }

    fn render_type_step(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let is_completed = self.active_step > 0;

        if is_completed {
            let type_badge = match self.issue_type {
                IssueType::Error => theme.badge("ERROR", theme.err, Color::White),
                IssueType::Warning => theme.badge("WARNING", theme.warn, Color::Black),
                IssueType::Fatal => theme.badge("FATAL", Color::Red, Color::White),
            };

            let line = Line::from(vec![
                Span::styled(" ✔ ", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
                Span::styled("Severity: ", Style::default().fg(theme.subtext)),
                type_badge,
            ]);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .merge_borders(MergeStrategy::Exact);
            let widget = Paragraph::new(line).block(block);
            frame.render_widget(widget, area);
        } else {
            let block = theme.focused_block(" Step 1: Select Severity Type (← / → or 1 / 2 / 3) ");
            let inner = block.inner(area);
            frame.render_widget(block, area);

            let sym = &theme.select_symbol;
            let make_btn = |label: &str, matches: bool, bg: Color| -> Span {
                if matches {
                    Span::styled(format!("  {sym} [ {label} ]  "), Style::default().bg(bg).fg(Color::Black).add_modifier(Modifier::BOLD))
                } else {
                    let spaces = " ".repeat(sym.chars().count() + 2);
                    Span::styled(format!("{spaces}[ {label} ]  "), Style::default().fg(theme.subtext))
                }
            };

            let line = Line::from(vec![
                make_btn("1. Error", matches!(self.issue_type, IssueType::Error), theme.err),
                make_btn("2. Warning", matches!(self.issue_type, IssueType::Warning), theme.warn),
                make_btn("3. Fatal", matches!(self.issue_type, IssueType::Fatal), Color::Magenta),
            ]);

            let pad_lines = (inner.height.saturating_sub(1)) / 2;
            let mut lines = Vec::new();
            for _ in 0..pad_lines {
                lines.push(Line::raw(""));
            }
            lines.push(line);

            let widget = Paragraph::new(lines)
                .alignment(Alignment::Center);
            frame.render_widget(widget, inner);
        }
    }

    fn render_text_step(&self, frame: &mut Frame, area: Rect, step: usize, label: &str, value: &str, theme: &Theme) {
        let is_active = step == self.active_step;
        let is_completed = step < self.active_step;

        if !is_active && is_completed {
            let display_val = if value.is_empty() {
                Span::styled("(none)", Style::default().fg(theme.subtext))
            } else {
                let truncated = if value.len() > 35 {
                    format!("{}...", &value[..32])
                } else {
                    value.to_string()
                };
                Span::styled(truncated, Style::default().fg(theme.text).add_modifier(Modifier::BOLD))
            };

            let line = Line::from(vec![
                Span::styled(" ✔ ", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{label}: "), Style::default().fg(theme.subtext)),
                display_val,
            ]);

            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .merge_borders(MergeStrategy::Exact);
            let widget = Paragraph::new(line).block(block);
            frame.render_widget(widget, area);
        } else if is_active {
            let title = format!(" Step {}: {} ", step + 1, label);
            let block = theme.focused_block(&title);

            let text_with_cursor = format!("{value}█");
            let widget = Paragraph::new(text_with_cursor)
                .style(Style::default().fg(theme.text))
                .block(block)
                .wrap(Wrap { trim: false });
            frame.render_widget(widget, area);
        } else {
            let title = format!(" Step {}: {} (waiting) ", step + 1, label);
            let block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .title(format!(" {title} "))
                .title_alignment(Alignment::Center)
                .title_style(Style::default().fg(theme.subtext));

            let widget = Paragraph::new(value).block(block);
            frame.render_widget(widget, area);
        }
    }

    fn render_live_card_preview(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = theme.block(" Live Issue Card Preview ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let type_badge = match self.issue_type {
            IssueType::Error => theme.badge("ERROR", theme.err, Color::White),
            IssueType::Warning => theme.badge("WARNING", theme.warn, Color::Black),
            IssueType::Fatal => theme.badge("FATAL", Color::Red, Color::White),
        };

        let code_str = if self.code.is_empty() { "ERR_CODE" } else { &self.code };
        let msg_str = if self.message.is_empty() { "(Issue message preview will appear here)" } else { &self.message };

        let mut preview_lines = vec![
            Line::from(vec![
                type_badge,
                Span::raw("  "),
                Span::styled(code_str, Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Message / Explanation:", Style::default().fg(theme.subtext))),
            Line::from(Span::styled(msg_str, Style::default().fg(theme.text))),
            Line::from(""),
        ];

        if !self.note.is_empty() {
            preview_lines.push(Line::from(Span::styled("Additional Note / Context:", Style::default().fg(theme.warn).add_modifier(Modifier::BOLD))));
            preview_lines.push(Line::from(Span::styled(&self.note, Style::default().fg(theme.text))));
            preview_lines.push(Line::from(""));
        }

        if !self.advice.is_empty() {
            preview_lines.push(Line::from(Span::styled("Suggested Advice / Fix:", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD))));
            preview_lines.push(Line::from(Span::styled(&self.advice, Style::default().fg(theme.text))));
            preview_lines.push(Line::from(""));
        }

        preview_lines.push(Line::from(Span::styled(
            "──────────────────────────────────────────────",
            Style::default().fg(theme.border),
        )));
        preview_lines.push(Line::from(Span::styled(
            "This card shows how your custom issue will appear when inspected.",
            Style::default().fg(theme.subtext),
        )));

        let card_widget = Paragraph::new(preview_lines).wrap(Wrap { trim: false });
        frame.render_widget(card_widget, inner);
    }
}
