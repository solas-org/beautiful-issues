use anyhow::Result;
use ratatui::crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::prelude::*;
use std::io::{stdout, Stdout};

pub(crate) struct Tui {
    pub(crate) terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub(crate) fn init() -> Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0))?;

        let panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
            panic_hook(panic_info);
        }));

        let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
        terminal.clear()?;
        Ok(Self { terminal })
    }

    pub(crate) fn restore(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}
