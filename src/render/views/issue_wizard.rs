use crate::models::issue::{Issue, IssueType};
use crate::render::views::helpers::nav_key;
use crate::render::theme::Theme;
use crate::render::views::wizard::{
    render_active_text_step, render_wizard_footer, render_wizard_header, TextInput, WizardHeader,
    WizardLayout, WizardEffect
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment, Spacing};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub(crate) enum IssueWizardMsg {
    InputChanged,
    NextStep,
    PrevStep,
    SelectType(IssueType),
    NextType,
    PrevType,
    Quit,
}

pub(crate) struct IssueWizardModel {
    pub(crate) active_step: usize,
    pub(crate) issue_type: IssueType,
    pub(crate) code: TextInput,
    pub(crate) message: TextInput,
    pub(crate) note: TextInput,
    pub(crate) advice: TextInput,
    pub(crate) status: Option<String>,
}

impl IssueWizardModel {
    pub(crate) fn new() -> Self {
        Self {
            active_step: 0,
            issue_type: IssueType::Error,
            code: TextInput::new_single_line(""),
            message: TextInput::new_single_line(""),
            note: TextInput::new_multiline(""),
            advice: TextInput::new_multiline(""),
            status: None,
        }
    }

    fn active_input(&self) -> Option<&TextInput> {
        match self.active_step {
            1 => Some(&self.code),
            2 => Some(&self.message),
            3 => Some(&self.note),
            4 => Some(&self.advice),
            _ => None,
        }
    }

    fn active_input_mut(&mut self) -> Option<&mut TextInput> {
        match self.active_step {
            1 => Some(&mut self.code),
            2 => Some(&mut self.message),
            3 => Some(&mut self.note),
            4 => Some(&mut self.advice),
            _ => None,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<IssueWizardMsg> {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && let KeyCode::Char(c) = key.code
            && nav_key(c) == Some('s')
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
                KeyCode::Char(c) => match nav_key(c) {
                    Some('h') => return Some(IssueWizardMsg::PrevType),
                    Some('l') => return Some(IssueWizardMsg::NextType),
                    _ => {}
                },
                _ => {}
            }
        } else if let Some(input) = self.active_input_mut()
            && input.handle_key(&key)
        {
            return Some(IssueWizardMsg::InputChanged);
        }

        match key.code {
            KeyCode::Esc => Some(IssueWizardMsg::Quit),
            KeyCode::Tab | KeyCode::Enter => Some(IssueWizardMsg::NextStep),
            KeyCode::BackTab => Some(IssueWizardMsg::PrevStep),
            _ => None,
        }
    }

    pub(crate) fn update(&mut self, msg: IssueWizardMsg) -> WizardEffect {
        match msg {
            IssueWizardMsg::Quit => WizardEffect::Quit,
            IssueWizardMsg::InputChanged => WizardEffect::None,
            IssueWizardMsg::SelectType(t) => {
                self.issue_type = t;
                WizardEffect::None
            }
            IssueWizardMsg::NextType => {
                self.issue_type = match self.issue_type {
                    IssueType::Error => IssueType::Warning,
                    IssueType::Warning => IssueType::Fatal,
                    IssueType::Fatal => IssueType::Error,
                };
                WizardEffect::None
            }
            IssueWizardMsg::PrevType => {
                self.issue_type = match self.issue_type {
                    IssueType::Error => IssueType::Fatal,
                    IssueType::Warning => IssueType::Error,
                    IssueType::Fatal => IssueType::Warning,
                };
                WizardEffect::None
            }
            IssueWizardMsg::PrevStep => {
                self.active_step = self.active_step.saturating_sub(1);
                WizardEffect::None
            }
            IssueWizardMsg::NextStep => {
                match self.active_step {
                    0 => {
                        self.active_step = 1;
                        WizardEffect::None
                    }
                    1 => {
                        if !self.code.trim().is_empty() {
                            self.active_step = 2;
                        }
                        WizardEffect::None
                    }
                    2 => {
                        if !self.message.trim().is_empty() {
                            self.active_step = 3;
                        }
                        WizardEffect::None
                    }
                    3 => {
                        self.active_step = 4;
                        WizardEffect::None
                    }
                    4 => self.save_issue(),
                    _ => WizardEffect::None,
                }
            }
        }
    }

    fn save_issue(&mut self) -> WizardEffect {
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
            Ok(..) => WizardEffect::Saved,
            Err(e) => {
                self.status = Some(format!("Error saving issue: {e}"));
                WizardEffect::None
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

        let mut footer_shortcuts = Vec::new();
        footer_shortcuts.push(("Enter/Tab", if self.active_step == 4 { "Save Issue" } else { "Next Step" }));
        if let Some(input) = self.active_input()
            && input.multiline
        {
            footer_shortcuts.push(("Shift+Enter", "Newline"));
        }
        if self.active_step > 0 {
            footer_shortcuts.push(("Shift+Tab", "Previous Step"));
        }
        if self.active_step == 0 {
            footer_shortcuts.push(("1/2/3 or ←/→", "Change Type"));
        }
        footer_shortcuts.push(("Esc", "Cancel"));
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

        let code_str = if self.code.is_empty() { "ERR_CODE" } else { self.code.as_str() };

        let mut preview_lines = vec![
            Line::from(vec![
                type_badge,
                Span::raw("  "),
                Span::styled(code_str, Style::default().fg(theme.info).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled("Message / Explanation:", Style::default().fg(theme.subtext))),
        ];

        if self.message.is_empty() {
            preview_lines.push(Line::from(Span::styled("(Issue message preview will appear here)", Style::default().fg(theme.subtext))));
        } else {
            for line in self.message.as_str().split('\n') {
                preview_lines.push(Line::from(Span::styled(line, Style::default().fg(theme.text))));
            }
        }
        preview_lines.push(Line::from(""));

        if !self.note.is_empty() {
            preview_lines.push(Line::from(Span::styled("Additional Note / Context:", Style::default().fg(theme.warn).add_modifier(Modifier::BOLD))));
            for line in self.note.as_str().split('\n') {
                preview_lines.push(Line::from(Span::styled(line, Style::default().fg(theme.text))));
            }
            preview_lines.push(Line::from(""));
        }

        if !self.advice.is_empty() {
            preview_lines.push(Line::from(Span::styled("Suggested Advice / Fix:", Style::default().fg(theme.ok).add_modifier(Modifier::BOLD))));
            for line in self.advice.as_str().split('\n') {
                preview_lines.push(Line::from(Span::styled(line, Style::default().fg(theme.text))));
            }
            preview_lines.push(Line::from(""));
        }

        let card_widget = Paragraph::new(preview_lines).wrap(Wrap { trim: false });
        frame.render_widget(card_widget, inner);
    }
}
