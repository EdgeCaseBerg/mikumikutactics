use mikumikutactics::GameResult;
use mikumikutactics::game::Game;
use mikumikutactics::game_options::GameOptions;

fn main() -> GameResult<()> {
    let options = GameOptions::default();
    let game = Game::default();
    mikumikutactics::run(&options, game)
}

// Note if you looking for the entry point for wasm, it's not here, it's over in lib.rs See "start"
