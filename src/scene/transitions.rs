use crate::ReadyState;
use crate::game::GameContext;
use crate::interpolate;
use crate::scene::Scene;
use std::cmp::min;

/*
 * Transitions are just little state machines that go from one step to the other,
 * and when they're all done and have no more steps left to do, the transition is
 * complete and they queue up the next incoming scene to replace them. You can chain
 * them together as you please if you want to do multiple things. Weeee!
 */

pub enum Transition {
    Instant,
    FadeToBlack {
        fade_out_ticks: u32,
        fade_in_ticks: u32,
    }, // We could do a generic FadeTo if we wanted maybe for fade to white or similar.
}

impl Transition {
    fn get_initial_step_timers(&self) -> Vec<ReadyState> {
        match self {
            Transition::Instant => vec![ReadyState::Ready],
            Transition::FadeToBlack {
                fade_out_ticks,
                fade_in_ticks,
            } => vec![
                ReadyState::Cooldown {
                    ticks_waited: 0,
                    wait_for: *fade_out_ticks,
                },
                ReadyState::Cooldown {
                    ticks_waited: 0,
                    wait_for: *fade_in_ticks,
                },
            ],
        }
    }
}

pub struct SceneChange {
    pub next_scene: Box<dyn Scene>,
    pub transition: Transition,
}

pub struct TransitionScene {
    outgoing: Box<dyn Scene>,
    incoming: Box<dyn Scene>,
    transition: Transition,
    step_timers: Vec<ReadyState>,
    step: usize,
}

impl TransitionScene {
    fn new(outgoing: Box<dyn Scene>, incoming: Box<dyn Scene>, transition: Transition) -> Self {
        TransitionScene {
            outgoing,
            incoming,
            transition,
            step_timers: vec![],
            step: 0,
        }
    }
}

impl Scene for TransitionScene {
    fn init(&mut self, _game_context: &mut GameContext) {
        self.step = 0;
        self.step_timers = self.transition.get_initial_step_timers();
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) {
        if self.step >= self.step_timers.len() {
            if !game_context.next_scene.is_none() {
                // If the next_scene is set, but we're still calling update on the transition,
                // it's likely a ticks == 0 situation where the frames are waiting for the next
                // true draw update or similar.
                return;
            }

            // otherwise we're out of steps in the state machine, so let the incoming scene take
            // the wheel and drive. Now, you'd like to basically do this:
            //  game_context.next_scene = Some(self.incoming);
            // but we can't because of the lifetime rules and that _this_ scene we're in is holding
            // onto the incoming scene still. So instead, we rely on the event loop run implementation
            // that must handle an Instant transition appropriately and drop this transition after
            // taking the incoming scene from it.
            self.transition = Transition::Instant;
            self.init(game_context);
            return;
        }

        // Move the state machine along
        self.step_timers[self.step] = self.step_timers[self.step].advance(ticks);
        let previous_step_state = self.step;
        if let ReadyState::Ready = self.step_timers[self.step] {
            self.step = min(self.step + 1, self.step_timers.len());
        }

        // Do any step specific logic.
        match self.transition {
            Transition::FadeToBlack { .. } => match self.step {
                0 => {
                    self.outgoing.update(ticks, game_context);
                }
                1 => {
                    if previous_step_state != self.step {
                        self.incoming.init(game_context);
                    }
                    // To avoid feeling like the game isn't responsive, start running the update
                    // for the incoming stream once the fade is _almost_ done. 50% ought to be ok
                    // for now we can fine tune it later or do it based off an absolute number of
                    // game ticks if that works better once we have something real to base it off of
                    let early_update = match self.step_timers[self.step] {
                        ReadyState::Ready => true,
                        ReadyState::Cooldown {
                            wait_for,
                            ticks_waited,
                        } => {
                            let total_wait_time = wait_for;
                            let allow_updating_mark = wait_for / 2;
                            total_wait_time.saturating_sub(ticks_waited) < allow_updating_mark
                        }
                    };
                    if early_update {
                        self.incoming.update(ticks, game_context);
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
    fn draw(&mut self, _game_context: &mut GameContext) {
        todo!();
    }
}
