use crate::commands::enums::{PushRemote, PushTarget};
use crate::commands::list::paths_by_extension;
use crate::commands::push::push;
use crate::configs::find_bissue_dir;
use crate::lock::Lock;
use crate::render::keys::normalize_char;
use crate::render::theme::Theme;
use chrono::{DateTime, Utc};
use ratatui::crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Alignment, Flex, Spacing};
use ratatui::prelude::*;
use ratatui::widgets::*;
use std::fs::metadata;
use std::path::PathBuf;
use std::process::Command;
use tokio::runtime::Runtime;

#[derive(Copy, Clone, PartialEq, Eq)]
pub(crate) enum ListTab {
    Crashes = 0,
    Issues = 1,
    Tickets = 2,
}

pub(crate) enum ListMsg {
    NextTab,
    PrevTab,
    SelectTab(ListTab),
    NextItem,
    PrevItem,
    OpenFile,
    DeleteSelected,
    PushSelected,
    Quit,
}

pub(crate) enum ListEffect {
    None,
    Quit,
    DeleteConfirm(Vec<PathBuf>, String),
}

pub(crate) struct ListModel {
    pub(crate) current_tab: ListTab,
    pub(crate) selected_index: usize,
    pub(crate) items: Vec<PathBuf>,
    pub(crate) lock: Lock,
    pub(crate) status_message: Option<String>,
}

impl ListModel {
    pub(crate) fn new(initial_tab: Option<ListTab>) -> Self {
        let tab = initial_tab.unwrap_or(ListTab::Crashes);
        let mut model = Self {
            current_tab: tab,
            selected_index: 0,
            items: Vec::new(),
            lock: Lock::load(),
            status_message: None,
        };
        model.reload_items();
        model
    }

    pub(crate) fn reload_items(&mut self) {
        let bissue_dir = find_bissue_dir();
        self.lock = Lock::load();
        let (folder, ext) = match self.current_tab {
            ListTab::Crashes => ("crashes", "crash"),
            ListTab::Issues => ("issues", "issue"),
            ListTab::Tickets => ("tickets", "ticket"),
        };

        let dir = bissue_dir.join(folder);
        self.items = paths_by_extension(&dir, ext).unwrap_or_default();
        if self.selected_index >= self.items.len() {
            self.selected_index = self.items.len().saturating_sub(1);
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) -> Option<ListMsg> {
        match key.code {
            KeyCode::Esc => Some(ListMsg::Quit),
            KeyCode::Tab => Some(ListMsg::NextTab),
            KeyCode::BackTab => Some(ListMsg::PrevTab),
            KeyCode::Down => Some(ListMsg::NextItem),
            KeyCode::Up => Some(ListMsg::PrevItem),
            KeyCode::Enter => Some(ListMsg::OpenFile),
            KeyCode::Char('1') => Some(ListMsg::SelectTab(ListTab::Crashes)),
            KeyCode::Char('2') => Some(ListMsg::SelectTab(ListTab::Issues)),
            KeyCode::Char('3') => Some(ListMsg::SelectTab(ListTab::Tickets)),
            KeyCode::Char(c) => match normalize_char(c) {
                'j' => Some(ListMsg::NextItem),
                'k' => Some(ListMsg::PrevItem),
                'o' => Some(ListMsg::OpenFile),
                'd' => Some(ListMsg::DeleteSelected),
                'p' => Some(ListMsg::PushSelected),
                _ => None,
            },
            _ => None,
        }
    }

    pub(crate) fn update(&mut self, msg: ListMsg) -> ListEffect {
        match msg {
            ListMsg::Quit => ListEffect::Quit,
            ListMsg::NextTab => {
                self.current_tab = match self.current_tab {
                    ListTab::Crashes => ListTab::Issues,
                    ListTab::Issues => ListTab::Tickets,
                    ListTab::Tickets => ListTab::Crashes,
                };
                self.selected_index = 0;
                self.reload_items();
                ListEffect::None
            }
            ListMsg::PrevTab => {
                self.current_tab = match self.current_tab {
                    ListTab::Crashes => ListTab::Tickets,
                    ListTab::Issues => ListTab::Crashes,
                    ListTab::Tickets => ListTab::Issues,
                };
                self.selected_index = 0;
                self.reload_items();
                ListEffect::None
            }
            ListMsg::SelectTab(tab) => {
                self.current_tab = tab;
                self.selected_index = 0;
                self.reload_items();
                ListEffect::None
            }
            ListMsg::NextItem => {
                if !self.items.is_empty() {
                    self.selected_index = (self.selected_index + 1).min(self.items.len() - 1);
                }
                ListEffect::None
            }
            ListMsg::PrevItem => {
                self.selected_index = self.selected_index.saturating_sub(1);
                ListEffect::None
            }
            ListMsg::OpenFile => {
                if let Some(path) = self.items.get(self.selected_index) {
                    #[cfg(target_os = "windows")]
                    let _ = Command::new("cmd").args(["/C", "start", "", &path.to_string_lossy()]).spawn();
                    #[cfg(target_os = "macos")]
                    let _ = Command::new("open").arg(path).spawn();
                    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
                    let _ = Command::new("xdg-open").arg(path).spawn();
                }
                ListEffect::None
            }
            ListMsg::DeleteSelected => {
                if let Some(path) = self.items.get(self.selected_index) {
                    let name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    ListEffect::DeleteConfirm(vec![path.clone()], name)
                } else {
                    ListEffect::None
                }
            }
            ListMsg::PushSelected => {
                if self.current_tab == ListTab::Tickets
                    && !self.items.is_empty()
                    && let Some(path) = self.items.get(self.selected_index)
                {
                    let path_clone = path.clone();
                    let target = PushTarget::Single { path: path_clone };
                    let remote = PushRemote::All;
                    let custom = Vec::new();

                    let rt = Runtime::new().ok();
                    let res = rt.and_then(|r| r.block_on(push(&target, &remote, &custom)).ok());
                    if res.is_some() {
                        self.status_message = Some(format!("Ticket pushed: {}", path.display()));
                        self.reload_items();
                    } else {
                        self.status_message = Some("Failed to push ticket (check remotes/network)".to_string());
                    }
                }
                ListEffect::None
            }
        }
    }

    pub(crate) fn view(&self, frame: &mut Frame, area: Rect, theme: &Theme) {
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(8),
                Constraint::Length(1),
                Constraint::Length(3),
            ])
            .split(area);

