use crate::ReadyState;
use crate::Rect;
use crate::game::GameContext;
use crate::interpolate;
use crate::renderer::{Color, RenderCommand};
use crate::scene::Scene;

pub enum Transition {
    FadeOut { within_ticks: u32 },
    FadeIn { within_ticks: u32 },
    // Dimmed { within_ticks: u32 } We could add this to be used as a pause overlay type thing
}

impl Transition {
    fn length(&self) -> u32 {
        match self {
            Transition::FadeOut { within_ticks } => *within_ticks,
            Transition::FadeIn { within_ticks } => *within_ticks,
        }
    }
}

impl From<&Transition> for ReadyState {
    fn from(t: &Transition) -> ReadyState {
        ReadyState::Cooldown {
            ticks_waited: 0,
            wait_for: t.length(),
        }
    }
}

pub struct TransitionScene {
    pub transition: Transition,
    pub timer: ReadyState,
}

impl TransitionScene {
    pub fn is_done(&self) -> bool {
        match self.timer {
            ReadyState::Ready => true,
            _ => false,
        }
    }

    pub fn new_fade_out() -> Self {
        let transition = Transition::FadeOut { within_ticks: 15 };
        Self {
            timer: (&transition).into(),
            transition,
        }
    }
    pub fn new_fade_in() -> Self {
        let transition = Transition::FadeIn { within_ticks: 15 };
        Self {
            timer: (&transition).into(),
            transition,
        }
    }
}

impl Scene for TransitionScene {
    fn init(&mut self, _game_context: &mut GameContext) {
        self.timer = (&self.transition).into();
    }
    fn update(&mut self, ticks: u32, _game_context: &mut GameContext) {
        self.timer = self.timer.advance(ticks);
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        let (screen_width, screen_height) = game_context.screen_size;
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };
        let rectangle = Rect::new(0, 0, screen_width as isize, screen_height as isize);

        // -1.0 is to enable the first draw to be completely opaque, otherwise its 0.93 or so
        let progress = match self.timer {
            ReadyState::Cooldown {
                wait_for,
                ticks_waited,
            } => 1.0_f32.min((ticks_waited as f32 - 1.0) / wait_for as f32),
            _ => 1.0,
        };

        let alpha = match &self.transition {
            Transition::FadeOut { .. } => interpolate(0.0, 1.0, progress),
            Transition::FadeIn { .. } => interpolate(1.0, 0.0, progress),
        };

        let mut color = Color::black();
        color.a = alpha;
        renderer.send_command(RenderCommand::FillRect {
            color,
            destination: rectangle,
        });
    }
}

#[cfg(test)]
mod transition_scene_tests {
    use super::*;
    use crate::renderer::Renderer;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct FakeRenderer {
        actions: Rc<RefCell<Vec<RenderCommand>>>,
    }
    impl Renderer for FakeRenderer {
        fn name(&self) -> String {
            "Fake!".to_string()
        }
        fn send_command(&mut self, cmd: RenderCommand) {
            eprintln!("COMMAND ADDED {:?}", cmd);
            self.actions.as_ref().borrow_mut().push(cmd);
        }
        fn clear(&mut self, _color: Color) {}
        fn present(&mut self) {}
    }

    #[test]
    fn fade_in_fades_to_0_alpha_once_done() {
        let mut scene = TransitionScene::new_fade_in();
        let actions = Rc::new(RefCell::new(vec![]));
        let mut game_context = GameContext::default();
        game_context.renderer = Some(Box::new(FakeRenderer {
            actions: actions.clone(),
        }));

        scene.init(&mut game_context);
        for _ in 0..scene.transition.length() + 1 {
            scene.update(1, &mut game_context);
            scene.draw(&mut game_context);
        }

        let actions = actions.borrow();
        let first_command = actions.first();
        if let RenderCommand::FillRect { color, .. } = first_command.unwrap() {
            assert_eq!(1.0, color.a);
        } else {
            assert!(false, "first command was not a fill rect with 1.0 alpha");
        }

        let last_command = actions.last();
        if let RenderCommand::FillRect { color, .. } = last_command.unwrap() {
            assert_eq!(0.0, color.a);
        } else {
            assert!(false, "last command was not a fill rect");
        }
    }

    #[test]
    fn fade_out_fades_to_opaque_once_done() {
        let mut scene = TransitionScene::new_fade_out();
        let actions = Rc::new(RefCell::new(vec![]));
        let mut game_context = GameContext::default();
        game_context.renderer = Some(Box::new(FakeRenderer {
            actions: actions.clone(),
        }));

        scene.init(&mut game_context);
        for _ in 0..scene.transition.length() + 1 {
            scene.update(1, &mut game_context);
            scene.draw(&mut game_context);
        }

        let actions = actions.borrow();
        let first_command = actions.first();
        if let RenderCommand::FillRect { color, .. } = first_command.unwrap() {
            assert_eq!(0.0, color.a);
        } else {
            assert!(false, "first command was not a fill rect with 1.0 alpha");
        }

        let last_command = actions.last();
        if let RenderCommand::FillRect { color, .. } = last_command.unwrap() {
            assert_eq!(1.0, color.a);
        } else {
            assert!(false, "last command was not a fill rect");
        }
    }
}
