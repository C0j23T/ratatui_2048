use crate::app::structs::Player;

pub trait DataManager: Default {
    fn is_first_launch(&mut self) -> bool;

    fn get_current_player(&mut self) -> Player;

    fn get_players_best_except_self(&mut self) -> Vec<Player>;

    fn save_current_player(&mut self, player: Player) -> bool;
}

#[derive(Default)]
pub struct DummyDataManager;

impl DataManager for DummyDataManager {
    fn is_first_launch(&mut self) -> bool {
        false
    }

    fn get_current_player(&mut self) -> Player {
        Player::default()
    }

    fn get_players_best_except_self(&mut self) -> Vec<Player> {
        vec![
            Player {
                id: 123,
                name: String::from("DARE"),
                score: 256,
                time: 114,
                timestamp: 1145141919810,
            };
            100
        ]
    }

    fn save_current_player(&mut self, _: Player) -> bool {
        true
    }
}
