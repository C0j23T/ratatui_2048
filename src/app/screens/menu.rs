use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use lolcat::Lolcat;
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Stylize, palette::tailwind},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use rolling_background::RollingBackground;

use crate::app::{
    ascii,
    gameplay::colors,
    math::{inverse_lerp, Interpolation},
    structs::Player,
    time::TIME,
    utils::{fade_in, rect_move},
};

use super::{Activity, AppState};

#[derive(Default)]
pub struct MenuActivity {
    pub exit: bool,
    player: Player,
    state: MenuState,
    selection: usize,
    selected_time: Duration,
    transition_time: Duration,

    app_time: Duration,

    bg_changed: bool,
    bg_rect_a: Rect,
    bg_rect_b: Rect,
}

#[derive(Default)]
pub enum MenuState {
    #[default]
    Login,
    Entering,
    Menu,
    Exiting,
}

impl MenuActivity {
    pub fn new() -> Self {
        Self {
            exit: false,
            selection: 2,
            state: MenuState::Menu,
            ..Default::default()
        }
    }

    pub fn set_player(&mut self, player: Player) {
        self.player = player;
    }

    fn draw_frame(&mut self, mut menu: Rect, frame: &mut Frame<'_>) {
        if matches!(self.state, MenuState::Entering) {
            let area = frame.area();
            let interpolation = Interpolation::PowOut { value: 5 };
            let mut progress = inverse_lerp(0.5..=1.5, self.transition_time.as_secs_f32());
            progress = interpolation.apply(progress);
            menu = rect_move(menu, area, progress)
        } else if matches!(self.state, MenuState::Exiting) {
            let area = frame.area();
            let interpolation = Interpolation::CircleOut;
            let mut progress = inverse_lerp(0.0..=1.0, self.transition_time.as_secs_f32());
            progress = interpolation.apply(progress);
            menu = rect_move(area, menu, progress)
        }

        frame.render_widget(
            Block::default().fg(tailwind::NEUTRAL.c700).bg(Color::Reset),
            menu,
        );

        let [menu_top] = Layout::vertical([Constraint::Length(1)]).areas(menu);
        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .fg(tailwind::INDIGO.c50);
        if matches!(self.state, MenuState::Menu) {
            block = block.title("─ Menu ");
        }
        frame.render_widget(block, menu_top);

        let [_, menu_bottom] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(menu);
        let mut block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
            .title_alignment(Alignment::Right)
            .fg(tailwind::INDIGO.c50);
        if matches!(self.state, MenuState::Menu) {
            block = block.title(" ( ↑ 或 ↓ ) 切换 | ( ⏎ ) 确定 ─");
        }
        frame.render_widget(block, menu_bottom);

        let [menu_left] = Layout::horizontal([Constraint::Length(1)]).areas(menu);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::BOTTOM | Borders::LEFT | Borders::TOP)
            .fg(tailwind::INDIGO.c50);
        frame.render_widget(block, menu_left);

        let [_, menu_right] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Length(1)]).areas(menu);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .borders(Borders::BOTTOM | Borders::RIGHT | Borders::TOP)
            .fg(tailwind::INDIGO.c50);
        frame.render_widget(block, menu_right);
    }

    fn draw_menu(&mut self, menu: Rect, frame: &mut Frame<'_>) {
        self.draw_frame(menu, frame);
        if !self.render_menu() {
            return;
        }

        let interpolation = Interpolation::ExpOut { value: 10.0 };
        let mut progress = inverse_lerp(0.0..=0.3_f32, self.selected_time.as_secs_f32());
        progress = 1.0 - interpolation.apply(progress);

        let lines = Layout::vertical([Constraint::Length(1)].repeat(9)).split(menu);
        let options = indoc::indoc! {"


            进入游戏
            账号登出
            删除玩家
            查找玩家
            编辑玩家
            查看世界排名
            退出
        "}
        .split('\n');
        let it = lines.into_iter().zip(options).enumerate();
        it.for_each(|(i, (rect, text))| {
            let flag = i > 1 && i == self.selection;
            let mut bg = if flag {
                let (r, g, b) = (238.0, 242.0, 255.0);
                let factor = (self.app_time.as_secs_f32() * 2.0).sin() * 0.25 + 0.5;

                Color::Rgb((r * factor) as u8, (g * factor) as u8, (b * factor) as u8)
            } else {
                Color::Reset
            };
            if matches!(self.state, MenuState::Entering) && flag {
                let factor = (self.app_time.as_secs_f32() * 100.0) as i32 % 2 == 0;
                if factor {
                    bg = tailwind::INDIGO.c50
                } else {
                    bg = tailwind::INDIGO.c900
                }
            }

            if flag {
                let [_, bg_rect, _] = Layout::horizontal([
                    Constraint::Length(1),
                    Constraint::Fill(1),
                    Constraint::Length(1),
                ])
                .areas(*rect);
                frame.render_widget(Clear, bg_rect);
                let block = Block::default().bg(bg);
                frame.render_widget(block, bg_rect);
            }

            let fg = if flag {
                if colors::brightness(bg) > 0.5 {
                    tailwind::INDIGO.c900
                } else {
                    tailwind::INDIGO.c50
                }
            } else {
                tailwind::INDIGO.c50
            };

            let text = if flag {
                let spaces = [' ']
                    .repeat((10.0 * progress) as usize)
                    .into_iter()
                    .collect::<String>();
                format!("->>{spaces} {text} {spaces}<<-")
            } else if text.is_empty() {
                String::new()
            } else {
                format!("=- {text} -=")
            };
            let width = unicode_width::UnicodeWidthStr::width_cjk(text.as_str());
            let [_, text_block, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(width as u16),
                Constraint::Fill(1),
            ])
            .flex(Flex::Center)
            .areas(*rect);
            let para = Paragraph::new(text).fg(fg).bg(bg);
            frame.render_widget(para, text_block);
        });
    }

    fn draw_bg(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let [_, title, _, _, bottom] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(13),
            Constraint::Max(3),
            Constraint::Length(11),
            Constraint::Max(1),
        ])
        .areas(area);

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

            let block = Block::bordered()
                .title_bottom(format!(" ID {}", self.player.id))
                .title_alignment(Alignment::Left)
                .borders(Borders::NONE)
                .fg(tailwind::INDIGO.c200);
            frame.render_widget(block, bottom);
        }

        // Background
        {
            let resized =
                area.width != self.bg_rect_a.width || area.height != self.bg_rect_a.height;
            if self.app_time.as_secs() % 5 == 0 || resized {
                if !self.bg_changed || resized {
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
            let progress = inverse_lerp(0.0..=10.0, self.app_time.as_secs_f32() % 10.0);
            let rect = rect_move(self.bg_rect_a, self.bg_rect_b, progress);
            frame.render_widget(RollingBackground, rect);
        }
    }

    pub fn exiting_activity(&mut self) {
        self.state = MenuState::Exiting;
        self.transition_time = Duration::default();
    }

    pub fn can_enter_another_activity(&self) -> bool {
        self.transition_time.as_secs_f32() >= 1.6
    }

    pub fn next_state(&self) -> Option<AppState> {
        if !matches!(self.state, MenuState::Entering) {
            return None;
        }
        match self.selection {
            2 => Some(AppState::Gameplay),
            3 => Some(AppState::SwitchPlayer),
            4 => Some(AppState::RemovePlayer),
            5 => Some(AppState::FindPlayer),
            6 => Some(AppState::EditPlayer),
            7 => Some(AppState::ListAllPlayer),
            _ => None,
        }
    }

    fn render_menu(&self) -> bool {
        matches!(self.state, MenuState::Menu)
            || matches!(self.state, MenuState::Entering)
                && self.transition_time.as_secs_f32() <= 0.5
            || matches!(self.state, MenuState::Exiting) && self.transition_time.as_secs_f32() >= 1.1
    }
}

