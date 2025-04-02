use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use rand::{Rng, SeedableRng, thread_rng};
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout},
    style::{Color, Style, Stylize, palette::tailwind},
    widgets::{Block, Paragraph},
};
use tui_textarea::{CursorMove, TextArea};

use crate::app::{ascii, entry, math::inverse_lerp, time::TIME};

use super::Activity;

#[derive(Default)]
pub struct OobeActivity<'a> {
    pub should_exit: bool,
    pub should_skip: bool,
    pub render_menu: bool,
    app_time: Duration,
    phase_time: Duration,
    spam_interval: Duration,
    text_area: TextArea<'a>,
    random_char_table: Vec<char>,
    phase: u16,
    text_foctor: u16,
}

impl OobeActivity<'_> {
    pub fn new() -> Self {
        let char_table = vec![
            '☺', '☻', '♥', '♦', '♣', '♠', '•', '◘', '○', '◙', '♂', '♀', '♪', '♫', '☼', '►', '◄',
            '↕', '‼', '¶', '§', '▬', '↨', '↑', '↓', '→', '←', '∟', '↔', '▲', '▼', '!', '"', '#',
            '$', '%', '&', '\'', '(', ')', '*', '+', ',', '-', '.', '/', '0', '1', '2', '3', '4',
            '5', '6', '7', '8', '9', ':', ';', '<', '=', '>', '?', '@', 'A', 'B', 'C', 'D', 'E',
            'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V',
            'W', 'X', 'Y', 'Z', '[', '\\', ']', '^', '_', '`', 'a', 'b', 'c', 'd', 'e', 'f', 'g',
            'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x',
            'y', 'z', '{', '|', '}', '~', '⌂', 'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë',
            'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü',
            '¢', '£', '¥', '₧', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '⌐', '¬', '½',
            '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕', '╣', '║', '╗', '╝',
            '╜', '╛', '┐', '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦', '╠', '═',
            '╬', '╧', '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐',
            '▀', 'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
            '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
        ];
        let textarea_text = indoc::indoc! {"
            ===== 欢迎使用 玩家得分排名系统 =====
            .          制作：畅通无组

            请输入用户名：
        "};
        let mut text_area = TextArea::new(
            textarea_text
                .split('\n')
                .into_iter()
                .map(String::from)
                .collect(),
        );
        let style = Style::default().fg(Color::from_u32(0xffffff));
        text_area.set_style(style);
        text_area.set_cursor_line_style(style);
        text_area.move_cursor(CursorMove::Down);
        text_area.move_cursor(CursorMove::Down);
        text_area.move_cursor(CursorMove::Down);
        text_area.move_cursor(CursorMove::Down);
        Self {
            random_char_table: char_table,
            text_area,
            spam_interval: Duration::from_secs(3),
            ..Default::default()
        }
    }

    /// higher means more pixels per frame are modified in the animation
    const DRIP_SPEED: usize = 500;

    fn drip(&mut self, frame: &mut Frame<'_>, transition_to: Buffer) {
        let area = frame.area();
        let buf = frame.buffer_mut();

        // a seeded rng as we have to move the same random pixels each frame
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(10);
        let frame_count = self.phase_time.as_secs_f32() * entry::FPS as f32;
        let ramp_frames = 450.0;
        let fractional_speed = frame_count / ramp_frames;
        let variable_speed = Self::DRIP_SPEED as f32 * fractional_speed.powi(3);
        let pixel_count = (frame_count * variable_speed).floor() as usize;
        for _ in 0..pixel_count {
            let src_x = rng.gen_range(0..area.width);
            let src_y = rng.gen_range(1..area.height - 1);
            let src = {
                let index = buf.index_of(src_x, src_y);
                let ptr = buf.content.as_ptr();
                unsafe { &*ptr.add(index) }
            };
            // 1% of the time, move a blank or pixel (10:1) to the top line of the screen
            if rng.gen_ratio(1, 100) {
                let dest_x = rng
                    .gen_range(src_x.saturating_sub(5)..src_x.saturating_add(5))
                    .clamp(area.left(), area.right() - 1);
                let dest_y = area.top() + 1;

                let dest = &mut buf[(dest_x, dest_y)];
                dest.reset();
            } else {
                // move the pixel down one row
                let dest_x = src_x;
                let dest_y = src_y.saturating_add(1).min(area.bottom() - 1);
                // copy the cell to the new location
                buf[(dest_x, dest_y)].set_symbol(src.symbol());
                buf[(dest_x, dest_y)].set_skip(src.skip);
                buf[(dest_x, dest_y)].set_style(src.style());
            }
        }
        let mut white_count = 0;
        for row in 1..area.height - 1 {
            for col in 0..area.width {
                if buf[(col, row)].bg != Color::Reset {
                    white_count += 1;
                }
                if row == 1 {
                    buf[(col, 0)] = buf[(col, 1)].clone();
                }
            }
        }
        if white_count <= 1 {
            self.phase = 5;
        }

        for row in 0..area.height {
            for col in 0..area.width {
                if buf[(col, row)].bg == Color::Reset {
                    let transition_to = &transition_to[(col, row)];
                    buf[(col, row)].set_symbol(transition_to.symbol());
                    buf[(col, row)].set_skip(transition_to.skip);
                    buf[(col, row)].set_style(transition_to.style());
                }
            }
        }
    }

    fn new_line(&mut self, width: u16) {
        if self.text_foctor <= 10 {
            self.text_area.insert_str("请输入用户名：？\n");
        } else {
            let mut rng = thread_rng();
            let mut s = (0..width)
                .map(|_| {
                    let c = rng.gen_range(0..self.random_char_table.len());
                    self.random_char_table[c]
                })
                .collect::<String>();
            s.push('\n');
            self.text_area.insert_str(s);
            if self.phase == 1 {
                self.phase = 2;
            }
        }
    }

    fn draw_welcome(&self, frame: &mut Frame<'_>) {
        // 6 x 62
        let area = frame.area();
        let [_, vertical_center, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
        ])
        .areas(area);
        let [_, welcome_div, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(70),
            Constraint::Fill(1),
        ])
        .areas(vertical_center);
        let paragraph = Paragraph::new(ascii::welcome())
            .alignment(Alignment::Center)
            .block(Block::bordered())
            .fg(tailwind::WHITE)
            .bg(tailwind::INDIGO.c900);
        frame.render_widget(paragraph, welcome_div);
    }

    pub fn draw_oobe(&mut self, frame: &mut Frame<'_>, menu_buffer: Buffer) {
        let area = frame.area();

        if self.phase == 1 {
            if self.phase_time >= self.spam_interval {
                self.phase_time = Duration::default();
                self.spam_interval = (self.spam_interval / 3) * 2;
                self.new_line(area.width);
                self.text_foctor = self.text_foctor.saturating_add(1);
            }
        } else if self.phase == 2 {
            self.new_line(area.width);
            let progress = inverse_lerp(2.0..=4.0_f32, self.phase_time.as_secs_f32());
            let col = (progress * 255.0) as u8;
            let bg = Color::Rgb(col, col, col);
            self.text_area
                .set_style(Style::default().bg(bg).fg(tailwind::WHITE));
            if progress >= 1.0 && self.phase == 2 {
                self.phase = 3;
                self.render_menu = true;
                self.phase_time = Duration::default();
            }
        }

        if self.phase < 3 {
            frame.render_widget(&self.text_area, area);
        } else if self.phase == 3 {
            let block = Block::default().bg(tailwind::WHITE);
            frame.render_widget(block, area);
            if self.phase_time.as_secs() == 1 {
                self.phase = 4;
                self.phase_time = Duration::default();
            }
        } else if self.phase == 4 {
            let block = Block::default().bg(tailwind::WHITE);
            frame.render_widget(block, area);
            self.draw_welcome(frame);
            self.drip(frame, menu_buffer);
        }
    }
}

impl Activity for OobeActivity<'_> {
    fn draw(&mut self, _frame: &mut Frame<'_>) {}

    fn update(&mut self, event: Option<Event>) {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
            self.phase_time += time.delta;
        }

        if self.phase == 5 {
            self.should_skip = true;
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
        } else if key.code == KeyCode::Char(' ') || key.code == KeyCode::Char('s') {
            self.should_skip = true;
        } else if self.phase == 0 {
            self.phase = 1;
        }
    }
}
