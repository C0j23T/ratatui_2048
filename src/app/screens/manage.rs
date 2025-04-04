use std::{
    io::Cursor,
    sync::{Arc, atomic::AtomicI8},
    time::Duration,
};

use chrono::TimeZone;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use image::{AnimationDecoder, DynamicImage, codecs::gif::GifDecoder};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Offset, Rect},
    style::{Color, Style, Stylize, palette::tailwind},
    symbols,
    widgets::{
        Axis, Block, BorderType, Borders, Cell, Chart, Clear, Dataset, GraphType, HighlightSpacing,
        Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState, Table, TableState,
    },
};
use ratatui_image::{
    Resize, StatefulImage,
    picker::{Picker, ProtocolType},
};
use tui_textarea::TextArea;

use crate::{
    app::{
        manage::PlayerListSelector,
        math::inverse_lerp_f64,
        structs::Player,
        time::TIME,
        utils::{fade_in, format_date_short, format_datetime},
    },
    data_manager,
};

use super::{
    Activity,
    dialog::{DIALOG_MANAGER, Dialog},
};

static MOMOI: &'static [u8] = include_bytes!("../manage/momoi.gif");
static DORO: &'static [u8] = include_bytes!("../manage/doro.gif");

pub struct ManageActivity<'a> {
    pub should_exit: bool,
    selector: PlayerListSelector<'a>,
    in_selector: bool,
    player: Player,
    app_time: Duration,

    avatar: Vec<image::Frame>,
    avatar_picker: Picker,
    avatar_animation_interval: Duration,
    avatar_animation_index: usize,
    avatar_animation_current: DynamicImage,

    record_rows: Vec<Row<'a>>,
    record_state: TableState,
    record_scroll: ScrollbarState,
    record_remove_entered: bool,

    chart_earliest: i64,
    chart_latest: i64,
    chart_min: f64,
    chart_max: f64,
    chart_datasets: Vec<(f64, f64)>,

    renaming: bool,
    rename_textarea: TextArea<'a>,
    update_required: bool,
    remove_choice: Arc<AtomicI8>,
    remove_required: bool,
}

