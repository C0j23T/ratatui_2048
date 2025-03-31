use std::{io::Result, time::Duration};

use crossterm::event::{self, Event};
use dialog::DIALOG_MANAGER;
use ratatui::{Frame, Terminal, prelude::Backend};

use super::data::DataManager;

mod dialog;
mod gameplay;
mod menu;
mod ranking;

pub trait Activity {
    fn draw(&mut self, frame: &mut Frame<'_>);

    fn update(&mut self, event: Option<Event>);
}

#[derive(Default)]
pub enum AppState {
    #[default]
    MainMenu,
    Gameplay,
    SwitchPlayer,
    RemovePlayer,
    FindPlayer,
    EditPlayer,
    ListAllPlayer,
    Quit,
}

#[derive(Default)]
pub struct App<D: DataManager> {
    state: AppState,
    data_manager: D,
    pub state_changed: bool,

    gameplay_activity: Option<gameplay::GameplayActivity>,
    ranking_activity: Option<ranking::RankingActivity>,
    gameplay_move_save: bool,
}

impl<D: DataManager> App<D> {
    pub fn new(data_manager: D) -> Self {
        Self {
            state: AppState::MainMenu,
            data_manager,
            ..Default::default()
        }
    }

    pub fn change_state(&mut self, state: AppState) {
        self.state = state;
        self.state_changed = true;
    }
}

impl<D: DataManager> App<D> {
    pub fn update<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<bool> {
        crate::app::time::update_time();

        let event = if event::poll(Duration::from_millis(20))? {
            Some(event::read()?)
        } else {
            None
        };

        terminal.draw(|frame| {
            let event_clone = event.clone();
            match self.state {
                AppState::Gameplay => self.update_gameplay(frame, event_clone),
                AppState::MainMenu => self.update_menu(frame, event_clone),
                _ => todo!(),
            };

            let mut dialog_manager = DIALOG_MANAGER.write().unwrap();
            dialog_manager.draw(frame);
            if let Some(event) = event {
                dialog_manager.update_input(event);
            }
        })?;

        self.state_changed = false;
        if matches!(self.state, AppState::Quit) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {}

    fn update_gameplay(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            self.gameplay_activity = Some(gameplay::GameplayActivity::new(
                self.data_manager.get_current_player(),
            ));
            self.ranking_activity = Some(ranking::RankingActivity::new(
                self.data_manager.get_players_best_except_self(),
            ));
        }

        let gameplay = self.gameplay_activity.as_mut().unwrap();

        if !gameplay.show_ranking {
            gameplay.draw(frame);
            gameplay.update(event);
            if gameplay.exit {
                if gameplay.game_over {
                    self.data_manager.save_current_player(gameplay.get_save());
                }

                self.change_state(AppState::MainMenu);
            }
        } else {
            let ranking = self.ranking_activity.as_mut().unwrap();
            if !self.gameplay_move_save {
                self.gameplay_move_save = true;
                ranking.set_save(gameplay.get_save());
                ranking.by_score();
            }

            ranking.draw(frame);
            ranking.update(event);

            if ranking.exit {
                ranking.reset();
                self.gameplay_move_save = false;
                gameplay.show_ranking = false;
                gameplay.queue_clear_message();
            }
        }
    }
}
