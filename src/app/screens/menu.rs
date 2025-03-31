use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use lolcat::Lolcat;
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Stylize, palette::tailwind},
    widgets::{Block, BorderType, Borders, Paragraph},
};
use rolling_background::RollingBackground;

use crate::app::{ascii, math::inverse_lerp, time::TIME, utils::rect_move};

use super::Activity;

#[derive(Default)]
pub struct MenuActivity {
    pub exit: bool,

    app_time: Duration,

    bg_changed: bool,
    bg_rect_a: Rect,
    bg_rect_b: Rect,
}

impl MenuActivity {
    pub fn new() -> Self {
        Self {
            exit: false,
            ..Default::default()
        }
    }
}

impl Activity for MenuActivity {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let [_, title, menu, bottom] = Layout::vertical([
            Constraint::Max(1),
            Constraint::Percentage(60),
            Constraint::Min(0),
            Constraint::Max(1),
        ])
        .areas(area);
        let [_, menu, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(60),
            Constraint::Fill(1),
        ])
        .flex(Flex::Center)
        .areas(menu);

        // Title
        {
            let title_block = Paragraph::new(ascii::logo())
                .alignment(Alignment::Center)
                .fg(tailwind::INDIGO.c50);
            frame.render_widget(title_block, title);
            frame.render_widget(
                Lolcat {
                    seed: 0.0,
                    frequency: 3.0,
                    spread: 20.0,
                    offset: self.app_time.as_secs_f32(),
                },
                title,
            );
        }

        // Bottom
        {
            let block = Block::bordered()
                .title_bottom(" Ver. 0.1.0  ")
                .title_alignment(Alignment::Right)
                .borders(Borders::NONE)
                .fg(tailwind::INDIGO.c200);
            frame.render_widget(block, bottom);
        }

        // Background
        {
            if self.app_time.as_secs() % 5 == 0 {
                if !self.bg_changed {
                    let length = 75;
                    let area_width = area.width as i32;
                    let area_height = area.height as i32;
                    let width = rolling_background::WIDTH as i32 - length;
                    let height = rolling_background::HEIGHT as i32 - length;

                    let mut rng = rand::rng();
                    let a_x = rng.random_range(length..width - area_width);
                    let a_y = rng.random_range(length..height - area_height);
                    let direction = rng.random_bool(0.5);
                    let motion = if rng.random_bool(0.5) { length } else { length };
                    let b_x = a_x + if direction { motion } else { 0 };
                    let b_y = a_y + if !direction { motion } else { 0 };
                    self.bg_rect_a = Rect::new(a_x as u16, a_y as u16, area.width, area.height);
                    self.bg_rect_b = Rect::new(b_x as u16, b_y as u16, area.width, area.height);
                }
                self.bg_changed = true;
            } else {
                self.bg_changed = false;
            }
            let progress = inverse_lerp(0.0..=10.0, self.app_time.as_secs_f32() % 10.0)
                .unwrap_or(1.0)
                .min(1.0);
            let rect = rect_move(self.bg_rect_a, self.bg_rect_b, progress);
            frame.render_widget(RollingBackground, rect);
        }

        let menu_block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("Menu")
            .fg(tailwind::INDIGO.c50);
        frame.render_widget(menu_block, menu);
    }

    fn update(&mut self, event: Option<Event>) {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
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
            self.exit = true;
        }
    }
}

