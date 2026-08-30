use crate::GameResult;
use crate::game::GameContext;

pub mod button;
pub mod story;
pub mod story_screen;
pub mod test_screen;
pub mod title_screen;
pub mod transitions;

pub trait Scene {
    fn init(&mut self, game_context: &mut GameContext) -> GameResult<()>;
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) -> GameResult<()>;
    fn draw(&mut self, game_context: &mut GameContext) -> GameResult<()>;
}