impl Activity for MenuActivity {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.draw_bg(frame);

        if !matches!(self.state, MenuState::Login) {
            let area = frame.area();
            let [_, _, _, menu, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(13),
                Constraint::Max(3),
                Constraint::Length(11),
                Constraint::Max(1),
            ])
            .areas(area);
            let [_, menu, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Percentage(40),
                Constraint::Fill(1),
            ])
            .flex(Flex::Center)
            .areas(menu);
            self.draw_menu(menu, frame);
        } else {
        }

        fade_in(frame, 2.0, self.app_time.as_secs_f32(), Some(114514));
    }

    fn update(&mut self, event: Option<Event>) {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
            self.selected_time += time.delta;
            self.transition_time += time.delta;
        }

        if matches!(self.state, MenuState::Exiting) && self.transition_time.as_secs_f32() >= 1.1 {
            self.state = MenuState::Menu;
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
        if !matches!(self.state, MenuState::Menu) {
            return;
        }
        match key.code {
            KeyCode::Up => {
                self.selection -= 1;
                self.selected_time = Duration::default();
                if self.selection < 2 {
                    self.selection = 8;
                }
            }
            KeyCode::Down => {
                self.selection += 1;
                self.selected_time = Duration::default();
                if self.selection > 8 {
                    self.selection = 2;
                }
            }
            KeyCode::Enter => {
                self.transition_time = Duration::default();
                self.state = MenuState::Entering;

                if self.selection == 8 {
                    self.exit = true;
                }
            }
            _ => (),
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

    use crate::app::ascii::NOVAK;

    pub struct RollingBackground;

    static COLOR: LazyLock<Vec<Color>> = LazyLock::new(|| {
        [
            0x2E1065, 0x2E1266, 0x2E1467, 0x2D1668, 0x2D1869, 0x2D196A, 0x2D1B6C, 0x2D1D6D,
            0x2C1F6E, 0x2C216F, 0x2C2370, 0x2C2571, 0x2C2772, 0x2C2973, 0x2B2B74, 0x2B2C75,
            0x2B2E76, 0x2B3077, 0x2B3279, 0x2A347A, 0x2A367B, 0x2A387C, 0x2A3A7D, 0x2A3C7E,
            0x293E7F, 0x293F80, 0x294181, 0x294382, 0x294583, 0x294785, 0x284986, 0x284B87,
            0x284D88, 0x284F89, 0x28518A, 0x27528B, 0x27548C, 0x27568D, 0x27588E, 0x275A8F,
            0x265C90, 0x265E92, 0x266093, 0x266294, 0x266495, 0x266596, 0x256797, 0x256998,
            0x256B99, 0x256D9A, 0x256F9B, 0x24719C, 0x24739E, 0x24759F, 0x2477A0, 0x2478A1,
            0x237AA2, 0x237CA3, 0x237EA4, 0x2380A5, 0x2382A6, 0x2384A7, 0x2286A8, 0x2288A9,
            0x228AAB, 0x228BAC, 0x228DAD, 0x218FAE, 0x2191AF, 0x2193B0,
        ]
        .into_iter()
        .map(Color::from_u32)
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
            for y in area.y..area.y + area.height {
                let seed_of_lines = self.seed + y as f32;
                for x in area.x..area.x + area.width {
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
