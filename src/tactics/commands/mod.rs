use crate::GameResult;
use crate::tactics::map::*;
use crate::tactics::units::*;
use std::collections::HashSet;

pub mod attack_command;
pub mod command_log;
pub mod move_command;
// re-export for easier use.
pub use self::attack_command::AttackCommand;
pub use self::command_log::CommandLog;
pub use self::command_log::CommandLogState;
pub use self::command_log::UnitAction;
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
    fn description(&self) -> &str;
}

pub struct UnitActions {
    pub unit_id: UnitId,
    pub commands: Vec<Box<dyn Command>>,
}

pub struct PlayerTurn {
    pub units_tapped: HashSet<UnitId>,
    pub unit_actions: Vec<UnitActions>,
}
