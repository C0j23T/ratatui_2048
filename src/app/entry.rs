use std::sync::RwLock;
use std::time::{Duration, Instant};
use std::{
    io::{Result, stdout},
    sync::LazyLock,
};

use crossterm::{
    ExecutableCommand,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::CrosstermBackend};

use super::{data::DataManager, screens::App};

pub(super) static DATA_MANAGER: LazyLock<RwLock<Option<Box<dyn DataManager>>>> =
    LazyLock::new(|| RwLock::new(None));

#[macro_use]
mod macros {
    #[macro_export]
    macro_rules! data_manager {
        ($method:ident) => {{
                data_manager!($method,)
            }};
        (immediate, $method:ident) => {{
            let binding = $crate::app::entry::DATA_MANAGER.read().unwrap();
            binding.as_ref().unwrap().$method()
        }};
        ($method:ident, $($params:tt)*) => {{
            let mut binding = $crate::app::entry::DATA_MANAGER.write().unwrap();
            match binding.as_mut().unwrap().$method($($params)*) {
                Ok(x) => Some(x),
                Err(e) => match e {
                    $crate::app::data::TryRecvError::Empty => None,
                    $crate::app::data::TryRecvError::Timeout => {
                        let mut dialog_manager = $crate::app::screens::dialog::DIALOG_MANAGER.write().unwrap();
                        dialog_manager.push($crate::app::screens::dialog::Dialog::new(
                            " 遇到问题 ",
                            "在处理数据时遇到超时问题，部分操作无法继续",
                            ratatui::prelude::Alignment::Left,
                            false,
                            vec![String::from("确定")],
                            None,
                        ));
                        None
                    },
                    $crate::app::data::TryRecvError::Disconnect => {
                        let mut dialog_manager = $crate::app::screens::dialog::DIALOG_MANAGER.write().unwrap();
                        dialog_manager.push($crate::app::screens::dialog::Dialog::new(
                            " 遇到问题 ",
                            "严重错误：内部连接已断开",
                            ratatui::prelude::Alignment::Left,
                            false,
                            vec![String::from("确定")],
                            None,
                        ));
                        None
                    },
                }
            }
        }};
    }
}

pub const FPS: i32 = 30;

pub fn run_app(mut data: Box<dyn DataManager>) -> Result<()> {
    let is_first_launch = data.is_first_launch();
    {
        let mut data_manager = DATA_MANAGER.write().unwrap();
        *data_manager = Some(data);
    }

    let mut app = App::new(is_first_launch);
    app.change_state(super::screens::AppState::ManagePlayer);

    init()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let frame_duration = Duration::from_millis(1000 / FPS as u64);
    loop {
        let frame_start = Instant::now();

        let exit = app.update(&mut terminal)?;
        if exit {
            break;
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
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
