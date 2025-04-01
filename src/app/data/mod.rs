use crate::app::structs::Player;

pub enum TryRecvError {
    Empty,
    Timeout,
    Disconnect,
}

pub trait DataManager: Send {
    fn is_first_launch(&mut self) -> bool;

    fn verify_account(&mut self, username: String, password: String) -> Result<Option<Player>, TryRecvError>;

    fn register_account(&mut self, username: String, password: String) -> Result<Option<Player>, TryRecvError>;

    fn get_current_player(&mut self) -> Result<Player, TryRecvError>;

    fn get_players_best_except_self(&mut self) -> Result<Vec<Player>, TryRecvError>;

    fn save_current_player(&mut self, player: Player) -> Result<bool, TryRecvError>;
}

#[derive(Default)]
pub struct DummyDataManager;

impl DataManager for DummyDataManager {
    fn is_first_launch(&mut self) -> bool {
        false
    }

    fn get_current_player(&mut self) -> Result<Player, TryRecvError> {
        Ok(Player::default())
    }

    fn get_players_best_except_self(&mut self) -> Result<Vec<Player>, TryRecvError> {
        Ok(vec![
            Player {
                id: 123,
                name: String::from("DARE"),
                score: 256,
                time: 114,
                timestamp: 1145141919810,
            };
            100
        ])
    }

    fn save_current_player(&mut self, _: Player) -> Result<bool, TryRecvError> {
        Ok(true)
    }

    fn verify_account(&mut self, _: String, _: String) -> Result<Option<Player>, TryRecvError> {
        Ok(Some(Player::default()))
    }

    fn register_account(&mut self, _: String, _: String) -> Result<Option<Player>, TryRecvError> {
        Ok(Some(Player::default()))
    }
}
