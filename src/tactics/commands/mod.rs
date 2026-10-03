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
    pub untapped_units: HashSet<UnitId>,
}

pub fn units_of(map_state: &MapState, faction: Faction) -> HashSet<UnitId> {
    let mut set = HashSet::new();
    for (unit_id, unit_data) in &map_state.units {
        if matches!(unit_data.faction, faction) {
            set.insert(*unit_id);
        }
    }
    set
}

pub trait Command {
    fn tick_duration(&self) -> u32;
    fn apply(&self, state: &mut BattleData) -> GameResult<()>;
    fn undo(&self, state: &mut BattleData) -> GameResult<()>;
    fn description(&self) -> &str;
}
