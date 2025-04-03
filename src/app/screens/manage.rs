use std::{io::Cursor, time::Duration};

use chrono::TimeZone;
use crossterm::event::{Event, KeyCode, KeyEventKind};
use image::{AnimationDecoder, DynamicImage, codecs::gif::GifDecoder};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Offset, Rect},
    style::{Stylize, palette::tailwind},
    widgets::{
        Block, BorderType, Borders, Cell, HighlightSpacing, Paragraph, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Table, TableState, Wrap,
    },
};
use ratatui_image::{
    Resize, StatefulImage,
    picker::{Picker, ProtocolType},
};

use crate::app::{manage::PlayerListSelector, structs::Player, time::TIME};

use super::Activity;

static MOMOI_DANCE: &'static [u8] = include_bytes!("../manage/momoi.gif");

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
}

impl ManageActivity<'_> {
    pub fn new() -> Self {
        let cursor = Cursor::new(MOMOI_DANCE);
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
        }
    }

    fn reenter_selector(&mut self) {
        self.in_selector = true;
        self.selector = PlayerListSelector::new("玩家管理");
    }

    fn draw_top(&mut self, area: Rect, frame: &mut Frame<'_>) {
        let block = Block::bordered()
            .borders(Borders::TOP)
            .title("── 玩家管理 ")
            .fg(tailwind::INDIGO.c400);
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
        .format("%Y年%m月%d日 %H:%M:%S秒")
        .to_string();
        let para = Paragraph::new(format!(
            indoc::indoc! {"
            ID:   {}
            名称: {}

            最高分数:\n{}
            所用时间:\n{}
            达成时间:\n{}
        "},
            self.player.id, self.player.name, self.player.best_score, self.player.best_time, time,
        ))
        .fg(tailwind::INDIGO.c400)
        .wrap(Wrap { trim: false })
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .title("─ 详细信息")
                .fg(tailwind::INDIGO.c400),
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
            .bg(tailwind::INDIGO.c600)
            .fg(tailwind::INDIGO.c50);

        let table = Table::new(self.record_rows.clone(), widths)
            .header(header)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol("> ")
            .row_highlight_style(tailwind::INDIGO.c400)
            .block(
                Block::bordered()
                    .title("─ 存档管理 ")
                    .border_type(BorderType::Rounded)
                    .fg(tailwind::INDIGO.c400),
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

    fn draw_chart(&self, area: Rect, frame: &mut Frame<'_>) {
        let block = Block::bordered()
            .title("─ 分数趋势 ")
            .border_type(BorderType::Rounded)
            .fg(tailwind::INDIGO.c400);
        frame.render_widget(block, area);
    }

    fn draw_hint(&self, area: Rect, frame: &mut Frame<'_>) {
        let para = Paragraph::new("( ← ↑ ↓ → ) 移动光标 | ( S ) 返回选择界面").block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .fg(tailwind::INDIGO.c400),
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

    fn setup_rows(&mut self) {
        let mut buffer = itoa::Buffer::new();
        self.record_rows = self
            .player
            .records
            .clone()
            .into_iter()
            .map(|x| {
                let time = (chrono::Utc.timestamp_millis_opt(x.timestamp).unwrap()
                    + chrono::Duration::hours(8))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
                [
                    Cell::from(buffer.format(x.score).to_string()),
                    Cell::from(buffer.format(x.time).to_string()),
                    Cell::from(time),
                    Cell::from("删除"),
                ]
                .into_iter()
                .collect::<Row>()
            })
            .collect::<Vec<_>>();
    }
}

impl Activity for ManageActivity<'_> {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        if self.in_selector {
            self.selector.draw(frame);
            return;
        }

        let area = frame.area();

        let [top, div] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(area);
        let [left, div] =
            Layout::horizontal([Constraint::Percentage(30), Constraint::Min(0)]).areas(div);
        let [avatar, info, remove] = Layout::vertical([
            Constraint::Percentage(40),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(left);
        let [records, chart, hint] = Layout::vertical([
            Constraint::Min(0),
            Constraint::Percentage(40),
            Constraint::Length(3),
        ])
        .areas(div);

        self.draw_top(top, frame);
        self.draw_avatar(avatar, frame);
        self.draw_info(info, frame);
        self.draw_remove(remove, frame);
        self.draw_table(records, frame);
        self.draw_chart(chart, frame);
        self.draw_hint(hint, frame);
    }

    fn update(&mut self, event: Option<Event>) {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
        }

        if self.in_selector {
            self.selector.update(event);

            if self.selector.should_exit {
                self.in_selector = false;
                if let Some(player) = self.selector.get_result() {
                    self.record_scroll = self.record_scroll.content_length(player.records.len());
                    self.player = player;
                    self.setup_rows();
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
        if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
            self.should_exit = true;
        }
    }
}
