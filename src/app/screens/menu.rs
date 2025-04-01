use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyEventKind};
use lolcat::Lolcat;
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Color, Modifier, Style, Styled, Stylize, palette::tailwind},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use rolling_background::RollingBackground;
use tui_textarea::TextArea;

use crate::{
    app::{
        ascii,
        gameplay::colors,
        math::{Interpolation, inverse_lerp},
        structs::Player,
        time::TIME,
        utils::{fade_in, rect_move},
    },
    data_manager,
};

use super::{
    Activity, AppState,
    dialog::{DIALOG_MANAGER, Dialog},
};

#[derive(Default)]
pub struct MenuActivity<'a> {
    pub exit: bool,
    player: Player,
    player_requested: bool,
    state: MenuState<'a>,
    focus: usize,
    selected_time: Duration,
    transition_time: Duration,

    app_time: Duration,

    bg_changed: bool,
    bg_rect_a: Rect,
    bg_rect_b: Rect,
}

pub enum MenuState<'a> {
    Login {
        username: TextArea<'a>,
        password: TextArea<'a>,
        confirm: TextArea<'a>,
        logged_in: bool,
        focus: u16,
        register: bool,
        login_pressed: bool,
    },
    Entering,
    Menu,
    Exiting,
}

impl Default for MenuState<'_> {
    fn default() -> Self {
        let mut username = TextArea::default();
        let fg = tailwind::ZINC.c500;
        let style = Style::default().fg(fg);
        username.set_style(style);
        username.set_cursor_line_style(style);
        username.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(fg)
                .title("─ 用户名 / ID "),
        );
        let mut password = username.clone();
        password.set_mask_char('\u{2022}');
        password.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(fg)
                .title("─ 密码 "),
        );
        let mut confirm = password.clone();
        confirm.set_block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(fg)
                .title("─ 确认密码 "),
        );

        Self::Login {
            username,
            password,
            confirm,
            logged_in: false,
            focus: 0,
            register: false,
            login_pressed: false,
        }
    }
}

