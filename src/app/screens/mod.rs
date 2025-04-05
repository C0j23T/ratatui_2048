use std::{io::Result, time::Duration};

use crossterm::event::{self, Event};
use dialog::DIALOG_MANAGER;
use ratatui::{Frame, Terminal, prelude::Backend};

use crate::data_manager;

pub(crate) mod dialog;
mod find_player;
mod gameplay;
mod manage;
mod menu;
mod oobe;
mod simple_ranking;

pub trait Activity {
    fn draw(&mut self, frame: &mut Frame<'_>);

    fn update(&mut self, event: Option<Event>);
}

#[derive(Default)]
pub enum AppState {
    FirstLaunch,
    #[default]
    MainMenu,
    Gameplay,
    SwitchPlayer,
    ManagePlayer,
    Ranking,
    Exit,
}

#[derive(Default)]
pub struct App<'a> {
    state: AppState,
    pub state_changed: bool,
    first_launch: bool,

    gameplay_activity: Option<gameplay::GameplayActivity>,
    ranking_activity: Option<simple_ranking::RankingActivity>,
    menu_activity: Option<menu::MenuActivity<'a>>,
    oobe_activity: Option<oobe::OobeActivity<'a>>,
    remove_activity: Option<manage::ManageActivity<'a>>,
    gameplay_move_save: bool,
}

impl App<'_> {
    pub fn new(first_launch: bool) -> Self {
        Self {
            first_launch,
            state_changed: true,
            state: if first_launch {
                AppState::FirstLaunch
            } else {
                AppState::default()
            },
            ..Default::default()
        }
    }

    pub fn change_state(&mut self, state: AppState) {
        self.state = state;
        self.state_changed = true;
    }
}

impl App<'_> {
    pub fn update<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<bool> {
        crate::app::time::update_time();

        let event = if event::poll(Duration::from_millis(20))? {
            Some(event::read()?)
        } else {
            None
        };

        let last_state_changed = self.state_changed;

        terminal.draw(|frame| {
            let has_dialog = {
                let mut dialog_manager = DIALOG_MANAGER.write().unwrap();
                if let Some(event) = event.clone() {
                    dialog_manager.update_input(event);
                }
                dialog_manager.has_dialog()
            };
            let event = if !has_dialog { event } else { None };

            match self.state {
                AppState::Gameplay => self.update_gameplay(frame, event),
                AppState::MainMenu => self.update_menu(frame, event),
                AppState::SwitchPlayer => {
                    if !last_state_changed {
                        self.menu_activity = None;
                        self.change_state(AppState::MainMenu);
                    }
                }
                AppState::Ranking => self.update_ranking(frame, event),
                AppState::FirstLaunch => self.update_oobe(frame, event),
                AppState::ManagePlayer => self.update_remove(frame, event),
                _ => todo!(),
            };

            let mut dialog_manager = DIALOG_MANAGER.write().unwrap();
            dialog_manager.draw(frame);
        })?;

        if self.state_changed == last_state_changed {
            self.state_changed = false;
        }
        if matches!(self.state, AppState::Exit) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn update_remove(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            self.remove_activity = Some(manage::ManageActivity::new());
        }

        let remove = self.remove_activity.as_mut().unwrap();
        remove.draw(frame);
        remove.update(event);

        if remove.should_exit {
            self.change_state(AppState::MainMenu);
            let x = std::mem::take(&mut self.remove_activity);
            drop(x);
        }
    }

    fn update_oobe(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            self.oobe_activity = Some(oobe::OobeActivity::new());
            self.menu_activity = Some(menu::MenuActivity::new(false));
        }

        let oobe = self.oobe_activity.as_mut().unwrap();
        let render_menu = oobe.render_menu;
        if render_menu {
            let menu = self.menu_activity.as_mut().unwrap();
            menu.draw(frame);
            menu.update(None);
        }
        oobe.draw(frame);
        oobe.update(event);

        if oobe.should_exit {
            self.change_state(AppState::Exit);
        } else if oobe.should_skip {
            self.change_state(AppState::MainMenu);
            let x = std::mem::take(&mut self.oobe_activity);
            drop(x);
        }
    }

    fn update_ranking(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            self.ranking_activity = Some(simple_ranking::RankingActivity::new());
        }

        let ranking = self.ranking_activity.as_mut().unwrap();
        if !self.gameplay_move_save {
            if let Some(player) = data_manager!(get_current_player) {
                self.gameplay_move_save = true;
                ranking.set_save(player);
                ranking.by_score();
            }
        }

        ranking.draw(frame);
        ranking.update(event);

        if ranking.should_exit {
            self.gameplay_move_save = false;
            self.ranking_activity = None;
            let x = std::mem::take(&mut self.ranking_activity);
            drop(x);
            self.change_state(AppState::MainMenu);
        }
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            if let Some(ref mut menu_activity) = self.menu_activity {
                if !self.first_launch {
                    menu_activity.exiting_activity();
                }
            } else {
                self.menu_activity = Some(menu::MenuActivity::new(!self.first_launch));
            }
            self.first_launch = false;
        }
        let menu = self.menu_activity.as_mut().unwrap();
        menu.draw(frame);
        menu.update(event);

        if menu.can_enter_another_activity() {
            if let Some(next_state) = menu.next_state() {
                self.change_state(next_state);
                return;
            }
        }

        if menu.should_exit {
            self.change_state(AppState::Exit);
        }
    }

    fn update_gameplay(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            self.gameplay_activity = Some(gameplay::GameplayActivity::new());
            self.ranking_activity = Some(simple_ranking::RankingActivity::new());
        }

        let gameplay = self.gameplay_activity.as_mut().unwrap();

        if !gameplay.show_ranking {
            gameplay.draw(frame);
            gameplay.update(event);
            if gameplay.should_exit && gameplay.record_saved {
                self.change_state(AppState::MainMenu);
                let x = std::mem::take(&mut self.gameplay_activity);
                drop(x);
                let x = std::mem::take(&mut self.ranking_activity);
                drop(x);
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

            if ranking.should_exit {
                ranking.reset();
                self.gameplay_move_save = false;
                gameplay.show_ranking = false;
                gameplay.queue_clear_message();
            }
        }
    }
}
