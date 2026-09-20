use anyhow::Result;
use ratatui::crossterm::{
    cursor::MoveTo,
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use ratatui::prelude::*;
use std::io::{stdout, Stdout};
use std::panic::{set_hook, take_hook};

pub(crate) struct Tui {
    pub(crate) terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    pub(crate) fn init() -> Result<Self> {
        enable_raw_mode()?;
        let mut out = stdout();
        execute!(out, EnterAlternateScreen, Clear(ClearType::All), MoveTo(0, 0))?;
        let _ = execute!(
            out,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        );

        let panic_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = execute!(stdout(), PopKeyboardEnhancementFlags);
            let _ = disable_raw_mode();
            let _ = execute!(stdout(), LeaveAlternateScreen);
            panic_hook(panic_info);
        }));

        let mut terminal = Terminal::new(CrosstermBackend::new(out))?;
        terminal.clear()?;
        Ok(Self { terminal })
    }

    pub(crate) fn restore(&mut self) -> Result<()> {
        let _ = execute!(self.terminal.backend_mut(), PopKeyboardEnhancementFlags);
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
