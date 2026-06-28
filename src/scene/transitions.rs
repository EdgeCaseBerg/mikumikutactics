use crate::scene::Scene;

pub enum Transition {
    Instant,
    FadeToBlack {
        fade_out_ticks: u32,
        fade_in_ticks: u32,
    }, // We could do a generic FadeTo if we wanted maybe for fade to white or similar.
}

pub struct SceneChange {
    pub next_scene: Box<dyn Scene>,
    pub transition: Transition,
}

pub struct TransitionScene {
    outgoing: Box<dyn Scene>,
    incoming: Box<dyn Scene>,
    transition: Transition,
}

impl TransitionScene {
    fn new(outgoing: Box<dyn Scene>, incoming: Box<dyn Scene>, transition: Transition) -> Self {
        TransitionScene {
            outgoing,
            incoming,
            transition,
        }
    }
}
