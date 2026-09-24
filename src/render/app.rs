use crate::render::theme::Theme;
use crate::render::tui::Tui;
use crate::render::views::del_modal::{DelModalEffect, DelModalModel};
use crate::render::views::inspect::{InspectEffect, InspectModel};
use crate::render::views::issue_wizard::{IssueWizardModel};
use crate::render::views::list::{ListEffect, ListModel};
use crate::render::views::ticket_wizard::{TicketWizardModel};
use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use std::time::Duration;
use crate::render::views::wizard::WizardEffect;

pub(crate) enum AppScreen {
    Inspect(InspectModel),
    List(ListModel),
    DelModal {
        model: DelModalModel,
        return_to: Option<Box<AppScreen>>,
    },
    TicketWizard(TicketWizardModel),
    IssueWizard(IssueWizardModel),
}

pub(crate) struct App {
    pub(crate) screen: AppScreen,
    pub(crate) theme: Theme,
    pub(crate) running: bool,
    pub(crate) needs_clear: bool,
}

macro_rules! effect_match {
    ($model:expr, $key:expr, $($arms:tt)*) => {
        if let Some(msg) = $model.handle_key($key) {
            match $model.update(msg) {
                $($arms)*
            }
        }
    };
}

impl App {
    pub(crate) fn new(screen: AppScreen, theme: Theme) -> Self {
        Self {
            screen,
            theme,
            running: true,
            needs_clear: true,
        }
    }

    pub(crate) fn handle_key(&mut self, key: KeyEvent) {
        match &mut self.screen {
            AppScreen::Inspect(model) => {
                effect_match!(model, key,
                    InspectEffect::Quit => self.running = false,
                    InspectEffect::CreateTicketForCrash(path) => {
                        self.screen = AppScreen::TicketWizard(TicketWizardModel::new(Some(path)));
                        self.needs_clear = true;
                    }
                    InspectEffect::None => {}
                )
            }
            AppScreen::List(model) => {
                effect_match!(model, key,
                    ListEffect::Quit => self.running = false,
                    ListEffect::DeleteConfirm(files, title) => {
                        let current_list = AppScreen::List(ListModel::new(Some(model.current_tab)));
                        self.screen = AppScreen::DelModal {
                            model: DelModalModel::new(files, title),
                            return_to: Some(Box::new(current_list)),
                        };
                        self.needs_clear = true;
                    }
                    ListEffect::None => {}
                )
            }
            AppScreen::DelModal { model, return_to } => {
                effect_match!(model, key,
                    DelModalEffect::Confirmed | DelModalEffect::Cancelled => {
                        if let Some(prev) = return_to.take() {
                            self.screen = *prev;
                            self.needs_clear = true;
                        } else {
                            self.running = false;
                        }
                    }
                    DelModalEffect::None => {}
                )
            }
            AppScreen::TicketWizard(model) => {
                effect_match!(model, key,
                    WizardEffect::Quit | WizardEffect::Saved => self.running = false,
                    WizardEffect::None => {}
                )
            }
            AppScreen::IssueWizard(model) => {
                effect_match!(model, key,
                    WizardEffect::Quit | WizardEffect::Saved => self.running = false,
                    WizardEffect::None => {}
                )
            }
        }
    }

    pub(crate) fn view(&self, frame: &mut Frame) {
        let area = frame.area();
        match &self.screen {
            AppScreen::Inspect(model) => model.view(frame, area, &self.theme),
            AppScreen::List(model) => model.view(frame, area, &self.theme),
            AppScreen::DelModal { model, .. } => model.view(frame, area, &self.theme),
            AppScreen::TicketWizard(model) => model.view(frame, area, &self.theme),
            AppScreen::IssueWizard(model) => model.view(frame, area, &self.theme),
        }
    }
}

pub(crate) fn run_tui_app(mut tui: Tui, mut app: App) -> Result<()> {
    tui.terminal.clear()?;
    while app.running {
        if app.needs_clear {
            tui.terminal.clear()?;
            app.needs_clear = false;
        }
        tui.terminal.draw(|f| app.view(f))?;

        if event::poll(Duration::from_millis(50))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            app.handle_key(key);
        }
    }
    tui.restore()?;
    Ok(())
}
