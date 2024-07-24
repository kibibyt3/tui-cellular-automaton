use std::io::{self, stdout, Stdout};

use ratatui::{crossterm::{cursor::{self, Show}, execute, terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}}, prelude::CrosstermBackend, Terminal};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> io::Result<Tui> {
    execute!(stdout(), cursor::Hide, EnterAlternateScreen)?;
    enable_raw_mode()?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

pub fn restore() -> io::Result<()> {
    execute!(stdout(), cursor::Show, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
