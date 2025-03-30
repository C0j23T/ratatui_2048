/// A button widget from ratatui's example
///
/// https://github.com/ratatui/ratatui/blob/main/examples/apps/custom-widget/src/main.rs
pub mod button {
    use std::sync::LazyLock;

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

    pub static BLUE: LazyLock<Theme> = LazyLock::new(|| Theme {
        text: tailwind::BLUE.c50,
        background: tailwind::BLUE.c500,
        highlight: tailwind::BLUE.c400,
        shadow: tailwind::BLUE.c600,
    });

    impl<'a> Button<'a> {
        pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
            Button {
                label: label.into(),
                theme: *BLUE,
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
