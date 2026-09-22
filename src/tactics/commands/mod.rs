use crate::GameResult;
use crate::tactics::map::*;
use crate::tactics::units::*;

pub mod move_command;
// re-export for easier use.
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

pub struct AttackCommand {
    pub defender: Defender,
    pub old_health: usize,
    pub new_health: usize,
}

impl Command for AttackCommand {
    fn tick_duration(&self) -> u32 {
        15
    }
    fn apply(&self, state: &mut BattleData) -> GameResult<()> {
        eprintln!("attack command apply");
        let (defender_uid, defender_position) = self.defender;
        let Some(defender) = state.map_state.units.get_mut(&defender_uid) else {
            return Err(
                format!("defending unit does not exist in units list {defender_uid:?}").into(),
            );
        };
        defender.health = self.new_health;
        eprintln!("defender now has health {:?}", defender.health);
        if defender.health == 0 {
            state
                .map_instance
                .get_mut(defender_position)
                .and_then(|tile| tile.occupant.take());
            eprintln!("unit {:?} defeated", defender_uid);
        }
        Ok(())
    }
    fn undo(&self, state: &mut BattleData) -> GameResult<()> {
        let (defender_uid, defender_position) = self.defender;
        let Some(defender) = state.map_state.units.get_mut(&defender_uid) else {
            return Err(
                format!("defending unit does not exist in units list {defender_uid:?}").into(),
            );
        };
        defender.health = self.old_health;
        eprintln!("defender now has health {:?}", defender.health);
        if self.new_health == 0 {
            state.map_instance.get_mut(defender_position).map(|tile| {
                tile.occupant = Some(defender_uid);
            });
            eprintln!("unit {:?} defeated", defender_uid);
        }

        Ok(())
    }
}
