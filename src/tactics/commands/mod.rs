use crate::GameResult;
use crate::tactics::map::*;
use crate::tactics::units::*;

pub mod attack_command;
pub mod move_command;
// re-export for easier use.
pub use self::attack_command::AttackCommand;
pub use self::move_command::MoveCommand;

#[derive(Debug)]
pub struct BattleData {
    pub map_instance: MapInstance,
    pub map_state: MapState,
}

pub trait Command {
    fn tick_duration(&self) -> u32;
    fn apply(&self, state: &mut BattleData) -> GameResult<()>;
    fn undo(&self, state: &mut BattleData) -> GameResult<()>;
}
