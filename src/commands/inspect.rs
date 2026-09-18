use crate::render::app::{run_tui_app, App, AppScreen};
use crate::render::theme::Theme;
use crate::render::tui::Tui;
use crate::render::views::inspect::InspectModel;
use std::path::Path;

pub(crate) fn inspect(crash_file: Option<&Path>) -> anyhow::Result<()> {
    let theme = Theme::default_theme();
    let tui = Tui::init()?;
    let inspect_model = InspectModel::load(crash_file)?;
    let app = App::new(AppScreen::Inspect(inspect_model), theme);
    run_tui_app(tui, app)?;
    Ok(())
}