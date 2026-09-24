use std::fs::Metadata;
use std::rc::Rc;
use chrono::{DateTime, Utc};
use ratatui::layout::{Constraint, Direction, Layout, Rect, Spacing};

pub(crate) fn split_min_layout(direction: Direction, len: u16, min: u16, area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(direction)
        .constraints([
            Constraint::Min(min),
            Constraint::Length(len),
        ])
        .split(area)
}

pub(crate) fn split_len_layout(direction: Direction, len: u16, min: u16, area: Rect) -> Rc<[Rect]> {
    Layout::default()
        .direction(direction)
        .spacing(Spacing::Overlap(1))
        .constraints([
            Constraint::Length(len),
            Constraint::Min(min),
        ])
        .split(area)
}

pub(crate) fn time(meta: Option<&Metadata>) -> String{
    meta
        .and_then(|m| m.modified().ok())
        .map(|t| {
            let dt: DateTime<Utc> = t.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_default()
}