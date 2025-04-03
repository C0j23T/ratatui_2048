use crate::app::structs::Player;

pub enum TryRecvError {
    Empty,
    Timeout,
    Disconnect,
}

pub trait DataManager: Send + Sync {
    fn is_first_launch(&mut self) -> bool;

    fn verify_account(
        &mut self,
        username: String,
        password: String,
    ) -> Result<Option<Player>, TryRecvError>;

    fn register_account(
        &mut self,
        username: String,
        password: String,
    ) -> Result<Option<Player>, TryRecvError>;

    fn get_current_player(&mut self) -> Result<Player, TryRecvError>;

    fn get_players_best_except_self(&mut self) -> Result<Vec<Player>, TryRecvError>;

    fn get_players_except_self(&mut self) -> Result<Vec<Player>, TryRecvError>;

    fn save_current_player(&mut self, player: Player) -> Result<bool, TryRecvError>;

    fn find_player(&mut self, player: Player) -> Result<Vec<Player>, TryRecvError>;

    fn update_player(&mut self, player: Player) -> Result<bool, TryRecvError>;

    fn remove_player(&mut self, player: Player) -> Result<bool, TryRecvError>;
}

#[derive(Default)]
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
            records: Vec::default(),
        });
        for i in 0..100 {
            players.push(Player {
                id: i + 1,
                name: String::from("DARE"),
                best_score: i * 100,
                best_time: (i * 100) as i64,
                best_timestamp: 1145141919810,
                records: Vec::default(),
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

    fn get_players_except_self(&mut self) -> Result<Vec<Player>, TryRecvError> {
        Ok(Self::gen_example_players())
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
