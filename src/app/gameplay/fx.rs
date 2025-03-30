use std::time::Duration;

use ratatui::style::palette::tailwind;
use tui_rain::Rain;

pub fn gen_matrix(duration: Duration) -> Rain {
    Rain::new_matrix(duration)
        .with_head_color(tailwind::LIME.c100)
        .with_color(tailwind::LIME.c400)
}
