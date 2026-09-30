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

#[derive(Default)]
pub struct CommandLog {
    commands: Vec<Box<dyn Command>>,
    unit_actions: Vec<(UnitId, usize)>, // stack to undo
    index: usize,
    ticker: Option<ReadyState>,
}

#[derive(Debug)]
pub enum CommandLogState {
    Exhausted,
    Running,
}

impl CommandLog {
    pub fn add_unit_actions(&mut self, unit_id: UnitId, commands: Vec<Box<dyn Command>>) {
        let offset = self.commands.len();
        for c in commands {
            self.commands.push(c);
        }
        self.unit_actions.push((unit_id, offset));
    }
    pub fn get_unit_actions(&self) -> std::slice::Iter<'_, (UnitId, usize)> {
        self.unit_actions.iter()
    }
    pub fn has_next(&self) -> bool {
        self.commands.get(self.index).is_some()
    }
    pub fn update(&mut self, ticks: u32, state: &mut BattleData) -> GameResult<CommandLogState> {
        if !self.has_next() {
            return Ok(CommandLogState::Exhausted);
        }
        let command = self.commands.get(self.index).expect("has_next lied");
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
                    let _ = command.apply(state);
                    self.index = self.index.saturating_add(1);
                }
            }
        }
        Ok(CommandLogState::Running)
    }
    pub fn rewind(&mut self, to: usize, state: &mut BattleData) -> GameResult<CommandLogState> {
        // instant rewind for now.
        while self.index >= to {
            self.index = self.index.saturating_sub(1);
            if let Some(command) = self.commands.pop() {
                let _ = command.undo(state)?;
            }

            self.unit_actions
                .pop_if(|(uid, offset)| *offset == self.index);

            if self.index == 0 || self.index < to {
                break;
            }
        }
        Ok(CommandLogState::Exhausted)
    }
}

#[cfg(test)]
mod command_log_units {
    use super::*;

    fn move_command(id: usize) -> MoveCommand {
        MoveCommand {
            unit_id: UnitId(id),
            from: (0, 0),
            to: (0, 1),
        }
    }

    #[test]
    fn empty_command_log_has_no_next_item() {
        let cl = CommandLog::default();
        assert_eq!(false, cl.has_next());
        assert_eq!(0, cl.get_unit_actions().len());
    }

    #[test]
    fn command_log_can_be_added_at_correct_offsets() {
        let mut cl = CommandLog::default();
        cl.add_unit_actions(
            UnitId(0),
            vec![Box::new(move_command(0)), Box::new(move_command(0))],
        );
        assert_eq!(true, cl.has_next());
        cl.add_unit_actions(UnitId(1), vec![Box::new(move_command(0))]);
        let mut actions = cl.get_unit_actions();
        eprintln!("{:?}", actions);
        assert_eq!(
            &(UnitId(0), 0),
            actions.next().expect("unexpected empty option")
        );
        assert_eq!(
            &(UnitId(1), 2),
            actions.next().expect("unexpected empty option")
        );
    }
}
