use std::time::Duration;

use chrono::TimeZone;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Margin, Rect},
    style::{Color, Style, Stylize, palette::tailwind},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};
use tui_textarea::TextArea;

use crate::data_manager;

use super::{screens::Activity, structs::Player, time::TIME, utils::fade_in};

// TODO: Ranking控件，实现搜索，和选择条目
#[derive(Default)]
pub struct PlayerListSelector<'a> {
    pub should_exit: bool,
    pub feature_name: String,
    app_time: Duration,

    search_bar: TextArea<'a>,
    search_result: Vec<Player>,
    search_result_rows: Vec<Row<'a>>,
    search_table_id_width: u16,
    last_search_text: String,
    search_table_state: TableState,
    cursor_state: CursorState,

    table_state: TableState,
    players: Vec<Player>,
    players_requested: bool,
    player_rows: Vec<Row<'a>>,
    player_columns_longest: (u16, u16, u16, u16),
    scroll_state: ScrollbarState,

    selection: Option<Player>,
}

#[derive(Default)]
enum CursorState {
    Search,
    #[default]
    Table,
}

impl PlayerListSelector<'_> {
    pub fn new(feature_name: impl AsRef<str>) -> Self {
        let mut search_bar = TextArea::default();
        search_bar.set_block(Block::bordered());
        Self {
            feature_name: feature_name.as_ref().to_string(),
            search_bar,
            last_search_text: String::from("\u{0}"),
            ..Default::default()
        }
    }

    fn draw_title(&self, rect: Rect, frame: &mut Frame<'_>) {
        let block = Block::bordered()
            .borders(Borders::TOP)
            .title(format!("── {} :: 请选择要操作的用户 ", self.feature_name))
            .fg(tailwind::PURPLE.c50);
        frame.render_widget(block, rect);
    }

    fn draw_left_bar(&mut self, rect: Rect, frame: &mut Frame<'_>) {
        let is_search = matches!(self.cursor_state, CursorState::Search);
        let fg = if is_search {
            tailwind::INDIGO.c200
        } else {
            tailwind::INDIGO.c400
        };
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("─ 查找 ")
            .fg(fg);
        frame.render_widget(&block, rect);

        let content = block.inner(rect);
        let [search_bar, result_bar] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(content);

        let cursor_color = if self.app_time.as_secs() % 2 == 0 {
            fg
        } else {
            Color::Reset
        };
        let cursor_style = Style::default().bg(cursor_color);
        self.search_bar.set_cursor_style(cursor_style);

        frame.render_widget(&self.search_bar, search_bar);

        let widths = [
            Constraint::Length(self.search_table_id_width),
            Constraint::Min(0),
        ];
        let table = Table::new(self.search_result_rows.clone(), widths)
            .block(Block::bordered().title(" 结果 "))
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol("> ")
            .row_highlight_style(Style::default().bg(if is_search {
                tailwind::INDIGO.c400
            } else {
                tailwind::INDIGO.c500
            }));

        frame.render_stateful_widget(table, result_bar, &mut self.search_table_state);
    }

    fn draw_main_table(&mut self, rect: Rect, frame: &mut Frame<'_>) {
        let is_table = matches!(self.cursor_state, CursorState::Table);
        let fg = if is_table {
            tailwind::INDIGO.c50
        } else {
            tailwind::INDIGO.c300
        };

        let widths = [
            Constraint::Min(self.player_columns_longest.0 + 1),
            Constraint::Min(self.player_columns_longest.1 + 1),
            Constraint::Min(self.player_columns_longest.2 + 1),
            Constraint::Min(self.player_columns_longest.3 + 1),
            Constraint::Min(22),
        ];
        let header = ["ID", "名称", "分数", "所用时间", "达成时间"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .bg(tailwind::INDIGO.c600)
            .fg(tailwind::INDIGO.c50);

        let table = Table::new(self.player_rows.clone(), widths)
            .block(
                Block::bordered()
                    .title("─ 玩家列表 ")
                    .fg(fg)
                    .border_type(BorderType::Rounded),
            )
            .header(header)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol("> ")
            .row_highlight_style(
                Style::default()
                    .bg(if is_table {
                        tailwind::INDIGO.c400
                    } else {
                        tailwind::INDIGO.c500
                    })
                    .fg(if is_table {
                        tailwind::INDIGO.c50
                    } else {
                        tailwind::INDIGO.c100
                    }),
            );

        frame.render_stateful_widget(table, rect, &mut self.table_state);

        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            rect.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.scroll_state,
        );
    }

    fn draw_hint(&mut self, rect: Rect, frame: &mut Frame<'_>) {
        let hint = if matches!(self.cursor_state, CursorState::Table) {
            indoc::indoc! {"
                ( F ) 搜索 | ( ⏎ ) 确定 | ( ← → ) 切换 查找 / 选择 | ( ESC ) 退出
            "}
        } else {
            indoc::indoc! {"
                ( ↑ ↓ ) 切换预选条目 | ( ← → ) 切换 查找 / 选择 | ( ESC ) 退出
            "}
        };
        let para = Paragraph::new(hint)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("─ 提示 "),
            )
            .fg(tailwind::EMERALD.c400);
        frame.render_widget(para, rect);
    }

    fn update_input(&mut self, event: Option<Event>) {
        let Some(event) = event else {
            return;
        };
        let Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if !matches!(self.cursor_state, CursorState::Search) && key.code == KeyCode::Char('q')
            || key.code == KeyCode::Esc
        {
            self.should_exit = true;
        }
        let is_search = matches!(self.cursor_state, CursorState::Search);
        if key.code == KeyCode::Left || key.code == KeyCode::Right {
            if is_search {
                self.cursor_state = CursorState::Table;
            } else {
                self.cursor_state = CursorState::Search;
            }
            return;
        }
        if is_search {
            match key.code {
                KeyCode::Down => {
                    let i = match self.search_table_state.selected() {
                        Some(i) => {
                            if i >= self.search_result.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.search_table_state.select(Some(i));
                    self.search_index_to_main_table(i);
                }
                KeyCode::Up => {
                    let i = match self.search_table_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.search_result.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.search_table_state.select(Some(i));
                    self.search_index_to_main_table(i);
                }
                KeyCode::Enter => (),
                _ => {
                    self.search_bar.input(event);
                }
            }
        } else {
            match key.code {
                KeyCode::Char('f') => self.cursor_state = CursorState::Search,
                KeyCode::Down => {
                    let i = match self.table_state.selected() {
                        Some(i) => {
                            if i >= self.players.len() - 1 {
                                0
                            } else {
                                i + 1
                            }
                        }
                        None => 0,
                    };
                    self.table_state.select(Some(i));
                    self.scroll_state = self.scroll_state.position(i);
                }
                KeyCode::Up => {
                    let i = match self.table_state.selected() {
                        Some(i) => {
                            if i == 0 {
                                self.players.len() - 1
                            } else {
                                i - 1
                            }
                        }
                        None => 0,
                    };
                    self.table_state.select(Some(i));
                    self.scroll_state = self.scroll_state.position(i);
                }
                KeyCode::Enter => {
                    let Some(i) = self.table_state.selected() else {
                        return;
                    };
                    self.selection = self.players.get(i).cloned();
                    self.should_exit = self.selection.is_some();
                }
                _ => (),
            }
        }
    }

    fn calculate_players_columns_longest(&mut self) {
        let mut buffer = itoa::Buffer::new();
        let id_len = self
            .players
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width(buffer.format(x.id)))
            .max()
            .unwrap_or(0) as u16;
        let name_len = self
            .players
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(x.name.as_str()))
            .max()
            .unwrap_or(0) as u16;
        let score_len = self
            .players
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(buffer.format(x.best_score)))
            .max()
            .unwrap_or(0) as u16;
        let time_len = self
            .players
            .iter()
            .map(|x| unicode_width::UnicodeWidthStr::width_cjk(buffer.format(x.best_time)))
            .max()
            .unwrap_or(0) as u16;
        self.player_columns_longest = (id_len, name_len, score_len, time_len)
    }

    fn find_row(&mut self, id: i32) {
        let i = self
            .players
            .iter()
            .enumerate()
            .find(|(_, x)| x.id == id)
            .map_or(0, |(x, _)| x);

        self.table_state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i);
    }

    fn search_index_to_main_table(&mut self, i: usize) {
        let id = self.search_result.get(i).map_or(0, |x| x.id);
        self.find_row(id);
    }

    pub fn get_result(&mut self) -> Option<Player> {
        self.selection.clone()
    }
}

