use std::{io::Result, time::Duration};

use chrono::TimeZone;
use crossterm::event::{self, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame, Terminal,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    prelude::Backend,
    style::{Color, Style, Stylize, palette::tailwind},
    text::Text,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};

use crate::app::{math::inverse_lerp, structs::Player, time::TIME};

const ITEM_HEIGHT: usize = 1;

#[derive(Default)]
pub struct RankingActivity {
    itoa_buffer: itoa::Buffer,
    save: Player,
    players: Vec<Player>,
    app_time: Duration,
    by_score: bool,

    show_items: Vec<Player>,
    state: TableState,
    longest_item_lens: (u16, u16, u16),
    scroll_state: ScrollbarState,

    pub exit: bool,
}

impl RankingActivity {
    pub fn new(players: Vec<Player>) -> Self {
        let mut this = Self {
            players,
            ..Default::default()
        };
        this.state.select(Some(0));
        this
    }

    pub fn reset(&mut self) {
        self.exit = false;
        self.save = Default::default();
        self.app_time = Default::default();
    }

    pub fn set_save(&mut self, save: Player) {
        self.save = save;
        self.show_items = self.players.clone();
        self.show_items.push(self.save.clone());
        self.constrant_len();
        self.scroll_state = self
            .scroll_state
            .content_length(self.show_items.len() * ITEM_HEIGHT);
    }

    pub fn by_score(&mut self) {
        self.show_items.sort_by(|a, b| b.cmp(a));
        self.by_score = true;
        self.set_row(
            self.show_items
                .iter()
                .enumerate()
                .find(|(_, x)| x.name == self.save.name)
                .map_or(0, |x| x.0),
        );
    }

    #[allow(dead_code)]
    pub fn by_timestamp(&mut self) {
        self.show_items
            .sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        self.by_score = false;
        self.set_row(
            self.show_items
                .iter()
                .enumerate()
                .find(|(_, x)| x.name == self.save.name)
                .map_or(0, |x| x.0),
        );
    }

    fn set_row(&mut self, index: usize) {
        self.state.select(Some(index));
        self.scroll_state = self.scroll_state.position(index * ITEM_HEIGHT);
    }

    fn next_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.show_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn prev_row(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.show_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    fn reset_row(&mut self) {
        self.state.select(Some(0));
        self.scroll_state = self.scroll_state.position(0);
    }

    fn constrant_len(&mut self) {
        let name_len = self
            .show_items
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(x.name.as_str()))
            .max()
            .unwrap_or(0) as u16;
        let score_len = self
            .show_items
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(self.itoa_buffer.format(x.score)))
            .max()
            .unwrap_or(0) as u16;
        let time_len = self
            .show_items
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(self.itoa_buffer.format(x.time)))
            .max()
            .unwrap_or(0) as u16;
        self.longest_item_lens = (name_len, score_len, time_len);
    }

    pub fn update<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
        }

        terminal.draw(|frame| {
            self.draw(frame);
            self.fade_in(frame);
        })?;

        self.update_input()?;

        Ok(())
    }

    pub fn update_input(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(20))? {
            let event = event::read()?;

            let event::Event::Key(key) = event else {
                return Ok(());
            };

            if matches!(key.code, KeyCode::Char('q')) || matches!(key.code, KeyCode::Esc) {
                self.exit = true;
            }

            if key.kind != KeyEventKind::Press {
                return Ok(());
            }
            let ctrl_pressed = key.modifiers.contains(KeyModifiers::CONTROL);
            match key.code {
                KeyCode::Up => {
                    if ctrl_pressed {
                        self.reset_row()
                    } else {
                        self.prev_row()
                    }
                }
                KeyCode::Down => self.next_row(),
                _ => (),
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let [table, footer] =
            Layout::vertical([Constraint::Min(3), Constraint::Length(3)]).areas(area);

        self.render_table(frame, table);
        self.render_scrollbar(frame, table);

        self.render_footer(frame, footer);
    }

    pub fn render_footer(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let header = Paragraph::new("(Q) 退出 | (↓) 向下移动 | (↑) 向上移动 | (Ctrl + ↑) 回到顶部")
            .fg(tailwind::INDIGO.c100)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .fg(tailwind::INDIGO.c300),
            )
            .alignment(Alignment::Center);
        frame.render_widget(header, area);
    }

    pub fn render_table(&mut self, frame: &mut Frame<'_>, area: Rect) {
        let header = ["排名", "名称", "分数", "所用时间", "达成时间"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .style(
                Style::default()
                    .fg(tailwind::INDIGO.c50)
                    .bg(tailwind::INDIGO.c700),
            )
            .height(1);
        let len = self.show_items.len();
        let rows = self.show_items.iter().enumerate().map(|(i, data)| {
            let bg = Color::Rgb(
                (99.0 * (1.0 - i as f32 / len as f32)) as u8,
                (102.0 * (1.0 - i as f32 / len as f32)) as u8,
                (241.0 * (1.0 - i as f32 / len as f32)) as u8,
            );

            let fg = if self.by_score {
                match i {
                    0 => tailwind::AMBER.c200,
                    1 => tailwind::NEUTRAL.c300,
                    2 => tailwind::YELLOW.c500,
                    _ => tailwind::INDIGO.c50,
                }
            } else {
                tailwind::INDIGO.c50
            };

            let time = (chrono::Utc.timestamp_millis_opt(data.timestamp).unwrap()
                + chrono::Duration::hours(8))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

            [
                Cell::from(format!("#{}", i + 1)),
                Cell::from(data.name.as_str()),
                Cell::from(self.itoa_buffer.format(data.score).to_string()),
                Cell::from(self.itoa_buffer.format(data.time).to_string()),
                Cell::from(time),
            ]
            .into_iter()
            .collect::<Row>()
            .style(Style::new().fg(fg).bg(bg))
            .height(ITEM_HEIGHT as u16)
        });
        let bar = " > ";
        let t = Table::new(
            rows,
            [
                Constraint::Min(7),
                Constraint::Min(self.longest_item_lens.0 + 1),
                Constraint::Min(self.longest_item_lens.1 + 1),
                Constraint::Min(self.longest_item_lens.2 + 1),
                Constraint::Min(22),
            ],
        )
        .header(header)
        .row_highlight_style(Style::new().bg(tailwind::INDIGO.c400))
        .column_highlight_style(Style::new().bg(tailwind::INDIGO.c400))
        .cell_highlight_style(Style::new().bg(tailwind::INDIGO.c400))
        .highlight_symbol(Text::from(vec![bar.into(), "".into()]))
        .bg(tailwind::INDIGO.c700)
        .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(t, area, &mut self.state);
    }

    pub fn render_scrollbar(&mut self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    pub fn fade_in(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();
        let progress = inverse_lerp(0.0..=0.5_f32, self.app_time.as_secs_f32())
            .unwrap_or(1.0)
            .min(1.0);
        let buf = frame.buffer_mut();
        for row in area.rows() {
            for col in row.columns() {
                let cell = &mut buf[(col.x, col.y)];
                if let Color::Rgb(r, g, b) = cell.fg {
                    cell.fg = Color::Rgb(
                        (r as f32 * progress) as u8,
                        (g as f32 * progress) as u8,
                        (b as f32 * progress) as u8,
                    );
                }
                if let Color::Rgb(r, g, b) = cell.bg {
                    cell.bg = Color::Rgb(
                        (r as f32 * progress) as u8,
                        (g as f32 * progress) as u8,
                        (b as f32 * progress) as u8,
                    );
                }
            }
        }
    }
}
