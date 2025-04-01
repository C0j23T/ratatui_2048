use std::{
    collections::VecDeque,
    sync::{Arc, LazyLock, RwLock, atomic::AtomicI8},
    time::Duration,
};

use button::*;
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Style, Stylize, palette::tailwind},
    widgets::{Block, BorderType, Clear, Paragraph},
};

use crate::app::{
    math::{Interpolation, inverse_lerp},
    time::TIME,
    utils::rect_scale,
};

pub static DIALOG_MANAGER: LazyLock<RwLock<DialogManager>> =
    LazyLock::new(|| RwLock::new(DialogManager::new()));

pub struct DialogManager {
    queue: VecDeque<Dialog>,
    active: Option<Dialog>,
}

pub struct Dialog {
    title: String,
    content: String,
    buttons: Vec<String>,
    callback: Arc<AtomicI8>,
    content_alignment: Alignment,
    warp: bool,

    hover: i8,
    active: i8,
    rects: Vec<Rect>,
    return_at_next_frame: bool,
    duration: Duration,
}
impl Dialog {
    pub fn new(
        title: &str,
        content: &str,
        content_alignment: Alignment,
        warp: bool,
        mut buttons: Vec<String>,
        callback: Arc<AtomicI8>,
    ) -> Self {
        if buttons.is_empty() {
            buttons = vec![String::from("确定")];
        }
        Self {
            hover: buttons.len() as i8 - 1,
            title: title.to_string(),
            content: content.to_string(),
            content_alignment,
            warp,
            buttons: buttons.iter().take(3).cloned().collect(),
            callback,
            active: -1,
            return_at_next_frame: false,
            rects: Vec::new(),
            duration: Duration::default(),
        }
    }
}

