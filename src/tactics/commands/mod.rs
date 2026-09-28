use crate::GameResult;
use crate::ReadyState;
use crate::tactics::map::*;
use crate::tactics::units::*;
use std::collections::HashSet;

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

pub struct UnitActions {
    pub unit_id: UnitId,
    pub commands: Vec<Box<dyn Command>>,
}

pub struct PlayerTurn {
    pub units_tapped: HashSet<UnitId>,
    pub unit_actions: Vec<UnitActions>,
}

pub struct ActionRunner {
    pub action: UnitActions,
    index: usize,
    ticker: Option<ReadyState>,
}

impl ActionRunner {
    pub fn new(action: UnitActions) -> Self {
        ActionRunner {
            action,
            index: 0,
            ticker: None,
        }
    }
}

#[derive(Debug)]
pub enum ActionRunnerState {
    Exhausted,
    Running,
}

impl ActionRunner {
    pub fn has_next(&self) -> bool {
        self.action.commands.get(self.index).is_some()
    }
    pub fn update(&mut self, ticks: u32, state: &mut BattleData) -> GameResult<ActionRunnerState> {
        if !self.has_next() {
            return Ok(ActionRunnerState::Exhausted);
        }
        let command = self.action.commands.get(self.index).expect("has_next lied");
        match self.ticker.as_mut() {
            None => {
                self.ticker = Some(ReadyState::Cooldown {
                    wait_for: command.tick_duration(),
                    ticks_waited: 0,
                });
            }
            Some(ready_state) => {
                *ready_state = ready_state.advance(ticks);
                if matches!(ready_state, ReadyState::Ready) {
                    self.apply(state);
                }
            }
        }
        Ok(ActionRunnerState::Running)
    }
    fn apply(&mut self, state: &mut BattleData) -> GameResult<()> {
        if let Some(command) = self.action.commands.get(self.index).as_ref() {
            let _ = command.apply(state)?;
            self.index = self.index.saturating_add(1);
        }
        Ok(())
    }
    // should we call this undo_one because it doesn't undo the whole thing after all
    // should we maybe have a rewind that runs an update loop in reverse? should it tick?
    fn undo(&mut self, state: &mut BattleData) -> GameResult<()> {
        if let Some(command) = self.action.commands.get(self.index).as_ref() {
            let _ = command.undo(state)?;
            self.index = self.index.saturating_sub(1);
        }
        Ok(())
    }
}