impl MenuActivity<'_> {
    pub fn new() -> Self {
        Self {
            exit: false,
            focus: 2,
            ..Default::default()
        }
    }

    fn draw_frame(&self, mut menu: Rect, animate_to: Option<Rect>, frame: &mut Frame<'_>) {
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
        } else if matches!(
            self.state,
            MenuState::Login {
                logged_in: true,
                ..
            }
        ) {
            let interpolation = Interpolation::PowOut { value: 5 };
            let mut progress = inverse_lerp(0.5..=1.5, self.transition_time.as_secs_f32());
            progress = interpolation.apply(progress);
            menu = rect_move(menu, animate_to.unwrap(), progress)
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
            block = block.title(" ( ↑ ↓ ) 切换 | ( ⏎ ) 确定 ─");
        }
        if matches!(
            self.state,
            MenuState::Login {
                logged_in: false,
                ..
            }
        ) {
            block = block.title(" ( ← ↑ ↓ → ) 切换 | ( ⏎ ) 确定 | ( ESC ) 退出 ─");
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
        self.draw_frame(menu, None, frame);
        if !self.render_menu() {
            return;
        }

        let interpolation = Interpolation::PowOut { value: 10 };
        let mut progress = inverse_lerp(0.0..=0.8_f32, self.selected_time.as_secs_f32());
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
        let it = lines.iter().zip(options).enumerate();
        it.for_each(|(i, (rect, text))| {
            let flag = i > 1 && i == self.focus;
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
            let [_, text_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(width as u16),
                Constraint::Fill(1),
            ])
            .flex(Flex::Center)
            .areas(*rect);
            let para = Paragraph::new(text).fg(fg).bg(bg);
            frame.render_widget(para, text_area);
        });
    }

    fn draw_bg(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let [_, title, _, _, bottom] = Layout::vertical([
            Constraint::Fill(2),
            Constraint::Length(13),
            Constraint::Min(1),
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
                    let motion = if rng.random_bool(0.5) {
                        -length
                    } else {
                        length
                    };
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
        self.player_requested = false;
    }

    pub fn can_enter_another_activity(&self) -> bool {
        self.transition_time.as_secs_f32() >= 1.6
    }

    pub fn next_state(&self) -> Option<AppState> {
        if !matches!(self.state, MenuState::Entering) {
            return None;
        }
        match self.focus {
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

    fn draw_login(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let divs = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(13),
            Constraint::Max(3),
            Constraint::Length(11),
            Constraint::Max(1),
        ])
        .split(area);
        let [_, panel, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Percentage(60),
            Constraint::Fill(1),
        ])
        .flex(Flex::Center)
        .areas(divs[3]);

        let MenuState::Login {
            ref username,
            ref password,
            ref confirm,
            logged_in,
            focus,
            register,
            ..
        } = self.state
        else {
            return;
        };
        if logged_in {
            let [_, menu, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Percentage(40),
                Constraint::Fill(1),
            ])
            .flex(Flex::Center)
            .areas(divs[3]);
            self.draw_frame(panel, Some(menu), frame);
            return;
        }
        self.draw_frame(panel, None, frame);

        let divs = Layout::horizontal([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(2),
        ])
        .split(panel);
        let [_, usr, pwd, bottom] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(divs[1]);
        let [pwd_small, cfm] =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(pwd);

        frame.render_widget(Clear, usr);
        frame.render_widget(username, usr);

        if register {
            frame.render_widget(Clear, pwd);
            frame.render_widget(password, pwd_small);
            frame.render_widget(confirm, cfm);
        } else {
            frame.render_widget(Clear, pwd);
            frame.render_widget(password, pwd);
        }

        let block = Block::default().fg(tailwind::INDIGO.c50);
        match focus {
            0 => frame.render_widget(block, usr),
            1 => {
                if register {
                    frame.render_widget(block, pwd_small)
                } else {
                    frame.render_widget(block, pwd)
                }
            }
            2 => frame.render_widget(block, cfm),
            _ => (),
        }

        let btn_text = if !register {
            ["登录", "注册账号"]
        } else {
            ["注册", "继续登录"]
        };
        let [_, reg, _] = Layout::horizontal([
            Constraint::Max(1),
            Constraint::Length(4),
            Constraint::Fill(1),
        ])
        .areas(bottom);
        let register_btn = Paragraph::new(btn_text[0]).set_style(
            Style::new()
                .fg(if focus == 3 {
                    tailwind::INDIGO.c50
                } else {
                    tailwind::ZINC.c500
                })
                .add_modifier(Modifier::UNDERLINED),
        );
        frame.render_widget(register_btn, reg);

        let [_, log, _] = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Max(1),
        ])
        .areas(bottom);
        let login_btn = Paragraph::new(btn_text[1]).set_style(
            Style::new()
                .fg(if focus == 4 {
                    tailwind::INDIGO.c50
                } else {
                    tailwind::ZINC.c500
                })
                .add_modifier(Modifier::UNDERLINED),
        );
        frame.render_widget(login_btn, log);
    }
}

