use crate::GameResult;
use crate::tactics::map::*;
use crate::tactics::units::*;

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

pub struct MoveCommand {
    pub unit_id: UnitId,
    pub from: Coordinate,
    pub to: Coordinate,
}

impl Command for MoveCommand {
    fn tick_duration(&self) -> u32 {
        15
    }
    fn apply(&self, state: &mut BattleData) -> GameResult<()> {
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
