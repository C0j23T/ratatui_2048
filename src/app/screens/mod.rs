use std::io::Result;

use ratatui::{Terminal, prelude::Backend};

use super::data::DataManager;

mod dialog;
mod gameplay;
mod menu;
mod ranking;

#[derive(Default)]
pub enum AppState {
    #[default]
    FirstStart,
    MainMenu,
    Gameplay,
    SwitchPlayer,
    RemovePlayer,
    FindPlayer,
    EditPlayer,
    ListAllPlayer,
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
    pub fn new(state: AppState, data_manager: D) -> Self {
        Self {
            state,
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

        match self.state {
            AppState::Gameplay => self.update_gameplay(terminal)?,
            _ => (),
        }

        self.state_changed = false;
        Ok(false)
    }

    fn update_gameplay<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
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
            gameplay.update(terminal)?;

            if gameplay.exit && gameplay.game_over {
                self.data_manager.save_current_player(gameplay.get_save());
                self.change_state(AppState::MainMenu);
            }
        } else {
            let ranking = self.ranking_activity.as_mut().unwrap();
            if !self.gameplay_move_save {
                self.gameplay_move_save = true;
                ranking.set_save(gameplay.get_save());
                ranking.by_score();
            }

            ranking.update(terminal)?;

            if ranking.exit {
                ranking.reset();
                self.gameplay_move_save = false;
                gameplay.show_ranking = false;
                gameplay.queue_clear_message();
            }
        }
        Ok(())
    }
}