impl ManageActivity<'_> {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let flag = rng.gen_bool(0.5);
        let cursor = Cursor::new(if flag { MOMOI } else { DORO });
        let decoder = GifDecoder::new(cursor).unwrap();
        let frames = decoder.into_frames().collect_frames().unwrap();

        let mut picker = loop {
            let result = Picker::from_query_stdio();
            if let Ok(result) = result {
                break result;
            }
        };
        picker.set_background_color([0, 0, 0, 0]);
        picker.set_protocol_type(ProtocolType::Sixel);

        Self {
            selector: PlayerListSelector::new("玩家管理"),
            in_selector: true,
            should_exit: false,
            player: Player::default(),
            app_time: Duration::default(),
            avatar: frames,
            avatar_picker: picker,
            avatar_animation_interval: Duration::from_secs_f32(1.0 / 12.0),
            avatar_animation_index: 0,
            avatar_animation_current: DynamicImage::default(),
            record_rows: Vec::default(),
            record_scroll: ScrollbarState::default(),
            record_state: TableState::default(),
            record_remove_entered: false,
            chart_earliest: 0,
            chart_latest: 0,
            chart_max: 0.0,
            chart_min: 0.0,
            chart_datasets: Vec::default(),
            renaming: false,
            rename_textarea: TextArea::default(),
            update_required: false,
            remove_choice: Arc::new(AtomicI8::new(-1)),
            remove_required: false,
        }
    }

    fn reenter_selector(&mut self) {
        self.in_selector = true;
        self.selector = PlayerListSelector::new("玩家管理");
        self.app_time = Duration::default();
    }

    fn draw_top(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let block = Block::bordered()
            .borders(Borders::TOP)
            .title("── 玩家管理 ")
            .fg(tailwind::PURPLE.c50);
        frame.render_widget(block, area);
    }

    fn draw_avatar(&mut self, rect: Rect, frame: &mut Frame<'_>) {
        let block = Block::bordered()
            .border_type(BorderType::Rounded)
            .title("─ 头像")
            .fg(tailwind::INDIGO.c400);
        frame.render_widget(&block, rect);
        let area = block.inner(rect);

        let f = self.app_time.as_secs_f32() / self.avatar_animation_interval.as_secs_f32();
        let f = f.round() as usize;
        if self.avatar_animation_index != f {
            self.avatar_animation_index = f;
            let gif_frame = f % self.avatar.len();
            self.avatar_animation_current =
                self.avatar.get(gif_frame).unwrap().buffer().clone().into();
        }

        let mut state = self
            .avatar_picker
            .new_resize_protocol(self.avatar_animation_current.clone());
        let image_rect = state.size_for(&Resize::Scale(None), area);
        let x_offset = ((area.width - image_rect.width) as f32 / 2.0).round() as i32;

        frame.render_stateful_widget(
            StatefulImage::new().resize(Resize::Scale(None)),
            area.offset(Offset { x: x_offset, y: 0 }),
            &mut state,
        );

        let mut border_rect = image_rect;
        border_rect.x = area.x + x_offset as u16 - 1;
        border_rect.y = area.y;
        border_rect.width += 2;
        let border = Block::bordered().borders(Borders::LEFT | Borders::RIGHT);
        frame.render_widget(border, border_rect);
    }

    fn draw_info(&self, rect: Rect, frame: &mut Frame<'_>) {
        let time = (chrono::Utc
            .timestamp_millis_opt(self.player.best_timestamp)
            .unwrap()
            + chrono::Duration::hours(8))
        .format("%Y年%m月%d日 %H:%M:%S")
        .to_string();
        let para = Paragraph::new(format!(
            indoc::indoc! {"
            ID:   {}
            名称: {}

            最高分数:
            {}
            所用时间:
            {}
            达成时间:
            {}
        "},
            self.player.id, self.player.name, self.player.best_score, self.player.best_time, time,
        ))
        .fg(tailwind::BLUE.c50)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("─ 详细信息")
                .fg(tailwind::BLUE.c400),
        );
        frame.render_widget(para, rect);
    }

    fn draw_table(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let widths = [
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(8),
        ];
        let header = ["分数", "所用时间", "达成时间", "操作"]
            .into_iter()
            .map(Cell::from)
            .collect::<Row>()
            .bg(tailwind::BLUE.c500)
            .fg(tailwind::BLUE.c50);

        let table = Table::new(self.record_rows.clone(), widths)
            .header(header)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol("> ")
            .row_highlight_style(
                Style::default()
                    .bg(tailwind::BLUE.c600)
                    .fg(tailwind::BLUE.c50),
            )
            .cell_highlight_style(
                Style::default()
                    .bg(tailwind::RED.c600)
                    .fg(tailwind::RED.c50),
            )
            .block(
                Block::bordered()
                    .title("─ 存档管理 ")
                    .border_type(BorderType::Rounded)
                    .fg(tailwind::BLUE.c400),
            );

        frame.render_stateful_widget(table, area, &mut self.record_state);

        frame.render_stateful_widget(
            Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None),
            area.inner(Margin {
                vertical: 1,
                horizontal: 1,
            }),
            &mut self.record_scroll,
        );
    }

    fn setup_chart(&mut self) {
        let timestamp = self.player.records.iter().map(|x| x.timestamp);
        let latest = timestamp.clone().max().unwrap_or_default();
        let earliest = timestamp.min().unwrap_or_default();
        let score = self.player.records.iter().map(|x| x.score);
        let max = score.clone().max().unwrap_or_default() as f64;
        let min = score.min().unwrap_or_default() as f64;
        let mut data = self
            .player
            .records
            .iter()
            .map(|x| {
                (
                    inverse_lerp_f64(earliest as f64..=latest as f64, x.timestamp as f64),
                    x.score as f64,
                )
            })
            .collect::<Vec<_>>();
        data.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        self.chart_earliest = earliest;
        self.chart_latest = latest;
        self.chart_max = max;
        self.chart_min = min;
        self.chart_datasets = data;
    }

    fn draw_chart(&self, area: Rect, frame: &mut Frame<'_>) {
        let datasets = vec![
            Dataset::default()
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .fg(tailwind::YELLOW.c500)
                .data(&self.chart_datasets),
            Dataset::default()
                .marker(symbols::Marker::HalfBlock)
                .graph_type(GraphType::Bar)
                .fg(tailwind::BLUE.c500)
                .data(&self.chart_datasets),
        ];
        let chart = Chart::new(datasets)
            .y_axis(
                Axis::default()
                    .title("分数")
                    .bounds([self.chart_min, self.chart_max])
                    .labels([
                        format!("{:.0}", self.chart_min),
                        format!("{:.0}", self.chart_max / 2.0),
                        format!("{:.0}", self.chart_max),
                    ]),
            )
            .x_axis(Axis::default().title("日期").bounds([0.0, 1.0]).labels([
                format_date_short(self.chart_earliest),
                format_date_short(
                    self.chart_earliest + (self.chart_latest - self.chart_earliest) / 2,
                ),
                format_date_short(self.chart_latest),
            ]))
            .fg(tailwind::EMERALD.c50)
            .block(
                Block::bordered()
                    .title("─ 分数趋势 ")
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(tailwind::EMERALD.c400)),
            )
            .hidden_legend_constraints((Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)));

        frame.render_widget(chart, area);
    }

    fn draw_hint(&self, area: Rect, frame: &mut Frame<'_>) {
        let para = Paragraph::new("( ← ↑ ↓ → ) 移动光标 | ( S ) 返回选择界面").block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(tailwind::YELLOW.c400),
        );
        frame.render_widget(para, area);
    }

    fn draw_remove(&self, area: Rect, frame: &mut Frame<'_>) {
        let para = Paragraph::new("键入 D 彻底删除玩家")
            .fg(tailwind::RED.c50)
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .fg(tailwind::RED.c600),
            )
            .alignment(Alignment::Center);
        frame.render_widget(para, area);
    }

    fn draw_rename(&self, area: Rect, frame: &mut Frame<'_>) {
        let para = Paragraph::new("键入 R 更改名称")
            .fg(tailwind::LIME.c50)
            .alignment(Alignment::Center)
            .block(
                Block::bordered()
                    .fg(tailwind::LIME.c400)
                    .border_type(BorderType::Rounded),
            );
        frame.render_widget(para, area);
    }

    fn setup_rows(&mut self) {
        let mut buffer = itoa::Buffer::new();
        self.record_rows = self
            .player
            .records
            .clone()
            .into_iter()
            .map(|x| {
                [
                    Cell::from(buffer.format(x.score).to_string()),
                    Cell::from(buffer.format(x.time).to_string()),
                    Cell::from(format_datetime(x.timestamp)),
                    Cell::from("删除"),
                ]
                .into_iter()
                .collect::<Row>()
            })
            .collect::<Vec<_>>();
    }

    fn update_data(&mut self) {
        if self.update_required {
            if let Some(result) = data_manager!(update_player, self.player.clone()) {
                if !result {
                    let mut dialog_manger = DIALOG_MANAGER.write().unwrap();
                    dialog_manger.push(Dialog::new(
                        " 错误 ",
                        "修改失败：在更新内置数据库时遇到问题",
                        Alignment::Left,
                        false,
                        vec![String::from("确定")],
                        None,
                    ));
                }
                self.update_required = false;
            }
        }
        if self.remove_required {
            if let Some(result) = data_manager!(remove_player, self.player.clone()) {
                if !result {
                    let mut dialog_manger = DIALOG_MANAGER.write().unwrap();
                    dialog_manger.push(Dialog::new(
                        " 错误 ",
                        "删除失败：在更新内置数据库时遇到问题",
                        Alignment::Left,
                        false,
                        vec![String::from("确定")],
                        None,
                    ));
                } else {
                    self.should_exit = true;
                }
            }
        }
    }

    fn update_input(&mut self, key: KeyCode) -> bool {
        let disable_flag = self.renaming || self.record_remove_entered;
        match key {
            KeyCode::Char('s') => {
                if !self.renaming {
                    self.reenter_selector();
                } else {
                    return true;
                }
            }
            KeyCode::Down => {
                if disable_flag {
                    return false;
                }
                let i = match self.record_state.selected() {
                    Some(i) => {
                        if i >= self.record_rows.len() - 1 {
                            0
                        } else {
                            i + 1
                        }
                    }
                    None => 0,
                };
                if let Some((_, c)) = self.record_state.selected_cell() {
                    self.record_state.select_cell(Some((i, c)));
                } else {
                    self.record_state.select(Some(i));
                }
                self.record_scroll = self.record_scroll.position(i);
            }
            KeyCode::Up => {
                if disable_flag {
                    return false;
                }
                let i = match self.record_state.selected() {
                    Some(i) => {
                        if i == 0 {
                            self.record_rows.len() - 1
                        } else {
                            i - 1
                        }
                    }
                    None => 0,
                };
                if let Some((_, c)) = self.record_state.selected_cell() {
                    self.record_state.select_cell(Some((i, c)));
                } else {
                    self.record_state.select(Some(i));
                }
                self.record_scroll = self.record_scroll.position(i);
            }
            KeyCode::Left | KeyCode::Right => {
                if disable_flag {
                    return false;
                }
                let Some(row) = self.record_state.selected() else {
                    return false;
                };
                if self.record_state.selected_cell().is_none() {
                    self.record_state.select_cell(Some((row, 3)));
                } else {
                    self.record_state.select_cell(None);
                    self.record_state.select(Some(row));
                }
            }
            KeyCode::Char('r') => {
                if !self.renaming {
                    if disable_flag {
                        return false;
                    }
                    let mut textarea = TextArea::default();
                    textarea.set_block(Block::bordered().title(" 重命名 "));
                    textarea.set_placeholder_text("请输入新的名称");
                    self.rename_textarea = textarea;
                    self.renaming = true;
                } else {
                    return true;
                }
            }
            KeyCode::Char('d') => {
                if !self.renaming {
                    if disable_flag {
                        return false;
                    }

                    let mut dialog_manager = DIALOG_MANAGER.write().unwrap();
                    dialog_manager.push(Dialog::new(
                        " 你确定吗？ ",
                        "如果你删除该玩家，它将永远会消失。（很长时间！）",
                        Alignment::Left,
                        false,
                        vec![String::from("确定"), String::from("取消")],
                        Some(self.remove_choice.clone()),
                    ));
                } else {
                    return true;
                }
            }
            KeyCode::Enter => {
                if self.renaming {
                    self.renaming = false;
                    self.player.name = self.rename_textarea.lines()[0].clone();
                    self.update_required = true;
                } else if self.record_remove_entered {
                    let Some(index) = self.record_state.selected() else {
                        return false;
                    };
                    self.player.records.remove(index);
                    self.validate_player();
                    self.setup_rows();
                    self.setup_chart();
                    self.record_remove_entered = false;
                    self.update_required = true;
                } else if self.record_state.selected_cell().is_some() && !self.record_remove_entered
                {
                    self.record_remove_entered = true;
                }
            }
            KeyCode::Esc => {
                self.record_remove_entered = false;
                self.renaming = false;
            }
            _ => {
                return true;
            }
        }
        false
    }

    fn draw_overlay(&self, frame: &mut Frame<'_>) {
        if !self.record_remove_entered && !self.renaming {
            return;
        }

        let area = frame.area();

        let buf = frame.buffer_mut();
        buf.content.iter_mut().for_each(|x| {
            if let Color::Rgb(r, g, b) = x.fg {
                x.fg = Color::Rgb(
                    r.saturating_sub(100),
                    g.saturating_sub(100),
                    b.saturating_sub(100),
                );
            }
            if let Color::Rgb(r, g, b) = x.bg {
                x.bg = Color::Rgb(
                    r.saturating_sub(100),
                    g.saturating_sub(100),
                    b.saturating_sub(100),
                );
            }
        });

        let [_, _, chunk, _] = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Fill(1),
            Constraint::Percentage(50),
            Constraint::Fill(1),
        ])
        .areas(area);

        let [_, dialog, _] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Percentage(35),
            Constraint::Fill(1),
        ])
        .areas(chunk);

        frame.render_widget(Clear, dialog);

        if self.record_remove_entered {
            let para = Paragraph::new(
                "确定要删除本条记录吗？\n如果你删除本条记录，它将永远会消失。（很长时间！）",
            )
            .block(
                Block::bordered()
                    .border_type(BorderType::Double)
                    .title_bottom("( ⏎ ) 确定 | ( ESC ) 取消")
                    .title_alignment(Alignment::Right),
            )
            .fg(tailwind::WHITE)
            .alignment(Alignment::Center);
            frame.render_widget(para, dialog);
        } else {
            let block = Block::bordered()
                .border_type(BorderType::Double)
                .title_bottom("( ⏎ ) 确定 | ( ESC ) 取消")
                .title_alignment(Alignment::Right);
            frame.render_widget(&block, dialog);

            let inner = block.inner(dialog);
            let [text, _] =
                Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).areas(inner);
            frame.render_widget(&self.rename_textarea, text);
        }
    }

    fn validate_player(&mut self) {
        self.player.best_score = 0;
        self.player.best_time = 0;
        self.player.best_timestamp = 0;
        let Some(max) = self.player.records.iter().max_by_key(|x| x.score) else {
            return;
        };
        self.player.best_score = max.score;
        self.player.best_time = max.time;
        self.player.best_timestamp = max.timestamp;
    }
}

