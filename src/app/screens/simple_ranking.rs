use std::time::Duration;

use chrono::TimeZone;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize, palette::tailwind},
    text::Text,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};

use crate::{
    app::{structs::Player, time::TIME, utils::fade_in},
    data_manager,
};

use super::Activity;

const ITEM_HEIGHT: usize = 1;

#[derive(Default)]
pub struct RankingActivity {
    itoa_buffer: itoa::Buffer,
    save: Player,
    players_requested: bool,
    app_time: Duration,

    show_items: Vec<Player>,
    state: TableState,
    longest_item_lens: (u16, u16, u16),
    scroll_state: ScrollbarState,

    pub should_exit: bool,
}

impl RankingActivity {
    pub fn new() -> Self {
        let mut this = Self::default();
        this.state.select(Some(0));
        this
    }

    pub fn reset(&mut self) {
        self.should_exit = false;
        self.save = Default::default();
        self.app_time = Default::default();
    }

    pub fn set_save(&mut self, save: Player) {
        self.save = save;
        self.show_items.clear();
        self.show_items.push(self.save.clone());
        self.players_requested = false;
    }

    pub fn by_score(&mut self) {
        self.show_items.sort_by(|a, b| b.cmp(a));
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
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(self.itoa_buffer.format(x.best_score)))
            .max()
            .unwrap_or(0) as u16;
        let time_len = self
            .show_items
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(self.itoa_buffer.format(x.best_time)))
            .max()
            .unwrap_or(0) as u16;
        self.longest_item_lens = (name_len, score_len, time_len);
    }

    fn update_input(&mut self, event: Event) {
        let event::Event::Key(key) = event else {
            return;
        };

        if matches!(key.code, KeyCode::Char('q')) || matches!(key.code, KeyCode::Esc) {
            self.should_exit = true;
        }

        if key.kind != KeyEventKind::Press {
            return;
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

    pub fn draw_ranking(&mut self, frame: &mut Frame<'_>) {
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
        let rows = self.show_items.iter().enumerate().map(|(i, data)| {
            let bg = Color::Reset;

            let fg = match i {
                0 => tailwind::AMBER.c200,
                1 => tailwind::NEUTRAL.c300,
                2 => tailwind::YELLOW.c500,
                _ => tailwind::INDIGO.c50,
            };

            let time = (chrono::Utc.timestamp_millis_opt(data.best_timestamp).unwrap()
                + chrono::Duration::hours(8))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

            [
                Cell::from(format!("#{}", i + 1)),
                Cell::from(data.name.as_str()),
                Cell::from(self.itoa_buffer.format(data.best_score).to_string()),
                Cell::from(self.itoa_buffer.format(data.best_time).to_string()),
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
        .bg(Color::Reset)
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
}

impl Activity for RankingActivity {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.draw_ranking(frame);
        fade_in(frame, 0.5, self.app_time.as_secs_f32(), None);
    }

    fn update(&mut self, event: Option<Event>) {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
        }

        if !self.players_requested {
            if let Some(players) = data_manager!(get_players_best_except_self) {
                self.players_requested = true;
                self.constrant_len();
                self.show_items.extend(players);
                self.scroll_state = self
                    .scroll_state
                    .content_length(self.show_items.len() * ITEM_HEIGHT);
                self.by_score();
            }
        }

        if let Some(event) = event {
            self.update_input(event);
        }
    }
}
