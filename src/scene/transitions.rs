use crate::ReadyState;
use crate::Rect;
use crate::game::GameContext;
use crate::interpolate;
use crate::renderer::{Color, RenderCommand};
use crate::scene::Scene;
use std::cmp::{max, min};

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
            Transition::Instant => vec![
                // we intentionally allow for 1 tick so that we can write tests for behavior
                ReadyState::Cooldown {
                    ticks_waited: 0,
                    wait_for: 0,
                },
            ],
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
    incoming: Option<Box<dyn Scene>>,
    transition: Transition,
    step_timers: Vec<ReadyState>,
    step: usize,
    prev_step: usize,
}

impl TransitionScene {
    pub fn new(outgoing: Box<dyn Scene>, incoming: Box<dyn Scene>, transition: Transition) -> Self {
        TransitionScene {
            outgoing,
            incoming: Some(incoming),
            transition,
            step_timers: vec![],
            step: 0,
            prev_step: 0,
        }
    }
}

impl TransitionScene {
    fn fade_to_black_update(&mut self, ticks: u32, game_context: &mut GameContext) {
        // When the fade out step is on its last tick, then init the incoming scene
        if self.prev_step != self.step && self.step == 1 {
            // TODO: I think we need to call the audio prepare method here or else sounds won't play.
            self.incoming.as_mut().map(|o| {
                o.init(game_context);
                // let audio = game_context.audio.as_mut();
                //         if let Some(audio) = audio {
                //             audio.prepare();
                //         }
            });

        }
        match self.prev_step {
            0 => {
                // Don't call update on the outgoing because then you end up still handling
                // events in that scene which can cause multiple Transitions to get set up
                // all at once back to back in a terible linked list sort of way that can cause
                // flashing and flickering and horror. We should make a flag to block the incoming
                // input so that we _can_ keep updating the previous scene but oh man oh man was
                // that a weird bug to track down. Once we fix that input control flag up then we
                // can uncomment this and get the tests passing again
                // self.outgoing.update(ticks, game_context);
            }
            1 => {
                // To avoid feeling like the game isn't responsive, start running the update
                // for the incoming stream once the fade is _almost_ done. 50% ought to be ok
                // for now we can fine tune it later or do it based off an absolute number of
                // game ticks if that works better once we have something real to base it off of
                let early_update = match self.step_timers[self.prev_step] {
                    ReadyState::Ready => true,
                    ReadyState::Cooldown {
                        wait_for,
                        ticks_waited,
                    } => {
                        let total_wait_time = wait_for;
                        let allow_updating_mark = wait_for / 2;
                        assert!(
                            allow_updating_mark != 0,
                            "programmer error: misconfigured wait time for cooldown was 0"
                        );
                        total_wait_time.saturating_sub(ticks_waited) < allow_updating_mark
                    }
                };
                if early_update {
                    self.incoming
                        .as_mut()
                        .map(|o| o.update(ticks, game_context));
                }
            }
            _ => {}
        }
    }
}