        let table_chunks = Layout::default()
            .direction(Direction::Vertical)
            .spacing(Spacing::Overlap(1))
            .constraints([
                Constraint::Length(3),
                Constraint::Min(5),
            ])
            .split(main_chunks[0]);

        let block = theme.block(" Records Browser ");
        let inner = block.inner(table_chunks[0]);
        frame.render_widget(block, table_chunks[0]);

        let titles = vec!["  [1] Crashes  ", "  [2] Issues  ", "  [3] Tickets  "];
        let total_chars: u16 = titles.iter().map(|t| t.chars().count() as u16).sum::<u16>() + 4;
        let tab_area = Layout::default()
            .direction(Direction::Horizontal)
            .flex(Flex::Center)
            .constraints([Constraint::Length(total_chars)])
            .split(inner)[0];

        let tab_idx = self.current_tab as usize;
        let tabs = Tabs::new(titles)
            .select(tab_idx)
            .style(Style::default().fg(theme.subtext))
            .highlight_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
            .divider(Span::styled("│", Style::default().fg(theme.border)));
        frame.render_widget(tabs, tab_area);

        let header_cells = [" # ", "File Name", "Size", "Modified", "Status"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(theme.info).add_modifier(Modifier::BOLD)));
        let header = Row::new(header_cells).height(1).bottom_margin(1);

        let rows = self.items.iter().enumerate().map(|(idx, path)| {
            let num = format!("{:2}. ", idx + 1);
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let meta = metadata(path).ok();
            let size = meta.as_ref().map(|m| format!("{} B", m.len())).unwrap_or_default();
            let modified = meta
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let dt: DateTime<Utc> = t.into();
                    dt.format("%Y-%m-%d %H:%M").to_string()
                })
                .unwrap_or_else(|| "-".to_string());

            let status_cell = match self.current_tab {
                ListTab::Tickets => {
                    let canon = dunce::canonicalize(path).unwrap_or_else(|_| path.clone());
                    if self.lock.pushed.contains(&canon) {
                        Cell::from(Line::from(vec![
                            theme.badge("PUSHED", theme.ok, theme.text_invert),
                        ]))
                    } else {
                        Cell::from(Line::from(vec![
                            theme.badge("LOCAL", theme.warn, theme.text_invert),
                        ]))
                    }
                }
                ListTab::Crashes => Cell::from(Line::from(vec![
                    theme.badge("CRASH", theme.err, theme.text_invert),
                ])),
                ListTab::Issues => Cell::from(Line::from(vec![
                    theme.badge("ISSUE", theme.info, theme.text_invert),
                ])),
            };

            let row_cells = vec![
                Cell::from(num),
                Cell::from(name),
                Cell::from(size),
                Cell::from(modified),
                status_cell,
            ];

            let is_selected = idx == self.selected_index;
            let row_style = if is_selected {
                theme.selected_style()
            } else {
                Style::default().fg(theme.text)
            };

            Row::new(row_cells).style(row_style).height(1)
        });

        let widths = [
            Constraint::Length(5),
            Constraint::Percentage(45),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(15),
        ];

        let table_title = format!(" Files: {} ({} items) ", match self.current_tab {
            ListTab::Crashes => "Crashes",
            ListTab::Issues => "Issues",
            ListTab::Tickets => "Tickets",
        }, self.items.len());

        let table = Table::new(rows, widths)
            .header(header)
            .block(theme.block(&table_title));
        frame.render_widget(table, table_chunks[1]);

        if let Some(ref msg) = self.status_message {
            let status = Paragraph::new(Line::from(vec![
                theme.badge("STATUS", theme.info, theme.text_invert),
                Span::raw(" "),
                Span::styled(msg, Style::default().fg(theme.text)),
            ])).alignment(Alignment::Center);
            frame.render_widget(status, main_chunks[1]);
        }

        let is_tickets = self.current_tab == ListTab::Tickets;
        let mut shortcuts = vec![
            ("Tab", "Category"),
            ("↑/k, ↓/j", "Navigate"),
            ("o/Enter", "Open"),
            ("d", "Delete"),
        ];
        if is_tickets {
            shortcuts.push(("p", "Push Remote"));
        }
        shortcuts.push(("Esc", "Quit"));

        let footer = Paragraph::new(theme.shortcut_bar(&shortcuts)).alignment(Alignment::Center);
        frame.render_widget(footer, main_chunks[2]);
    }
}
