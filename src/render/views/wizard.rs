use crate::render::theme::Theme;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::{Alignment};
use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::render::views::helpers::{split_len_layout, split_min_layout};

pub(crate) enum WizardEffect {
    None,
    Quit,
    Saved,
}

pub(crate) struct WizardLayout {
    pub(crate) header_area: Rect,
    pub(crate) body_area: Rect,
    pub(crate) footer_area: Rect,
}

impl WizardLayout {
    pub(crate) fn new(area: Rect) -> Self {
        let main_vert = split_min_layout(Direction::Vertical, 1, 10, area);
        let chunks = split_len_layout(Direction::Vertical, 3, 7, main_vert[0]);

        Self {
            header_area: chunks[0],
            body_area: chunks[1],
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

#[derive(Clone, Debug)]
pub(crate) struct TextInput {
    pub(crate) value: String,
    pub(crate) cursor: usize,
    pub(crate) multiline: bool,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            multiline: true,
        }
    }
}

impl TextInput {
    #[allow(dead_code)]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self::new_multiline(value)
    }

    pub(crate) fn new_single_line(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self {
            value,
            cursor,
            multiline: false,
        }
    }

    pub(crate) fn new_multiline(value: impl Into<String>) -> Self {
        let value = value.into();
        let cursor = value.chars().count();
        Self {
            value,
            cursor,
            multiline: true,
        }
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        if !self.multiline && (c == '\n' || c == '\r') {
            return;
        }
        let byte_idx = self.byte_index_at_cursor();
        self.value.insert(byte_idx, c);
        self.cursor += 1;
    }

    pub(crate) fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            let byte_idx = self.byte_index_at_cursor();
            self.value.remove(byte_idx);
        }
    }

    pub(crate) fn delete(&mut self) {
        let char_count = self.value.chars().count();
        if self.cursor < char_count {
            let byte_idx = self.byte_index_at_cursor();
            self.value.remove(byte_idx);
        }
    }

    pub(crate) fn cursor_line_col(&self) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;
        for (i, c) in self.value.chars().enumerate() {
            if i == self.cursor {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }
        (line, col)
    }

    pub(crate) fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub(crate) fn move_right(&mut self) {
        let char_count = self.value.chars().count();
        if self.cursor < char_count {
            self.cursor += 1;
        }
    }

    pub(crate) fn move_vert(&mut self, diff: i64) -> bool {
        let lines: Vec<&str> = self.value.split('\n').collect();
        if lines.len() <= 1 {
            return false;
        }
        let (cur_line, cur_col) = self.cursor_line_col();
        if (diff > 0 && cur_line + 1 >= lines.len()) || (diff < 0 && cur_line == 0) {
            return false;
        }
        let target_line = (cur_line as i64 + diff) as usize;
        let target_col = cur_col.min(lines[target_line].chars().count());

        let mut new_cursor = 0;
        for (idx, l) in lines.iter().enumerate() {
            if idx == target_line {
                new_cursor += target_col;
                break;
            }
            new_cursor += l.chars().count() + 1;
        }
        self.cursor = new_cursor;
        true
    }

    pub(crate) fn move_home(&mut self) {
        let (_, cur_col) = self.cursor_line_col();
        self.cursor = self.cursor.saturating_sub(cur_col);
    }

    pub(crate) fn move_end(&mut self) {
        let (cur_line, cur_col) = self.cursor_line_col();
        let lines: Vec<&str> = self.value.split('\n').collect();
        if let Some(line_str) = lines.get(cur_line) {
            let line_len = line_str.chars().count();
            self.cursor += line_len.saturating_sub(cur_col);
        } else {
            self.cursor = self.value.chars().count();
        }
    }

    pub(crate) fn byte_index_at_cursor(&self) -> usize {
        self.value
            .char_indices()
            .nth(self.cursor)
            .map(|(idx, _)| idx)
            .unwrap_or(self.value.len())
    }

    pub(crate) fn rendered_with_cursor(&self) -> String {
        let byte_idx = self.byte_index_at_cursor();
        let (before, after) = self.value.split_at(byte_idx);
        format!("{before}█{after}")
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub(crate) fn trim(&self) -> &str {
        self.value.trim()
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.value
    }

    pub(crate) fn handle_key(&mut self, key: &KeyEvent) -> bool {
        let is_shift_enter = match key.code {
            KeyCode::Enter => key.modifiers.contains(KeyModifiers::SHIFT),
            KeyCode::Char('\n') | KeyCode::Char('\r') => key.modifiers.contains(KeyModifiers::SHIFT),
            _ => false,
        };
        if is_shift_enter {
            if self.multiline {
                self.insert_char('\n');
            }
            return true;
        }

        match key.code {
            KeyCode::Left => {
                self.move_left();
                true
            }
            KeyCode::Right => {
                self.move_right();
                true
            }
            KeyCode::Up => {
                self.move_vert(-1);
                true
            }
            KeyCode::Down => {
                self.move_vert(1);
                true
            }
            KeyCode::Home => {
                self.move_home();
                true
            }
            KeyCode::End => {
                self.move_end();
                true
            }
            KeyCode::Backspace => {
                self.backspace();
                true
            }
            KeyCode::Delete => {
                self.delete();
                true
            }
            KeyCode::Char(c)
                if !key.modifiers.contains(KeyModifiers::CONTROL)
                    && !key.modifiers.contains(KeyModifiers::ALT) =>
            {
                self.insert_char(c);
                true
            }
            _ => false,
        }
    }
}

pub(crate) fn render_active_text_step(
    frame: &mut Frame,
    area: Rect,
    step: usize,
    label: &str,
    input: &TextInput,
    theme: &Theme,
) {
    let title = format!(" Step {}: {} ", step + 1, label);
    let block = theme.focused_block(&title);

    let text_with_cursor = input.rendered_with_cursor();
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
