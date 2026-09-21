use crate::commands::enums::DelTarget;
use crate::commands::list::paths_by_extension;
use std::path::Path;
use anyhow::{Result};

use crate::render::app::{run_tui_app, App, AppScreen};
use crate::render::theme::Theme;
use crate::render::tui::Tui;
use crate::render::views::del_modal::DelModalModel;
use std::path::PathBuf;

pub(crate) fn del_tui(
    del_target: &DelTarget,
    dir: &Path,
    extension: &str,
    last_file_path: &Path,
) -> Result<()> {
    let files: Vec<PathBuf> = match del_target {
        DelTarget::Last => {
            if last_file_path.as_os_str().is_empty() || !last_file_path.exists() {
                println!("No last file exists to delete.");
                return Ok(());
            }
            vec![last_file_path.to_path_buf()]
        }
        DelTarget::Single { path } => {
            if !path.exists() {
                println!("File {} does not exist.", path.display());
                return Ok(());
            }
            vec![path.clone()]
        }
        DelTarget::All => {
            let found = paths_by_extension(dir, extension)?;
            if found.is_empty() {
                println!("No files with .{extension} found to delete.");
                return Ok(());
            }
            found
        }
    };

    let theme = Theme::default_theme();
    let tui = Tui::init()?;
    let modal = DelModalModel::new(files, format!("{extension}s"));
    let app = App::new(AppScreen::DelModal { model: modal, return_to: None }, theme);
    run_tui_app(tui, app)?;
    Ok(())
}