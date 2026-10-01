use crate::GameResult;
use crate::tactics::commands::{BattleData, Command};
use crate::tactics::units::*;

#[derive(Debug, Clone)]
pub struct AttackCommand {
    pub defender: Defender,
    pub damage: usize,
}

impl Command for AttackCommand {
    fn description(&self) -> &str {
        "attacked"
    }
    fn tick_duration(&self) -> u32 {
        15
    }
    fn apply(&self, state: &mut BattleData) -> GameResult<()> {
        let (defender_uid, defender_position) = self.defender;
        let Some(defender) = state.map_state.units.get_mut(&defender_uid) else {
            return Err(
                format!("defending unit does not exist in units list {defender_uid:?}").into(),
            );
        };
        defender.health = defender.health.saturating_sub(self.damage);
        if defender.health == 0 {
            state
                .map_instance
                .get_mut(defender_position)
                .and_then(|tile| tile.occupant.take());
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
        let new_health = defender.health.saturating_add(self.damage);
        if defender.health == 0 && new_health > 0 {
            state.map_instance.get_mut(defender_position).map(|tile| {
                tile.occupant = Some(defender_uid);
            });
        }
        defender.health = new_health;
        Ok(())
    }
}
