use crate::game::GameContext;

pub mod button;
pub mod title_screen;
pub mod transitions;

pub trait Scene {
    fn init(&mut self, game_context: &mut GameContext);
    fn update(&mut self, ticks: u32, game_context: &mut GameContext);
    fn draw(&mut self, game_context: &mut GameContext);
}