impl Activity for PlayerListSelector<'_> {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        let area = frame.area();

        let [top, div] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
        let [left, div] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Min(0)]).areas(div);
        let [table, hint] =
            Layout::vertical([Constraint::Min(0), Constraint::Percentage(20)]).areas(div);

        self.draw_title(top, frame);
        self.draw_left_bar(left, frame);
        self.draw_main_table(table, frame);
        self.draw_hint(hint, frame);

        fade_in(frame, 0.6, self.app_time.as_secs_f32(), Some(10));
    }

    fn update(&mut self, event: Option<Event>) {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
        }

        if !self.players_requested {
            if let Some(players) = data_manager!(get_players) {
                let mut buffer = itoa::Buffer::new();
                self.players_requested = true;
                self.player_rows = players
                    .clone()
                    .into_iter()
                    .map(|x| {
                        let time = if x.best_timestamp != 0 {
                            (chrono::Utc.timestamp_millis_opt(x.best_timestamp).unwrap()
                                + chrono::Duration::hours(8))
                            .format("%Y-%m-%d %H:%M:%S")
                            .to_string()
                        } else {
                            String::from("无")
                        };
                        [
                            Cell::from(buffer.format(x.id).to_string()),
                            Cell::from(x.name),
                            Cell::from(buffer.format(x.best_score).to_string()),
                            Cell::from(buffer.format(x.best_time).to_string()),
                            Cell::from(time),
                        ]
                        .into_iter()
                        .collect::<Row>()
                    })
                    .collect::<Vec<_>>();
                self.calculate_players_columns_longest();
                self.scroll_state = self.scroll_state.content_length(players.len());
                self.players = players;
            }
        }
        if self.last_search_text != self.search_bar.lines()[0] {
            let search_text = &self.search_bar.lines()[0];
            let player = Player {
                id: search_text.parse::<i32>().unwrap_or_default(),
                name: search_text.clone(),
                ..Default::default()
            };
            if let Some(players) = data_manager!(find_player, player) {
                let mut buffer = itoa::Buffer::new();
                self.search_result_rows = players
                    .clone()
                    .into_iter()
                    .map(|x| {
                        [
                            Cell::from(buffer.format(x.id).to_string()),
                            Cell::from(format!("│ {}", x.name)),
                        ]
                        .into_iter()
                        .collect::<Row>()
                    })
                    .collect::<Vec<_>>();
                let id_len = players
                    .iter()
                    .map(|x| unicode_width::UnicodeWidthStr::width(buffer.format(x.id)))
                    .max()
                    .unwrap_or(0) as u16;
                self.search_table_id_width = id_len;
                self.search_result = players;
                self.last_search_text = search_text.clone();
            }
        }
        self.update_input(event);
    }
}
