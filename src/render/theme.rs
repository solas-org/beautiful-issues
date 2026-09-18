use crate::configs::UserConfig;
use crate::render::styling::parse_hex;
use ratatui::layout::Alignment;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::symbols::merge::MergeStrategy;
use ratatui::widgets::{Block, BorderType, Borders};

#[derive(Clone, Debug)]
pub(crate) struct Theme {
    pub(crate) ok: Color,
    pub(crate) err: Color,
    pub(crate) warn: Color,
    pub(crate) info: Color,
    pub(crate) border: Color,
    pub(crate) text: Color,
    pub(crate) subtext: Color,
    pub(crate) highlight: Color,
    pub(crate) select_symbol: String,
}

fn hex_to_color(hex: &str) -> Color {
    let (r, g, b) = parse_hex(hex);
    Color::Rgb(r, g, b)
}

impl Theme {
    pub(crate) fn from_config(config: &UserConfig) -> Self {
        Self {
            ok: hex_to_color(&config.ok_color),
            err: hex_to_color(&config.err_color),
            warn: hex_to_color(&config.warn_color),
            info: hex_to_color(&config.info_color),
            border: hex_to_color(&config.border_color),
            text: hex_to_color(&config.text_color),
            subtext: hex_to_color(&config.subtext_color),
            highlight: hex_to_color(&config.highlight_color),
            select_symbol: config.select_symbol.clone(),
        }
    }

    pub(crate) fn default_theme() -> Self {
        Self::from_config(UserConfig::global())
    }

    pub(crate) fn block<'a>(&self, title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.border))
            .title(format!(" {title} "))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(self.text).add_modifier(Modifier::BOLD))
            .merge_borders(MergeStrategy::Replace)
    }

    pub(crate) fn focused_block<'a>(&self, title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.highlight))
            .title(format!(" {title} "))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(self.highlight).add_modifier(Modifier::BOLD))
            .merge_borders(MergeStrategy::Exact)
    }

    pub(crate) fn alert_block<'a>(&self, title: &'a str) -> Block<'a> {
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(self.err))
            .title(format!(" {title} "))
            .title_alignment(Alignment::Center)
            .title_style(Style::default().fg(self.err).add_modifier(Modifier::BOLD))
            .merge_borders(MergeStrategy::Exact)
    }

    pub(crate) fn selected_style(&self) -> Style {
        Style::default()
            .bg(self.highlight)
            .fg(self.text)
            .add_modifier(Modifier::BOLD)
    }

    pub(crate) fn badge<'a>(&self, text: &'a str, bg: Color, fg: Color) -> Span<'a> {
        Span::styled(
            format!(" {text} "),
            Style::default().bg(bg).fg(fg).add_modifier(Modifier::BOLD),
        )
    }

    pub(crate) fn shortcut_bar<'a>(&self, items: &[(&'a str, &'a str)]) -> Line<'a> {
        let mut spans = Vec::new();
        for (idx, (key, desc)) in items.iter().enumerate() {
            if idx > 0 {
                spans.push(Span::raw("   "));
            }
            spans.push(Span::styled(
                format!(" {key} "),
                Style::default().bg(self.border).fg(self.text).add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::styled(
                format!(" {desc}"),
                Style::default().fg(self.subtext),
            ));
        }
        Line::from(spans)
    }
}
