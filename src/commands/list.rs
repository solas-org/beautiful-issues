use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::render::app::{run_tui_app, App, AppScreen};
use crate::render::theme::Theme;
use crate::render::tui::Tui;
use crate::render::views::list::{ListModel, ListTab};

pub(crate) fn list_tui(initial_tab: Option<ListTab>) -> Result<()> {
    let theme = Theme::default_theme();
    let tui = Tui::init()?;
    let model = ListModel::new(initial_tab);
    let app = App::new(AppScreen::List(model), theme);
    run_tui_app(tui, app)?;
    Ok(())
}

pub(crate) fn paths_by_extension(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut res: Vec<PathBuf> = Vec::new();

    if !dir.exists() {
        return Ok(res);
    }

    let entries = std::fs::read_dir(dir)
        .with_context(|| format!("Cannot read directory {}", dir.display()))?;

    for file in entries.flatten() {
        let path = file.path();
        if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            let canonical = dunce::canonicalize(&path).unwrap_or(path);
            res.push(canonical);
        }
    }
    Ok(res)
}

