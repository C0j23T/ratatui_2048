use crossterm::event::Event;
use ratatui::Frame;

use super::Activity;

pub struct MenuActivity {
    pub exit: bool,
}

impl Activity for MenuActivity {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        
    }

    fn update(&mut self, event: Option<Event>) {
        
    }
}
