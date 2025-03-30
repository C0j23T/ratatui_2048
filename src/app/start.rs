use std::io::{Result, stdout};

use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

use super::{
    data::DataManager,
    screens::{App, AppState},
};

pub fn start_app<D: DataManager>(mut data: D) -> Result<()> {
    let mut app = App::new(
        if data.is_first_launch() {
            AppState::FirstStart
        } else {
            AppState::MainMenu
        },
        data,
    );
    app.change_state(AppState::Gameplay);

    init()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        let exit = app.update(&mut terminal)?;
        if exit {
            break;
        }
    }

    leave()?;
    Ok(())
}

pub fn init() -> Result<()> {
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    enable_raw_mode()?;
    Ok(())
}

pub fn leave() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
