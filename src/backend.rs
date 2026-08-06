use crate::GameResult;
use crate::asset_loader::AssetLoader;
use crate::audio::Audio;
use crate::clock::Clock;
use crate::font::FontLibrary;
use crate::game::Game;
use crate::game::GameContext;
use crate::game_options::GameOptions;
use crate::renderer::Renderer;

pub trait Backend {
    fn create_event_loop(&self, game_options: &GameOptions) -> Box<dyn BackendEventLoop>;
    fn create_clock(&self) -> Box<dyn Clock>;
}

pub trait BackendEventLoop {
    fn run(&mut self, game: Game, game_context: GameContext) -> GameResult<()>;
    fn new_renderer(&self, game_options: &GameOptions) -> Box<dyn Renderer>;
    fn create_asset_loader(&self, game_options: &GameOptions) -> Box<dyn AssetLoader>;
    fn create_audio(&self, game_options: &GameOptions) -> Box<dyn Audio>;
    fn create_font_library(&self, game_options: &GameOptions) -> Box<dyn FontLibrary>;
}

pub fn init_backend(game_options: &GameOptions) -> Box<dyn Backend> {
    #[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
    {
        use crate::backend_sdl3::BackendSDL3;
        Box::new(BackendSDL3::new(game_options))
    }

    #[cfg(all(target_family = "wasm", target_os = "unknown"))]
    {
        use crate::backend_wasm::BackendWasm;
        Box::new(BackendWasm::new(game_options))
    }
}
