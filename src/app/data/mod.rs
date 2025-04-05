use crate::app::structs::Player;

pub mod jni;
pub mod dummy;

pub enum TryRecvError {
    Empty,
    Timeout,
    Disconnected,
}

pub trait DataManager: Send {
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

    fn save_record(&mut self, player: Player) -> Result<bool, TryRecvError>;

    fn find_player(&mut self, player: Player) -> Result<Vec<Player>, TryRecvError>;

    fn update_player(&mut self, player: Player) -> Result<bool, TryRecvError>;

    fn remove_player(&mut self, player: Player) -> Result<bool, TryRecvError>;
}