impl Scene for TransitionScene {
    fn init(&mut self, _game_context: &mut GameContext) {
        self.step = 0;
        self.step_timers = self.transition.get_initial_step_timers();
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) {
        if ticks == 0 {
            return;
        }

        if self.step >= self.step_timers.len() {
            if !game_context.next_scene.is_none() {
                // If the next_scene is set, but we're still calling update on the transition,
                // then we're waiting for the event loop code to swap the scene.
                return;
            }

            // otherwise we're out of steps in the state machine, so let the incoming scene take
            // the wheel and drive. Now, you'd like to basically do this:
            //  game_context.next_scene = Some(self.incoming);
            // but we can't because of the lifetime rules and that _this_ scene we're in is holding
            // onto the incoming scene still. So instead, we rely on the event loop run implementation
            // that must handle an Instant transition appropriately and drop this transition after
            // taking the incoming scene from it.
            if self.incoming.is_some() {
                game_context.next_scene = self.incoming.take();
            }
            return;
        }

        // Move the state machine along
        self.prev_step = self.step;
        self.step_timers[self.step] = self.step_timers[self.step].advance(ticks);
        if let ReadyState::Ready = self.step_timers[self.step] {
            // we have to finish processing this current step, but set up the next call to be the 1st call onto
            // the next state machine.
            self.step = min(self.step + 1, self.step_timers.len());
        }

        // Do any step specific logic.
        match self.transition {
            Transition::FadeToBlack { .. } => self.fade_to_black_update(ticks, game_context),
            Transition::Instant => {
                if self.incoming.is_some() {
                    game_context.next_scene = self.incoming.take();
                }
            }
            _ => {}
        }
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        if self.step >= self.step_timers.len() {
            return;
        }

        let (screen_width, screen_height) = game_context.screen_size;
        let rectangle = Rect::new(0, 0, screen_width as isize, screen_height as isize);

        match self.transition {
            Transition::FadeToBlack {
                fade_out_ticks,
                fade_in_ticks,
            } => {
                let progress = match self.step_timers[self.step] {
                    ReadyState::Cooldown {
                        wait_for,
                        ticks_waited,
                    } => {
                        let x = 1.0_f32.min(ticks_waited as f32 / wait_for as f32);
                        println!("{:?}", (x, ticks_waited, wait_for));
                        x
                    }
                    _ => 1.0,
                };
                let alpha = match self.step {
                    0 => {
                        // fade out step
                        self.outgoing.draw(game_context);
                        interpolate(0.0, 1.0, progress)
                    }
                    _ => {
                        // fade in step
                        self.incoming.as_mut().map(|o| o.draw(game_context));
                        interpolate(1.0, 0.0, progress)
                    }
                };
                let mut color = Color::black();
                color.a = alpha;
                let Some(ref mut renderer) = game_context.renderer else {
                    return;
                };
                renderer.send_command(RenderCommand::FillRect {
                    color,
                    destination: rectangle,
                });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod transition_scene_tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Default)]
    struct ObservableState {
        actions: Vec<String>,
    }

    struct ObservableScene {
        state: Rc<RefCell<ObservableState>>,
        name: String,
    }

    impl ObservableScene {
        fn new(name: &str, state: Rc<RefCell<ObservableState>>) -> Self {
            Self {
                name: name.to_string(),
                state,
            }
        }

        fn track(&mut self, action: &str) {
            self.state
                .borrow_mut()
                .actions
                .push(format!("{}: {}", self.name, action));
        }
    }

    impl Scene for ObservableScene {
        fn init(&mut self, _game_context: &mut GameContext) {
            self.track("init");
        }
        fn update(&mut self, ticks: u32, _game_context: &mut GameContext) {
            self.track(&format!("{}{}", "update ticks ", ticks));
        }
        fn draw(&mut self, _game_context: &mut GameContext) {
            self.track("draw");
        }
    }

    #[test]
    fn instant_become_ready_on_first_update() {
        let incoming_state = Rc::new(RefCell::new(ObservableState::default()));
        let outgoing_state = Rc::new(RefCell::new(ObservableState::default()));
        let incoming = ObservableScene::new("incoming", incoming_state.clone());
        let outgoing = ObservableScene::new("outgoing", outgoing_state.clone());
        let mut scene =
            TransitionScene::new(Box::new(incoming), Box::new(outgoing), Transition::Instant);
        let mut game_context = GameContext::default();

        assert_eq!(scene.step, 0);
        scene.init(&mut game_context);
        scene.update(1, &mut game_context);
        assert_eq!(scene.step, 1);
        assert!(match scene.step_timers[0] {
            ReadyState::Ready => true,
            _ => false,
        });
    }

    #[test]
    fn fade_to_black_steps_through_fadeout_fadein_instant() {
        let state = Rc::new(RefCell::new(ObservableState::default()));
        let incoming = ObservableScene::new("incoming", state.clone());
        let outgoing = ObservableScene::new("outgoing", state.clone());
        let mut scene = TransitionScene::new(
            Box::new(outgoing),
            Box::new(incoming),
            Transition::FadeToBlack {
                fade_out_ticks: 2,
                fade_in_ticks: 4,
            },
        );
        let mut game_context = GameContext::default();

        assert_eq!(scene.step, 0);
        scene.init(&mut game_context);
        for _ in 0..9 {
            scene.update(1, &mut game_context);
        }
        let expected_underlying_calls = vec![
            "outgoing: update ticks 1",
            "outgoing: update ticks 1",
            "incoming: init",
            // We only see 2 incoming because the game starts updating the incoming scene when the fade is halfway done 4/2 = 2
            "incoming: update ticks 1",
            "incoming: update ticks 1",
        ];

        assert_eq!(expected_underlying_calls, state.borrow().actions);
    }
}
