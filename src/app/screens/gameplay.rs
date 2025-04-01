use std::{
    rc::Rc,
    sync::{Arc, atomic::AtomicI8},
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use rand::Rng;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Flex, Layout, Rect},
    style::{Style, palette::tailwind},
    widgets::{Block, BorderType, Borders, Clear, Padding, Paragraph},
};

use crate::app::{
    ascii,
    gameplay::{movement::*, *},
    math::{Interpolation, inverse_lerp, lerpf},
    structs::*,
    time::TIME,
    utils::{fade_in, get_time_millis, rect_move, rect_scale},
};

use super::{
    Activity,
    dialog::{DIALOG_MANAGER, Dialog},
};

pub struct GameplayActivity {
    cells: Grid,
    visual_cells: Grid,
    score: i32,
    show_score: i32,
    high_score: Player,
    play_time: Duration,
    app_time: Duration,
    play_started: bool,
    itoa_buffer: itoa::Buffer,
    animations: Vec<AnimationCell>,

    pub exit: bool,
    pub game_over: bool,
    dead_dialog: bool,
    dead_dialog_chose: Arc<AtomicI8>,
    dead_time: i64,
    pub show_ranking: bool,

    dead_dialog_time: Duration,
}

impl GameplayActivity {
    pub fn new(save: Player) -> Self {
        // 到这里应该早就被初始化了
        let mut this = Self {
            cells: vec![vec![Cell::default(); 4]; 4],
            visual_cells: vec![vec![Cell::default(); 4]; 4],
            itoa_buffer: itoa::Buffer::new(),
            high_score: save,
            score: 0,
            show_score: 0,
            animations: Vec::new(),
            exit: false,
            play_started: false,
            game_over: false,
            dead_dialog: false,
            show_ranking: false,
            dead_time: 0,
            play_time: Duration::default(),
            app_time: Duration::default(),
            dead_dialog_time: Duration::default(),
            dead_dialog_chose: Arc::new(AtomicI8::new(-1)),
        };

        this.animations.push(start_up(&mut this.cells));
        this
    }

    fn gameplay_update_input(&mut self, event: Event) {
        {
            let dialog_manager = DIALOG_MANAGER.read().unwrap();
            if dialog_manager.has_dialog() {
                return;
            }
        }

        let event::Event::Key(key) = event else {
            return;
        };
        if key.kind != KeyEventKind::Press {
            return;
        }
        if matches!(key.code, KeyCode::Char('q')) || matches!(key.code, KeyCode::Esc) {
            self.exit = true;
        }
        if self.game_over {
            return;
        }

        let mut pressed = false;
        let mut total_score = 0;
        let mut animations = Vec::new();
        match key.code {
            KeyCode::Up => {
                let (anim, score) = move_up(&mut self.cells);
                animations.extend(anim);
                total_score += score as i32;
                pressed = true;
            }
            KeyCode::Down => {
                let (anim, score) = move_down(&mut self.cells);
                animations.extend(anim);
                total_score += score as i32;
                pressed = true;
            }
            KeyCode::Left => {
                let (anim, score) = move_left(&mut self.cells);
                animations.extend(anim);
                total_score += score as i32;
                pressed = true;
            }
            KeyCode::Right => {
                let (anim, score) = move_right(&mut self.cells);
                animations.extend(anim);
                total_score += score as i32;
                pressed = true;
            }
            _ => (),
        }
        if pressed {
            if !animations.is_empty() {
                // 如果地块改变过
                add_cell(&mut self.cells)
                    .iter()
                    .for_each(|x| animations.push(*x));
            }
            animations.sort_by(|a, b| a.animation_type.partial_cmp(&b.animation_type).unwrap());
            self.play_started = true;
            self.animations = animations;
            self.score += total_score;
            self.game_over = check_game_over(&mut self.cells);
        }
        if self.game_over {
            self.dead_dialog_time = self.app_time + Duration::from_secs(2);
        }
    }

