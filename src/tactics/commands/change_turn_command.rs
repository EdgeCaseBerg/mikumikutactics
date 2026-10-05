use crate::GameResult;
use crate::tactics::commands::{BattleData, Command, units_of};
use crate::tactics::units::*;

#[derive(Debug, Clone)]
pub struct ChangeTurnCommand {
    pub to_faction: Faction,
    pub from_faction: Faction,
}

impl Command for ChangeTurnCommand {
    fn description(&self) -> &str {
        match self.to_faction {
            Faction::Player => "Player turn started",
            Faction::Computer => "Computer turn started",
        }
    }
    fn tick_duration(&self) -> u32 {
        15
    }
    fn apply(&self, state: &mut BattleData) -> GameResult<()> {
        state.active_faction = self.to_faction;
        state.untapped_units = units_of(&state.map_state, self.to_faction);
        Ok(())
    }
    fn undo(&self, state: &mut BattleData) -> GameResult<()> {
        state.active_faction = self.from_faction;
        state.untapped_units = units_of(&state.map_state, self.from_faction);
        Ok(())
    }
}
