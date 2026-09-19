use crate::render::theme::Theme;
use ratatui::layout::{Alignment, Spacing};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub(crate) struct WizardLayout {
    pub(crate) header_area: Rect,
    pub(crate) body_area: Rect,
    pub(crate) footer_area: Rect,
}

impl WizardLayout {
    pub(crate) fn new(area: Rect) -> Self {
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

        Self {
            header_area: vert_chunks[0],
            body_area: vert_chunks[1],
            footer_area: main_vert[1],
        }
    }
}

pub(crate) struct WizardHeader<'a> {
    pub(crate) badge_label: &'a str,
    pub(crate) badge_color: Color,
    pub(crate) active_step: usize,
    pub(crate) total_steps: usize,
    pub(crate) title: &'a str,
}

pub(crate) fn render_wizard_header(
    frame: &mut Frame,
    area: Rect,
    header: WizardHeader<'_>,
    theme: &Theme,
) {
    let header_line = Line::from(vec![
        theme.badge(header.badge_label, header.badge_color, Color::Black),
        Span::raw("  "),
        Span::styled(
            format!("Step {} of {}", header.active_step + 1, header.total_steps),
            Style::default().fg(theme.text).add_modifier(Modifier::BOLD),
        ),
    ]);
    let header_widget = Paragraph::new(header_line)
        .alignment(Alignment::Center)
        .block(theme.block(header.title));
    frame.render_widget(header_widget, area);
}

pub(crate) fn render_active_text_step(
    frame: &mut Frame,
    area: Rect,
    step: usize,
    label: &str,
    value: &str,
    theme: &Theme,
) {
    let title = format!(" Step {}: {} ", step + 1, label);
    let block = theme.focused_block(&title);

    let text_with_cursor = format!("{value}█");
    let widget = Paragraph::new(text_with_cursor)
        .style(Style::default().fg(theme.text))
        .block(block)
        .wrap(Wrap { trim: false });
    frame.render_widget(widget, area);
}

pub(crate) fn render_wizard_footer(
    frame: &mut Frame,
    area: Rect,
    shortcuts: &[(&str, &str)],
    theme: &Theme,
) {
    let footer_widget = Paragraph::new(theme.shortcut_bar(shortcuts))
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(footer_widget, area);
}