    fn gen_block(itoa_buffer: &mut itoa::Buffer, value: u16, rect: Rect) -> Paragraph {
        let block_text = if value == 0 {
            " "
        } else {
            itoa_buffer.format(value)
        };
        Paragraph::new(block_text)
            .style(Style::default().fg(colors::color_setter(value)))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .padding(Padding::new(0, 1, rect.height / 3, 1)),
            )
            .alignment(Alignment::Center)
    }

    #[allow(clippy::needless_range_loop)]
    fn gameplay_draw(&mut self, frame: &mut Frame<'_>) {
        let exp_out = Interpolation::ExpOut { value: 20.0 };
        let area = frame.area();

        let [title, div] = Layout::vertical([Constraint::Max(3), Constraint::Min(0)]).areas(area);

        let outer_subdiv = Layout::horizontal(vec![
            Constraint::Fill(1),
            Constraint::Max(area.height * 2 + 6),
            Constraint::Max((area.height / 2) + 7),
            Constraint::Fill(1),
        ])
        .flex(Flex::Center)
        .split(div);

        let outer_lower = Layout::vertical(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(outer_subdiv[1]);

        let mut cols: [Rc<[Rect]>; 4] = [Rc::new([]), Rc::new([]), Rc::new([]), Rc::new([])];

        for i in 0..4 {
            cols[i] = Layout::horizontal(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ])
            .split(outer_lower[i]);
        }

        // 内容绘制

        let text = if self.play_started {
            "2048 小游戏"
        } else {
            "按方向键以开始游戏"
        };
        let header = Paragraph::new(text)
            .style(Style::default().fg(tailwind::GREEN.c50))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded),
            )
            //center the text vertically and horizontally
            .alignment(Alignment::Center);
        frame.render_widget(header, title);

        {
            let cell = Paragraph::default()
                .style(Style::default().fg(colors::color_setter(0)))
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded),
                );
            for rows in &cols {
                for row in rows.iter() {
                    frame.render_widget(&cell, *row);
                }
            }
        }

        // Popup和Move动画完成后，将cells复制到visual_cells
        self.animations.retain(|x| {
            (matches!(x.animation_type, CellAnimationType::Popup) && x.duration.as_secs_f32() < 0.3)
                || (matches!(x.animation_type, CellAnimationType::Move)
                    && x.duration.as_secs_f32() < 0.2)
        });

        // 从显示列表删除地块
        for cell in &self.animations {
            self.visual_cells[cell.src.x][cell.src.y].set_v(0);
        }
        // 播放动画
        for cell in &self.animations {
            match cell.animation_type {
                CellAnimationType::Popup => {
                    let progress =
                        inverse_lerp(0.0..=0.8_f32, cell.duration.as_secs_f32());
                    let rect = rect_scale(cols[cell.src.x][cell.src.y], exp_out.apply(progress));
                    frame.render_widget(Clear, rect);
                    frame.render_widget(
                        Self::gen_block(&mut self.itoa_buffer, cell.value, rect),
                        rect,
                    );
                }
                CellAnimationType::Move => {
                    let progress =
                        inverse_lerp(0.0..=0.6_f32, cell.duration.as_secs_f32());
                    let dest = cell.dest.as_ref().unwrap();
                    let rect = rect_move(
                        cols[cell.src.x][cell.src.y],
                        // TODO: 这里似乎有个bug，但是暂时无法复现
                        cols[dest.x][dest.y],
                        exp_out.apply(progress),
                    );
                    frame.render_widget(Clear, rect);
                    frame.render_widget(
                        Self::gen_block(&mut self.itoa_buffer, cell.value, rect),
                        rect,
                    );
                }
            }
        }
        // 动画播放完成后，把判断列表里的地块全部复制到显示列表
        for i in 0..4 {
            for j in 0..4 {
                let value = self.cells[i][j].get();
                if !self.animations.iter().any(|x| {
                    if matches!(x.animation_type, CellAnimationType::Move) {
                        let dest = x.dest.unwrap();
                        dest.x == i && dest.y == j
                    } else {
                        x.src.x == i && x.src.y == j
                    }
                }) {
                    self.visual_cells[i][j].set_v(value);
                }
            }
        }

        {
            let time = TIME.read().unwrap();
            for cell in &mut self.animations {
                cell.duration += time.delta;
            }
        }

        for i in 0..4 {
            for j in 0..4 {
                let value = self.visual_cells[i][j].get();
                if value == 0 {
                    continue;
                }
                frame.render_widget(
                    Self::gen_block(&mut self.itoa_buffer, value, cols[i][j]),
                    cols[i][j],
                );
            }
        }

        self.show_score = lerpf(self.show_score as f32..=self.score as f32, 0.1).round() as i32;

        let stats_detail = Paragraph::new(format!(
            indoc::indoc! {"
                👤 玩家名:
                {}


                分数: {:04}
                最佳: {:04}

                ⌚ 游玩时间:
                {}




                🎮 如何控制:
                ← ↑ ↓ →


                键入'Q'以退出游戏
            "},
            self.high_score.name,
            self.show_score,
            self.high_score.score,
            self.play_time.as_secs()
        ))
        .style(Style::default().fg(tailwind::INDIGO.c300))
        .block(
            Block::default()
                .title("Stats")
                .title_alignment(Alignment::Left)
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .padding(Padding::new(1, 1, 1, 1)),
        )
        //center the text vertically and horizontally
        .alignment(Alignment::Left);
        frame.render_widget(stats_detail, outer_subdiv[2]);

        frame.render_widget(fx::gen_matrix(self.app_time), outer_subdiv[0]);
        frame.render_widget(fx::gen_matrix(self.app_time), outer_subdiv[3]);

        fade_in(frame, 0.8, self.app_time.as_secs_f32(), None);
    }

    pub fn queue_clear_message(&mut self) {
        let dialog_chose = self.dead_dialog_chose.clone();
        let ascii_art = if self.score < self.high_score.score {
            let mut rng = rand::rng();
            let num = rng.random_range(1..=10);
            if num <= 2 {
                ascii::god_fall()
            } else {
                ascii::the_end()
            }
        } else {
            ascii::the_end()
        };
        let mut dialog_manager = DIALOG_MANAGER.write().unwrap();
        dialog_manager.push(Dialog::new(
            " 游戏结束 ",
            &format!(
                "{}\n已经没有块可以移动了！\n\n最终成绩: {} 分\n最高成绩: {} 分 ({:+})\n最终用时: {}秒",
                ascii_art,
                self.score,
                self.high_score.score,
                self.score - self.high_score.score,
                self.play_time.as_secs(),
            ),
            Alignment::Center,
            false,
            vec![String::from("重试"), String::from("查看排行"), String::from("退出")],
            dialog_chose.clone(),
        ));
    }

    fn update_clear_chose(&mut self) {
        let chose = self
            .dead_dialog_chose
            .load(std::sync::atomic::Ordering::Relaxed);
        if chose == 0 {
            *self = Self::new(self.high_score.clone());
            let mut time = TIME.write().unwrap();
            time.startup = Instant::now();
            time.last_update = None;
        } else if chose == 1 {
            self.show_ranking = true;
        } else if chose == 2 {
            self.exit = true;
        }
        self.dead_dialog_chose
            .store(-1, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn get_save(&self) -> Player {
        Player {
            id: self.high_score.id,
            name: self.high_score.name.to_owned(),
            score: self.score,
            time: self.play_time.as_secs() as i64,
            timestamp: self.dead_time,
        }
    }
}

impl Activity for GameplayActivity {
    fn draw(&mut self, frame: &mut Frame<'_>) {
        self.gameplay_draw(frame);
    }

    fn update(&mut self, event: Option<Event>) {
        {
            let time = TIME.read().unwrap();
            self.app_time += time.delta;
            if !self.game_over && self.play_started {
                self.play_time += time.delta;
            }
        }

        if let Some(event) = event {
            self.gameplay_update_input(event);
        }

        if self.game_over {
            if !self.dead_dialog && self.app_time > self.dead_dialog_time {
                self.dead_dialog = true;
                self.dead_time = get_time_millis();
                self.queue_clear_message();
            }
            self.update_clear_chose();
        }
    }
}
