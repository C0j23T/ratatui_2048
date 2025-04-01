use std::{io::Result, time::Duration};

use crossterm::event::{self, Event};
use dialog::DIALOG_MANAGER;
use ratatui::{Frame, Terminal, prelude::Backend};

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
pub struct App<'a> {
    state: AppState,
    pub state_changed: bool,

    gameplay_activity: Option<gameplay::GameplayActivity>,
    ranking_activity: Option<ranking::RankingActivity>,
    menu_activity: Option<menu::MenuActivity<'a>>,
    gameplay_move_save: bool,
}

impl App<'_> {
    pub fn new() -> Self {
        Self {
            state_changed: true,
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
                _ => todo!(),
            };

            let mut dialog_manager = DIALOG_MANAGER.write().unwrap();
            dialog_manager.draw(frame);
        })?;

        if self.state_changed == last_state_changed {
            self.state_changed = false;
        }
        if matches!(self.state, AppState::Quit) {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn update_menu(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            if let Some(ref mut menu_activity) = self.menu_activity {
                menu_activity.exiting_activity();
            } else {
                self.menu_activity = Some(menu::MenuActivity::new());
            }
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

        if menu.exit {
            self.change_state(AppState::Quit);
        }
    }

    fn update_gameplay(&mut self, frame: &mut Frame<'_>, event: Option<Event>) {
        if self.state_changed {
            self.gameplay_activity = Some(gameplay::GameplayActivity::new());
            self.ranking_activity = Some(ranking::RankingActivity::new());
        }

        let gameplay = self.gameplay_activity.as_mut().unwrap();

        if !gameplay.show_ranking {
            gameplay.draw(frame);
            gameplay.update(event);
            if gameplay.exit {
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
