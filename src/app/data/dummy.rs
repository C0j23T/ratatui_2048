use crate::app::structs::{Player, PlayerRecord};

use super::{DataManager, TryRecvError};

pub struct DummyDataManager;

impl DummyDataManager {
    fn gen_example_players() -> Vec<Player> {
        let mut players = Vec::new();
        players.push(Player {
            id: 0,
            name: String::from("Flash"),
            best_score: 10000,
            best_time: 1,
            best_timestamp: 1145141919810,
            records: vec![
                PlayerRecord {
                    score: 10000,
                    time: 1,
                    timestamp: 1145141919810,
                },
                PlayerRecord {
                    score: 1500,
                    time: 300,
                    timestamp: 1000000000000,
                },
                PlayerRecord {
                    score: 2400,
                    time: 300,
                    timestamp: 900000000000,
                },
                PlayerRecord {
                    score: 2300,
                    time: 300,
                    timestamp: 800000000000,
                },
                PlayerRecord {
                    score: 2200,
                    time: 300,
                    timestamp: 700000000000,
                },
                PlayerRecord {
                    score: 2100,
                    time: 300,
                    timestamp: 600000000000,
                },
                PlayerRecord {
                    score: 2000,
                    time: 300,
                    timestamp: 500000000000,
                },
            ],
        });
        for i in 0..100 {
            players.push(Player {
                id: i + 1,
                name: String::from("DARE"),
                best_score: i * 100,
                best_time: (i * 100) as i64,
                best_timestamp: 1145141919810,
                records: vec![PlayerRecord {
                    score: i * 100,
                    time: (i * 100) as i64,
                    timestamp: 1145141919810,
                }],
            });
        }
        players
    }
}

impl DataManager for DummyDataManager {
    fn is_first_launch(&mut self) -> bool {
        false
    }

    fn get_current_player(&mut self) -> Result<Player, TryRecvError> {
        Ok(Player::default())
    }

    fn get_players_best_except_self(&mut self) -> Result<Vec<Player>, TryRecvError> {
        Ok(Self::gen_example_players())
    }

    fn get_players(&mut self) -> Result<Vec<Player>, TryRecvError> {
        Ok(Self::gen_example_players())
    }

    fn save_record(&mut self, _: Player) -> Result<bool, TryRecvError> {
        Ok(true)
    }

    fn verify_account(&mut self, _: String, _: String) -> Result<Option<Player>, TryRecvError> {
        Ok(Some(Player::default()))
    }

    fn register_account(&mut self, _: String, _: String) -> Result<Option<Player>, TryRecvError> {
        Ok(Some(Player::default()))
    }

    fn find_player(&mut self, _player: Player) -> Result<Vec<Player>, TryRecvError> {
        Ok(Self::gen_example_players())
    }

    fn update_player(&mut self, _player: Player) -> Result<bool, TryRecvError> {
        Ok(true)
    }

    fn remove_player(&mut self, _player: Player) -> Result<bool, TryRecvError> {
        Ok(true)
    }
}