use crate::GameResult;
use crate::tactics::commands::{BattleData, Command};
use crate::tactics::map::*;
use crate::tactics::units::*;

#[derive(Debug, Clone)]
pub struct MoveCommand {
    pub unit_id: UnitId,
    pub from: Coordinate,
    pub to: Coordinate,
}

impl Command for MoveCommand {
    fn description(&self) -> &str {
        "moved"
    }
    fn tick_duration(&self) -> u32 {
        15
    }
    fn apply(&self, state: &mut BattleData) -> GameResult<()> {
        eprintln!("move command apply");
        state
            .map_instance
            .move_unit_to(self.unit_id, self.from, self.to)?;
        Ok(())
    }
    fn undo(&self, state: &mut BattleData) -> GameResult<()> {
        state
            .map_instance
            .move_unit_to(self.unit_id, self.to, self.from)?;
        Ok(())
    }
}