impl DialogManager {
    fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            active: None,
        }
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        if self.active.is_none() && !self.queue.is_empty() {
            self.active = self.queue.pop_front();
        }
        let Some(ref mut dialog) = self.active else {
            return;
        };
        {
            let time = TIME.read().unwrap();
            dialog.duration += time.delta;
        }
        let col = Layout::horizontal(vec![Constraint::Percentage(75)])
            .flex(Flex::Center)
            .split(frame.area());

        let mut paragraph = Paragraph::new(dialog.content.as_str())
            .bg(tailwind::SKY.c600)
            .fg(tailwind::SKY.c50)
            .block(
                Block::bordered()
                    .border_style(Style::new().bg(tailwind::SKY.c600).fg(tailwind::SKY.c50))
                    .border_type(BorderType::Rounded)
                    .title(dialog.title.as_str()),
            )
            .alignment(dialog.content_alignment);
        if dialog.warp {
            paragraph = paragraph.wrap(ratatui::widgets::Wrap { trim: false });
        }
        let height = paragraph.line_count(col[0].width);
        let window = Layout::vertical(vec![Constraint::Length(height as u16 + 5)])
            .flex(Flex::Center)
            .split(col[0]);

        let progress = inverse_lerp(0.0..=0.6_f32, dialog.duration.as_secs_f32());
        let interpolation = Interpolation::SwingOut;
        let window = rect_scale(window[0], interpolation.apply(progress));
        frame.render_widget(Clear, window);
        frame.render_widget(paragraph, window);

        // 按钮
        let [_, _, buttons, _] = Layout::vertical([
            Constraint::Length(height as u16),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .areas(window);
        let [_, buttons, _] = Layout::horizontal([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .flex(Flex::Center)
        .areas(buttons);
        let rects = {
            let mut consts = vec![Constraint::Fill(1)];
            dialog.buttons.iter().for_each(|_| {
                consts.push(Constraint::Percentage(25));
                consts.push(Constraint::Length(1));
            });
            consts.pop();
            consts.push(Constraint::Fill(1));

            Layout::horizontal(consts).flex(Flex::Center).split(buttons)
        };
        let rects = dialog
            .buttons
            .iter()
            .enumerate()
            .map(|(i, _)| rects[(i + 1) * 2 - 1])
            .collect::<Vec<_>>();
        dialog.rects = rects.clone();
        rects
            .into_iter()
            .zip(&dialog.buttons)
            .enumerate()
            .map(|(i, (rect, line))| {
                let mut button = Button::new(line.as_str()).theme(BLUE);
                let i = i as i8;
                if i == dialog.active {
                    button = button.state(ButtonState::Active);
                } else if i == dialog.hover {
                    button = button.state(ButtonState::Selected);
                }
                (rect, button)
            })
            .for_each(|x| frame.render_widget(x.1, x.0));
    }

    pub fn update_input(&mut self, event: Event) {
        {
            let Some(ref dialog) = self.active else {
                return;
            };
            if dialog.return_at_next_frame {
                dialog
                    .callback
                    .store(dialog.active, std::sync::atomic::Ordering::Relaxed);
                self.active = None;
                return;
            }
        }
        match event {
            Event::Key(key) => self.handle_keyboard(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            _ => (),
        }
    }

    fn handle_keyboard(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }
        let Some(ref mut dialog) = self.active else {
            return;
        };

        match key.code {
            KeyCode::Left => dialog.hover = (dialog.hover - 1).max(0),
            KeyCode::Right => dialog.hover = (dialog.hover + 1).min(dialog.buttons.len() as i8 - 1),
            KeyCode::Char(' ') | KeyCode::Enter => {
                dialog.active = dialog.hover;
                dialog.return_at_next_frame = true;
            }
            _ => (),
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        let Some(ref mut dialog) = self.active else {
            return;
        };
        let position = Rect::new(mouse.column, mouse.row, 1, 1);
        match mouse.kind {
            MouseEventKind::Moved => {
                if let Some((i, _)) = dialog
                    .rects
                    .iter()
                    .enumerate()
                    .find(|(_, rect)| rect.intersects(position))
                {
                    dialog.hover = i as i8;
                } else {
                    dialog.hover = -1;
                }
            }
            MouseEventKind::Down(MouseButton::Left) => {
                if dialog.hover < 0 {
                    return;
                }
                dialog.active = dialog.hover;
                dialog.return_at_next_frame = true;
            }
            _ => (),
        }
    }

    pub fn has_dialog(&self) -> bool {
        self.active.is_some()
    }

    pub fn push(&mut self, dialog: Dialog) {
        self.queue.push_back(dialog);
    }
}

/// A button widget from ratatui's example
///
/// https://github.com/ratatui/ratatui/blob/main/examples/apps/custom-widget/src/main.rs
pub mod button {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        style::{Color, Style, palette::tailwind},
        text::Line,
        widgets::Widget,
    };

    pub struct Button<'a> {
        label: Line<'a>,
        theme: Theme,
        state: ButtonState,
    }

    #[derive(PartialEq, Eq)]
    pub enum ButtonState {
        Normal,
        Selected,
        Active,
    }

    #[derive(Clone, Copy)]
    pub struct Theme {
        text: Color,
        background: Color,
        highlight: Color,
        shadow: Color,
    }

    pub const BLUE: Theme = Theme {
        text: tailwind::BLUE.c50,
        background: tailwind::BLUE.c500,
        highlight: tailwind::BLUE.c400,
        shadow: tailwind::BLUE.c600,
    };

    impl<'a> Button<'a> {
        pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
            Button {
                label: label.into(),
                theme: BLUE,
                state: ButtonState::Normal,
            }
        }

        pub const fn theme(mut self, theme: Theme) -> Self {
            self.theme = theme;
            self
        }

        pub const fn state(mut self, state: ButtonState) -> Self {
            self.state = state;
            self
        }
    }

    impl Button<'_> {
        const fn colors(&self) -> (Color, Color, Color, Color) {
            let theme = &self.theme;
            match self.state {
                ButtonState::Normal => {
                    (theme.background, theme.text, theme.shadow, theme.highlight)
                }
                ButtonState::Selected => {
                    (theme.highlight, theme.text, theme.shadow, theme.highlight)
                }
                ButtonState::Active => {
                    (theme.background, theme.text, theme.highlight, theme.shadow)
                }
            }
        }
    }

    impl Widget for Button<'_> {
        fn render(self, area: Rect, buf: &mut Buffer) {
            let (background, text, shadow, highlight) = self.colors();
            buf.set_style(area, Style::new().bg(background).fg(text));

            // render top line if there's enough space
            if area.height > 2 {
                buf.set_string(
                    area.x,
                    area.y,
                    "▔".repeat(area.width as usize),
                    Style::new().fg(highlight).bg(background),
                );
            }
            // render bottom line if there's enough space
            if area.height > 1 {
                buf.set_string(
                    area.x,
                    area.y + area.height - 1,
                    "▁".repeat(area.width as usize),
                    Style::new().fg(shadow).bg(background),
                );
            }
            // render label centered
            buf.set_line(
                area.x + (area.width.saturating_sub(self.label.width() as u16)) / 2,
                area.y + (area.height.saturating_sub(1)) / 2,
                &self.label,
                area.width,
            );
        }
    }
}
