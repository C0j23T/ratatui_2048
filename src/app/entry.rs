use std::io::{Result, stdout};

use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

use super::{data::DataManager, screens::{App, AppState}};

pub fn start_app<D: DataManager>(data: D) -> Result<()> {
    let mut app = App::new(data);
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

    std::panic::set_hook(Box::new(|panic_info| {
        let _ = leave();
        println!("😱😱😱😱😱微距了😱😱😱😱😱");
        println!("{panic_info}");
        println!("😱😱😱😱😱😱😱😱😱😱😱😱😱");
    }));

    Ok(())
}

pub fn leave() -> Result<()> {
    disable_raw_mode()?;
    stdout().execute(DisableMouseCapture)?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
