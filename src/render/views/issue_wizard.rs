use crate::models::issue::{Issue, IssueType};
use crate::render::keys::normalize_char;
use crate::render::theme::Theme;
use crate::render::views::wizard::{
    render_active_text_step, render_wizard_footer, render_wizard_header, WizardHeader, WizardLayout,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Spacing};
use ratatui::prelude::*;
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
        let layout = WizardLayout::new(area);

        render_wizard_header(
            frame,
            layout.header_area,
            WizardHeader {
                badge_label: "ISSUE WIZARD",
                badge_color: theme.info,
                active_step: self.active_step,
                total_steps: 5,
                title: " New Issue Definition ",
            },
            theme,
        );

        let form_area = if self.active_step >= 1 {
            let horiz_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .spacing(Spacing::Overlap(1))
                .constraints([
                    Constraint::Percentage(60),
                    Constraint::Fill(1),
                ])
                .split(layout.body_area);

            self.render_live_card_preview(frame, horiz_chunks[1], theme);
            horiz_chunks[0]
        } else {
            layout.body_area
        };

        match self.active_step {
            0 => self.render_type_step(frame, form_area, theme),
            1 => render_active_text_step(frame, form_area, 1, "Issue Code (e.g. ERR001)", &self.code, theme),
            2 => render_active_text_step(frame, form_area, 2, "Short Description / Message", &self.message, theme),
            3 => render_active_text_step(frame, form_area, 3, "Additional Note / Context (optional)", &self.note, theme),
            4 => render_active_text_step(frame, form_area, 4, "Advice / Recommended Solution (optional)", &self.advice, theme),
            _ => {}
        }

        let footer_shortcuts = [
            ("Enter/Tab", if self.active_step == 4 { "Save Issue" } else { "Next Step" }),
            ("Shift+Tab", "Previous Step"),
            ("1/2/3 or ←/→", "Change Type (Step 1)"),
            ("Esc", "Cancel"),
        ];
        render_wizard_footer(frame, layout.footer_area, &footer_shortcuts, theme);
    }

    fn render_type_step(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = theme.focused_block(" Step 1: Select Severity Type (← / → or 1 / 2 / 3) ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let sym = &theme.select_symbol;
        let make_btn = |label: &str, matches: bool, bg: Color| -> Span {
            if matches {
                Span::styled(format!("  {sym} [ {label} ]  "), Style::default().bg(bg).fg(theme.text_invert).add_modifier(Modifier::BOLD))
            } else {
                let spaces = " ".repeat(sym.chars().count() + 2);
                Span::styled(format!("{spaces}[ {label} ]  "), Style::default().fg(theme.subtext))
            }
        };

        let line = Line::from(vec![
            make_btn("1. Error", matches!(self.issue_type, IssueType::Error), theme.err),
            make_btn("2. Warning", matches!(self.issue_type, IssueType::Warning), theme.warn),
            make_btn("3. Fatal", matches!(self.issue_type, IssueType::Fatal), theme.accent),
        ]);

        let pad_lines = (inner.height.saturating_sub(1)) / 2;
        let mut lines = Vec::new();
        for _ in 0..pad_lines {
            lines.push(Line::raw(""));
        }
        lines.push(line);

        let widget = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(widget, inner);
    }

    fn render_live_card_preview(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let block = theme.block(" Live Issue Card Preview ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let type_badge = match self.issue_type {
            IssueType::Error => theme.badge("ERROR", theme.err, theme.text_invert),
            IssueType::Warning => theme.badge("WARNING", theme.warn, theme.text_invert),
            IssueType::Fatal => theme.badge("FATAL", theme.accent, theme.text_invert),
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

        let card_widget = Paragraph::new(preview_lines).wrap(Wrap { trim: false });
        frame.render_widget(card_widget, inner);
    }
}
