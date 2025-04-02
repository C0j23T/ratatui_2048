use crossterm::event::Event;
use ratatui::{Frame, buffer::Buffer, layout::Rect, widgets::Widget};

use super::screens::Activity;

// TODO: Ranking控件，实现搜索，和选择条目
pub struct PlayerListSelector {}

impl Activity for PlayerListSelector {
    fn draw(&mut self, frame: &mut Frame<'_>) {}

    fn update(&mut self, event: Option<Event>) {}
}
