use crate::GameResult;
use crate::ReadyState;
use crate::tactics::commands::{BattleData, Command};
use crate::tactics::units::*;

#[derive(Default)]
pub struct CommandLog {
    commands: Vec<Box<dyn Command>>,
    unit_actions: Vec<UnitAction>,
    index: usize,
    ticker: Option<ReadyState>,
}

// Note that offsets are inclusive. So 1..=1 for a single action like move to x
// and something like 1..=2 for something like move, then attack
#[derive(Debug, PartialEq)]
pub struct UnitAction {
    pub unit_id: UnitId,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug)]
pub enum CommandLogState {
    Exhausted,
    Running,
}

impl CommandLog {
    pub fn add_unit_actions(&mut self, unit_id: UnitId, commands: Vec<Box<dyn Command>>) {
        let start_offset = self.commands.len();
        let end_offset = start_offset + commands.len() - 1;
        for c in commands {
            self.commands.push(c);
        }
        self.unit_actions.push(UnitAction {
            unit_id,
            start_offset,
            end_offset,
        });
    }
    pub fn get_unit_actions(&self) -> std::slice::Iter<'_, UnitAction> {
        self.unit_actions.iter()
    }
    pub fn get_command_at(&self, position: usize) -> Option<&dyn Command> {
        self.commands.get(position).map(|c| c.as_ref())
    }
    pub fn has_next(&self) -> bool {
        self.commands.get(self.index).is_some()
    }
    pub fn update(&mut self, ticks: u32, state: &mut BattleData) -> GameResult<CommandLogState> {
        // note that the update command must be called at least once per command, you can't process multiple commands in a single step
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
                    self.ticker = None;
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
                command.undo(state)?;
            }

            self.unit_actions
                .pop_if(|UnitAction { start_offset, .. }| *start_offset == self.index);

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

    struct FakeCommand {}
    impl Command for FakeCommand {
        fn tick_duration(&self) -> u32 {
            1
        }
        fn apply(&self, state: &mut BattleData) -> GameResult<()> {
            eprintln!("apply");
            Ok(())
        }
        fn undo(&self, state: &mut BattleData) -> GameResult<()> {
            eprintln!("undo");
            Ok(())
        }
        fn description(&self) -> &str {
            "fake"
        }
    }

    fn fake_command() -> FakeCommand {
        FakeCommand {}
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
            vec![Box::new(fake_command()), Box::new(fake_command())],
        );
        assert_eq!(true, cl.has_next());
        cl.add_unit_actions(UnitId(1), vec![Box::new(fake_command())]);
        let mut actions = cl.get_unit_actions();
        eprintln!("{:?}", actions);
        assert_eq!(
            &UnitAction {
                unit_id: UnitId(0),
                start_offset: 0,
                end_offset: 1
            },
            actions.next().expect("unexpected empty option")
        );
        assert_eq!(
            &UnitAction {
                unit_id: UnitId(1),
                start_offset: 2,
                end_offset: 2
            },
            actions.next().expect("unexpected empty option")
        );
    }

    #[test]
    fn command_log_applys_commands_correctly() {
        let mut state = BattleData {
            map_state: MapState::default(),
            map_instance: MapInstance::empty_walkable(10, 10),
        };
        let mut cl = CommandLog::default();
        cl.add_unit_actions(
            UnitId(0),
            vec![Box::new(fake_command()), Box::new(fake_command())],
        );
        // Prime the None -> Some
        cl.update(1, &mut state).unwrap();
        // Consume the command:
        match cl.update(1, &mut state) {
            Ok(CommandLogState::Running) => {}
            other => {
                panic!("Expected running command log bug got {:?}", other);
            }
        }
        // Prime the None -> Some
        cl.update(1, &mut state).unwrap();
        // Consume the command
        match cl.update(1, &mut state) {
            Ok(CommandLogState::Running) => {}
            other => {
                panic!("Expected running command log bug got {:?}", other);
            }
        }
        match cl.update(2, &mut state) {
            Ok(CommandLogState::Exhausted) => {}
            other => {
                panic!("Expected exhausted command log bug got {:?}", other);
            }
        }
        cl.add_unit_actions(UnitId(0), vec![Box::new(fake_command())]);
        // Prime the None -> Some
        cl.update(1, &mut state).unwrap();
        // Consume the command
        match cl.update(2, &mut state) {
            Ok(CommandLogState::Running) => {}
            other => {
                panic!("Expected running command log bug got {:?}", other);
            }
        }
        // Prime the None -> Some
        cl.update(1, &mut state).unwrap();
        // Consume the command
        match cl.update(2, &mut state) {
            Ok(CommandLogState::Exhausted) => {}
            other => {
                panic!("Expected exhausted command log bug got {:?}", other);
            }
        }
    }

    #[test]
    fn command_log_offsets_rewind_correctly() {
        let mut state = BattleData {
            map_state: MapState::default(),
            map_instance: MapInstance::empty_walkable(10, 10),
        };
        let mut cl = CommandLog::default();
        cl.add_unit_actions(
            UnitId(0),
            vec![Box::new(fake_command()), Box::new(fake_command())],
        );
        cl.add_unit_actions(
            UnitId(1),
            vec![
                Box::new(fake_command()),
                Box::new(fake_command()),
                Box::new(fake_command()),
            ],
        );
        cl.add_unit_actions(UnitId(2), vec![Box::new(fake_command())]);

        eprintln!("ticking actions forward");
        for _ in 0..12 {
            cl.update(1, &mut state).unwrap();
        }

        let actions = cl.get_unit_actions();
        assert_eq!(3, actions.len(), "actions are {:?}", actions);
        eprintln!("{:?}", actions);
        let UnitAction { start_offset, .. } = actions.last().unwrap();
        eprintln!("told to rewind to {}", start_offset);
        cl.rewind(*start_offset, &mut state).unwrap();

        let actions = cl.get_unit_actions();
        assert_eq!(2, actions.len(), "actions are {:?}", actions);
        let UnitAction { start_offset, .. } = actions.last().unwrap();
        eprintln!("told to rewind to {}", start_offset);
        cl.rewind(*start_offset, &mut state).unwrap();

        let actions = cl.get_unit_actions();
        let UnitAction { start_offset, .. } = actions.last().unwrap();
        eprintln!("told to rewind to {}", start_offset);
        cl.rewind(*start_offset, &mut state).unwrap();
        let actions = cl.get_unit_actions();
        assert_eq!(0, actions.len(), "actions are {:?}", actions);

        assert_eq!(false, cl.has_next());
    }
}