impl Activity for MenuActivity<'_> {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.draw_bg(frame);

        if !matches!(self.state, MenuState::Login { .. }) {
            let area = frame.area();
            let divs = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(13),
                Constraint::Max(3),
                Constraint::Length(11),
                Constraint::Max(1),
            ])
            .split(area);
            let [_, menu, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Percentage(40),
                Constraint::Fill(1),
            ])
            .flex(Flex::Center)
            .areas(divs[3]);
            self.draw_menu(menu, frame);
        } else {
            self.draw_login(frame);
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

        if let MenuState::Login {
            ref username,
            ref password,
            ref mut logged_in,
            register,
            ref mut login_pressed,
            ..
        } = self.state
        {
            if !*logged_in && *login_pressed {
                if register {
                    if let Some(x) = data_manager!(
                        register_account,
                        username.lines()[0].clone(),
                        password.lines()[0].clone()
                    ) {
                        *login_pressed = false;
                        if let Some(x) = x {
                            self.player = x;
                            *logged_in = true;
                        } else {
                            let mut dialog_manger = DIALOG_MANAGER.write().unwrap();
                            dialog_manger.push(Dialog::new(
                                " 无法注册 ",
                                "在注册时遇到问题",
                                Alignment::Left,
                                false,
                                vec![String::from("确定")],
                                None,
                            ));
                        }
                    }
                } else if let Some(x) = data_manager!(
                    verify_account,
                    username.lines()[0].clone(),
                    password.lines()[0].clone()
                ) {
                    *login_pressed = false;
                    if let Some(x) = x {
                        self.player = x;
                        *logged_in = true;
                    } else {
                        let mut dialog_manger = DIALOG_MANAGER.write().unwrap();
                        dialog_manger.push(Dialog::new(
                            " 登录失败 ",
                            "账号或密码错误",
                            Alignment::Left,
                            false,
                            vec![String::from("确定")],
                            None,
                        ));
                    }
                }

                if *logged_in {
                    self.transition_time = Duration::default();
                }
            }

            if *logged_in && self.transition_time.as_secs_f32() > 1.5 {
                self.state = MenuState::Menu;
                self.transition_time = Duration::default();
            }
        }

        if !self.player_requested {
            if let Some(player) = data_manager!(get_current_player) {
                self.player_requested = true;
                self.player = player;
            }
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
        if matches!(self.state, MenuState::Menu) {
            match key.code {
                KeyCode::Up => {
                    self.focus -= 1;
                    self.selected_time = Duration::default();
                    if self.focus < 2 {
                        self.focus = 8;
                    }
                }
                KeyCode::Down => {
                    self.focus += 1;
                    self.selected_time = Duration::default();
                    if self.focus > 8 {
                        self.focus = 2;
                    }
                }
                KeyCode::Enter => {
                    self.transition_time = Duration::default();
                    self.state = MenuState::Entering;

                    if self.focus == 8 {
                        self.exit = true;
                    }
                }
                KeyCode::Char('q') | KeyCode::Esc => {
                    self.exit = true;
                }
                _ => (),
            }
        } else if let MenuState::Login {
            ref mut username,
            ref mut password,
            ref mut confirm,
            ref mut focus,
            ref mut register,
            ref mut login_pressed,
            ..
        } = self.state
        {
            let f = *focus;
            match key.code {
                KeyCode::Up => {
                    if f == 2 {
                        *focus = 0;
                    } else if f == 0 {
                        *focus = 3;
                    } else if f == 4 {
                        if *register {
                            *focus = 2;
                        } else {
                            *focus = 1;
                        }
                    } else if f == 3 {
                        *focus = 1;
                    } else {
                        *focus -= 1;
                    }
                }
                KeyCode::Down => {
                    if f == 3 || f == 4 {
                        *focus = 0;
                    } else if f == 1 {
                        *focus = 3;
                    } else if f == 2 {
                        *focus = 4;
                    } else {
                        *focus += 1;
                    }
                }
                KeyCode::Left => {
                    if f == 3 {
                        *focus = 4;
                    } else if f == 4 {
                        *focus = 3;
                    } else if *register {
                        if f == 1 {
                            *focus = 2;
                        } else if f == 2 {
                            *focus = 1;
                        }
                    }
                }
                KeyCode::Right => {
                    if f == 3 {
                        *focus = 4;
                    } else if f == 4 {
                        *focus = 3;
                    } else if *register {
                        if f == 1 {
                            *focus = 2;
                        } else if f == 2 {
                            *focus = 1;
                        }
                    }
                }
                KeyCode::Enter => {
                    if f == 4 {
                        *register = !*register;
                        *focus = 0;
                    }
                    if f == 3 {
                        let mut dialog_manger = DIALOG_MANAGER.write().unwrap();
                        if username.lines()[0].is_empty() || password.lines()[0].is_empty() {
                            dialog_manger.push(Dialog::new(
                                " 用户名 / 密码 不能为空 ",
                                "请填写用户名和密码",
                                Alignment::Left,
                                false,
                                vec![String::from("确定")],
                                None,
                            ));
                            return;
                        }
                        if *register && password.lines() != confirm.lines() {
                            dialog_manger.push(Dialog::new(
                                " 两次密码输入不一致 ",
                                "两次密码输入不一致",
                                Alignment::Left,
                                false,
                                vec![String::from("确定")],
                                None,
                            ));
                            return;
                        }
                        *login_pressed = true;
                    }
                }
                _ => match focus {
                    0 => {
                        username.input(key);
                    }
                    1 => {
                        password.input(key);
                    }
                    2 => {
                        confirm.input(key);
                    }
                    _ => {
                        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                            self.exit = true;
                        }
                    }
                },
            }
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
