

use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::Frame;

use crate::app::ranking::PlayerListSelector;

use super::Activity;

#[derive(Default)]
pub struct ManageActivity<'a> {
    pub should_exit: bool,
    selector: PlayerListSelector<'a>,
    in_selector: bool,

}

impl ManageActivity<'_> {
    pub fn new() -> Self {
        Self {
            selector: PlayerListSelector::new("删除玩家"),
            in_selector: true,
            ..Default::default()
        }
    }

    fn reenter_selector(&mut self) {
        self.in_selector = true;
        self.selector = PlayerListSelector::new("删除玩家");
    }
}

impl Activity for ManageActivity<'_> {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        if self.in_selector {
            self.selector.draw(frame);
            return;
        }
    }

    fn update(&mut self, event: Option<Event>) {
        if self.in_selector {
            self.selector.update(event);

            if self.selector.should_exit {
                self.in_selector = false;
            }
            return;
        }

        let Some(event) = event else {
            return;
        };
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
            self.should_exit = true;
        }
    }
}
