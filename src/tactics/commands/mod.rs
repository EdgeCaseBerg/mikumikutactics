use crate::GameResult;
use crate::tactics::map::*;
use crate::tactics::units::*;
use std::collections::HashSet;

pub mod attack_command;
pub mod change_turn_command;
pub mod command_log;
pub mod move_command;
// re-export for easier use.
pub use self::attack_command::AttackCommand;
pub use self::change_turn_command::ChangeTurnCommand;
pub use self::command_log::CommandLog;
pub use self::command_log::CommandLogState;
pub use self::command_log::UnitAction;
pub use self::move_command::MoveCommand;

#[derive(Debug)]
pub struct BattleData {
    pub map_instance: MapInstance,
    pub map_state: MapState,
    pub untapped_units: HashSet<UnitId>,
    pub active_faction: Faction,
}

pub fn units_of(map_state: &MapState, faction: Faction) -> HashSet<UnitId> {
    let mut set = HashSet::new();
    for (unit_id, unit_data) in &map_state.units {
        if unit_data.faction == faction && unit_data.health > 0 {
            set.insert(*unit_id);
        }
    }
    set
}

pub fn loser(map_state: &MapState) -> Option<Faction> {
    let mut player_health = 0;
    let mut computer_health = 0;
    for (unit_id, unit_data) in &map_state.units {
        match (unit_data.health, unit_data.faction) {
            (h, Faction::Player) => player_health += h,
            (h, Faction::Computer) => computer_health += h,
        }
    }
    // Bias towards being nicer to the human by returning cpu first
    match (computer_health == 0, player_health == 0) {
        (true, _) => Some(Faction::Computer),
        (_, true) => Some(Faction::Player),
        _ => None,
    }
}

pub trait Command {
    fn tick_duration(&self) -> u32;
    fn apply(&self, state: &mut BattleData) -> GameResult<()>;
    fn undo(&self, state: &mut BattleData) -> GameResult<()>;
    fn description(&self) -> &str;
}
