use crossterm::event::{Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize, palette::tailwind},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Table, TableState,
    },
};
use tui_textarea::TextArea;

use crate::data_manager;

use super::{screens::Activity, structs::Player};

// TODO: Ranking控件，实现搜索，和选择条目
#[derive(Default)]
pub struct PlayerListSelector<'a> {
    pub should_exit: bool,
    pub feature_name: String,

    search_bar: TextArea<'a>,
    search_result: Vec<Player>,
    search_result_rows: Vec<Row<'a>>,
    search_table_id_width: u16,
    last_search_text: String,
    search_table_state: TableState,
    cursor_state: CursorState,

    table_state: TableState,
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
            cursor_state: CursorState::Search,
            search_bar,
            last_search_text: String::from("\u{0}"),
            ..Default::default()
        }
    }

    fn draw_title(&self, rect: Rect, frame: &mut Frame<'_>) {
        let block = Block::bordered()
            .borders(Borders::TOP)
            .title(format!("── {} - 请选择要操作的用户 ", self.feature_name))
            .fg(tailwind::INDIGO.c200);
        frame.render_widget(block, rect);
    }

    fn draw_left_bar(&mut self, rect: Rect, frame: &mut Frame<'_>) {
        let is_search = matches!(self.cursor_state, CursorState::Search);
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!("─ 查找 "))
            .fg(if is_search {
                tailwind::INDIGO.c200
            } else {
                tailwind::INDIGO.c400
            });
        frame.render_widget(block, rect);

        let [content] = Layout::default()
            .constraints([Constraint::Min(0)])
            .margin(1)
            .areas(rect);
        let [search_bar, result_bar] =
            Layout::vertical([Constraint::Length(3), Constraint::Min(0)]).areas(content);

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

    fn draw_main_table(&mut self, rect: Rect, frame: &mut Frame<'_>) {}

    fn draw_hint(&mut self, rect: Rect, frame: &mut Frame<'_>) {
        let hint = if matches!(self.cursor_state, CursorState::Table) {
            indoc::indoc! {"
                ( F ) 搜索 | ( ← → ) 切换 查找 / 选择 | ( ⏎ ) 确定
            "}
        } else {
            indoc::indoc! {"
                ( ↑ ↓ ) 切换预选条目 | ( ← → ) 切换 查找 / 选择
            "}
        };
        let para = Paragraph::new(hint)
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title("─ 提示 "),
            )
            .fg(tailwind::INDIGO.c200);
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
                }
                KeyCode::Enter => (),
                _ => {
                    self.search_bar.input(event);
                }
            }
        } else {
            match key.code {
                KeyCode::Char('f') => self.cursor_state = CursorState::Search,
                _ => (),
            }
        }
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
    }

    fn update(&mut self, event: Option<Event>) {
        if self.last_search_text != self.search_bar.lines()[0] {
            let search_text = &self.search_bar.lines()[0];
            let player = Player {
                id: if let Ok(x) = search_text.parse::<i32>() {
                    x
                } else {
                    0
                },
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
