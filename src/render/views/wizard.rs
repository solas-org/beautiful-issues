use crate::render::theme::Theme;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
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

    pub(crate) fn move_up(&mut self) -> bool {
        let lines: Vec<&str> = self.value.split('\n').collect();
        if lines.len() <= 1 {
            return false;
        }
        let (cur_line, cur_col) = self.cursor_line_col();
        if cur_line == 0 {
            return false;
        }
        let target_line = cur_line - 1;
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

    pub(crate) fn move_down(&mut self) -> bool {
        let lines: Vec<&str> = self.value.split('\n').collect();
        if lines.len() <= 1 {
            return false;
        }
        let (cur_line, cur_col) = self.cursor_line_col();
        if cur_line + 1 >= lines.len() {
            return false;
        }
        let target_line = cur_line + 1;
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
                self.move_up();
                true
            }
            KeyCode::Down => {
                self.move_down();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_cursor_navigation() {
        let mut input = TextInput::new("hello");
        assert_eq!(input.cursor, 5);
        assert_eq!(input.rendered_with_cursor(), "hello█");

        input.move_left();
        assert_eq!(input.cursor, 4);
        assert_eq!(input.rendered_with_cursor(), "hell█o");

        input.move_home();
        assert_eq!(input.cursor, 0);
        assert_eq!(input.rendered_with_cursor(), "█hello");

        input.move_end();
        assert_eq!(input.cursor, 5);

        input.move_right();
        assert_eq!(input.cursor, 5);
    }

    #[test]
    fn test_text_input_editing() {
        let mut input = TextInput::new("");
        input.insert_char('a');
        input.insert_char('c');
        assert_eq!(input.as_str(), "ac");

        input.move_left();
        input.insert_char('b');
        assert_eq!(input.as_str(), "abc");

        input.backspace();
        assert_eq!(input.as_str(), "ac");

        input.move_home();
        input.delete();
        assert_eq!(input.as_str(), "c");
    }

    #[test]
    fn test_text_input_unicode() {
        let mut input = TextInput::new("привет");
        assert_eq!(input.cursor, 6);
        input.move_left();
        assert_eq!(input.cursor, 5);
        input.insert_char('!');
        assert_eq!(input.as_str(), "приве!т");
    }

    #[test]
    fn test_text_input_shift_enter_newline() {
        let mut input = TextInput::new("first line");
        let shift_enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        let handled = input.handle_key(&shift_enter_event);
        assert!(handled);
        assert_eq!(input.as_str(), "first line\n");

        let char_lf_event = KeyEvent::new(KeyCode::Char('\n'), KeyModifiers::SHIFT);
        let handled2 = input.handle_key(&char_lf_event);
        assert!(handled2);
        assert_eq!(input.as_str(), "first line\n\n");

        let backtab_event = KeyEvent::new(KeyCode::BackTab, KeyModifiers::empty());
        let backtab_handled = input.handle_key(&backtab_event);
        assert!(!backtab_handled);

        let regular_enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::empty());
        let regular_enter_handled = input.handle_key(&regular_enter);
        assert!(!regular_enter_handled);
    }

    #[test]
    fn test_text_input_multiline_navigation() {
        let mut input = TextInput::new("hello\nworld");
        assert_eq!(input.cursor, 11);
        input.move_home();
        assert_eq!(input.cursor, 6);

        input.move_end();
        assert_eq!(input.cursor, 11);

        input.move_up();
        assert_eq!(input.cursor, 5);

        input.move_home();
        assert_eq!(input.cursor, 0);

        input.move_down();
        assert_eq!(input.cursor, 6);
    }

    #[test]
    fn test_text_input_single_line() {
        let mut input = TextInput::new_single_line("hello");
        let shift_enter_event = KeyEvent::new(KeyCode::Enter, KeyModifiers::SHIFT);
        let handled = input.handle_key(&shift_enter_event);
        assert!(handled);
        assert_eq!(input.as_str(), "hello");

        input.insert_char('\n');
        assert_eq!(input.as_str(), "hello");
    }
}