mod rolling_background {
    use std::sync::LazyLock;

    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style},
        widgets::Widget,
    };

    use crate::app::{ascii::NOVAK, gameplay::colors};

    pub struct RollingBackground;

    static COLOR: LazyLock<Vec<Color>> = LazyLock::new(|| {
        [
            0x3f3f46, 0x3e3e45, 0x3d3d44, 0x3d3d43, 0x3c3c43, 0x3b3b42, 0x3a3a41, 0x3a3a40,
            0x39393f, 0x38383e, 0x37373d, 0x36363d, 0x36363c, 0x35353b, 0x34343a, 0x333339,
            0x323238, 0x323237, 0x313137, 0x303036, 0x2f2f35, 0x2f2f34, 0x2e2e33, 0x2d2d32,
            0x2c2c31, 0x2b2b31, 0x2b2b30, 0x2a2a2f, 0x29292e, 0x28282d, 0x28282c, 0x27272b,
            0x26262b, 0x25252a, 0x242429, 0x242428, 0x232327, 0x222226, 0x212126, 0x202025,
            0x202024, 0x1f1f23, 0x1e1e22, 0x1d1d21, 0x1d1d20, 0x1c1c20, 0x1b1b1f, 0x1a1a1e,
            0x19191d, 0x19191c, 0x18181b, 0x17171a, 0x16161a, 0x161619, 0x151518, 0x141417,
            0x131316, 0x121215, 0x121214, 0x111114, 0x101013, 0x0f0f12, 0x0e0e11, 0x0e0e10,
            0x0d0d0f, 0x0c0c0e, 0x0b0b0e, 0x0b0b0d, 0x0a0a0c, 0x09090b,
        ]
        .into_iter()
        .map(colors::hex)
        .collect::<Vec<Color>>()
    });

    static CHAR_LIST: LazyLock<Vec<&str>> = LazyLock::new(|| {
        vec![
            "$", "@", "B", "%", "8", "&", "W", "M", "#", "*", "o", "a", "h", "k", "b", "d", "p",
            "q", "w", "m", "Z", "O", "0", "Q", "L", "C", "J", "U", "Y", "X", "z", "c", "v", "u",
            "n", "x", "r", "j", "f", "t", "/", "\\", "|", "(", ")", "1", "{", "}", "[", "]", "?",
            "-", "_", "+", "~", "<", ">", "i", "!", "l", "I", ";", ":", ",", "\"", "^", "`", "'",
            ".", " ",
        ]
    });

    pub const WIDTH: usize = 960;
    pub const HEIGHT: usize = 270;

    #[inline]
    fn color_map(input: &str) -> Color {
        let index = CHAR_LIST
            .iter()
            .enumerate()
            .find(|(_, x)| **x == input)
            .map_or(0, |(x, _)| x);
        COLOR[index]
    }

    #[inline]
    fn get_character(x: u16, y: u16) -> &'static str {
        let x = x as usize;
        let y = y as usize;
        let index = (y.min(HEIGHT) * WIDTH + x.min(WIDTH)).min(WIDTH * HEIGHT - 2);
        &NOVAK[index..=index]
    }

    impl Widget for RollingBackground {
        fn render(self, area: Rect, buf: &mut Buffer) {
            for y in 0..area.height {
                for x in 0..area.width {
                    if buf[(x, y)].symbol() != " " {
                        continue;
                    }
                    let c = get_character(x + area.x, y + area.y);
                    let col = color_map(c);
                    let style = Style::default().fg(col);
                    buf.set_string(x, y, c, style);
                }
            }
        }
    }
}

mod lolcat {
    use ratatui::{buffer::Buffer, layout::Rect, style::Color, widgets::Widget};

    pub struct Lolcat {
        pub frequency: f32,
        pub seed: f32,
        pub spread: f32,
        pub offset: f32,
    }

    impl Lolcat {
        fn get_color(&self, seed: f32) -> Color {
            let i = self.frequency * seed / self.spread + self.offset;
            let red = i.sin() * 127.00 + 128.00;
            let green = (i + (std::f32::consts::PI * 2.00 / 3.00)).sin() * 127.00 + 128.00;
            let blue = (i + (std::f32::consts::PI * 4.00 / 3.00)).sin() * 127.00 + 128.00;
            let (r, g, b) = (red as u8, green as u8, blue as u8);
            Color::Rgb(
                r.saturating_add(50),
                g.saturating_add(50),
                b.saturating_add(50),
            )
        }
    }

    impl Widget for Lolcat {
        fn render(self, area: Rect, buf: &mut Buffer) {
            for y in 0..area.height {
                let seed_of_lines = self.seed + y as f32;
                for x in 0..area.width {
                    if buf[(x, y)].symbol() == " " {
                        continue;
                    }
                    let seed_of_chars = seed_of_lines + x as f32;
                    buf[(x, y)].set_fg(self.get_color(seed_of_chars));
                }
            }
        }
    }
}