impl Activity for ManageActivity<'_> {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        if self.in_selector {
            self.selector.draw(frame);
            return;
        }

        let area = frame.area();

        let [top, chunk] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
        let [left, chunk] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(chunk);
        let [avatar, info, rename, remove] = Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .areas(left);
        let [records, chart, hint] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Percentage(40),
            Constraint::Length(3),
        ])
        .areas(chunk);

        self.draw_top(top, frame);
        self.draw_avatar(avatar, frame);
        self.draw_info(info, frame);
        self.draw_remove(remove, frame);
        self.draw_rename(rename, frame);
        self.draw_table(records, frame);
        self.draw_chart(chart, frame);
        self.draw_hint(hint, frame);
        self.draw_overlay(frame);

        fade_in(frame, 0.6, self.app_time.as_secs_f32(), None);
    }

    fn update(&mut self, event: Option<Event>) {
        if !self.in_selector {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
        }

        self.update_data();
        {
            let choice = self
                .remove_choice
                .load(std::sync::atomic::Ordering::Relaxed);
            if choice == 0 {
                self.remove_required = true;
            }
            self.remove_choice
                .store(-1, std::sync::atomic::Ordering::Relaxed);
        }

        if self.in_selector {
            self.selector.update(event);

            if self.selector.should_exit {
                self.in_selector = false;
                if let Some(player) = self.selector.get_result() {
                    self.record_scroll = self.record_scroll.content_length(player.records.len());
                    self.player = player;
                    self.validate_player();
                    self.setup_rows();
                    self.setup_chart();
                    let selector = std::mem::take(&mut self.selector);
                    drop(selector);
                } else {
                    self.should_exit = true;
                }
            }
            return;
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
        if !self.renaming && key.code == KeyCode::Char('q')
            || !self.renaming && !self.record_remove_entered && key.code == KeyCode::Esc
        {
            self.should_exit = true;
        }
        if self.update_input(key.code) && self.renaming {
            self.rename_textarea.input(event);
        }
    }
}
