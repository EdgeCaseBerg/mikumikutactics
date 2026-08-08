use crate::GameResult;
use crate::asset_loader::AssetLoader;
use crate::audio::Audio;
use crate::clock::Clock;
use crate::constants::{ProgramCounter, SceneId};
use crate::font::FontLibrary;
use crate::renderer::Color;
use crate::renderer::Renderer;
use crate::scene::Scene;

pub struct Game {
    // counters to track game updates on a fixed interval with catch up
    prev_tick: u128,
    next_tick: u128,
    pub tick_loops: u32,
    lag: u128,
    // counters to track if we should render anything when drawing (variable rendering)
    next_draw_tick: u128,
    should_draw: bool,
    pub scene: Option<Box<dyn Scene>>,
}

pub struct MouseContext {
    pub left_clicked: bool,
    pub right_clicked: bool,
    pub position: Option<(f32, f32)>,
}

impl Default for MouseContext {
    fn default() -> Self {
        Self {
            left_clicked: false,
            right_clicked: false,
            position: None,
        }
    }
}

impl MouseContext {
    pub fn update(&mut self, left: bool, right: bool, position: Option<(f32, f32)>) {
        self.left_clicked = left;
        self.right_clicked = right;
        self.position = position;
    }

    pub fn consume_left_click(&mut self) {
        self.left_clicked = false;
    }

    pub fn consume_right_click(&mut self) {
        self.right_clicked = false;
    }
}

pub struct GameContext {
    pub renderer: Option<Box<dyn Renderer>>,
    pub mouse_context: MouseContext,
    pub screen_size: (u32, u32),
    pub next_scene: Option<Box<dyn Scene>>,
    pub story_scene: Option<(SceneId, ProgramCounter)>,
    pub asset_loader: Option<Box<dyn AssetLoader>>,
    pub audio: Option<Box<dyn Audio>>,
    pub font_library: Option<Box<dyn FontLibrary>>,
    pub shutdown_flag: bool,
    pub clock: Option<Box<dyn Clock>>,
}

impl Default for GameContext {
    fn default() -> Self {
        GameContext {
            renderer: None,
            mouse_context: MouseContext::default(),
            screen_size: (1280, 720),
            next_scene: None,
            asset_loader: None,
            audio: None,
            font_library: None,
            shutdown_flag: false,
            clock: None,
            story_scene: None,
        }
    }
}

impl GameContext {
    pub fn queue_level(&mut self) {}

    pub fn shutdown(&mut self) {
        self.shutdown_flag = true;
    }

    pub fn queue_game_over(&mut self) {}
}

impl Default for Game {
    fn default() -> Game {
        Game {
            prev_tick: 0,
            next_tick: 0,
            tick_loops: 0,
            lag: 0,
            next_draw_tick: 0,
            should_draw: false,
            scene: None,
        }
    }
}

impl Game {
    pub fn reset_for_next_scene(&mut self) {
        self.tick_loops = 0;
        self.next_draw_tick = 0;
    }

    pub fn update(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        // For now let's just do 60hz, we can swap this to vsync mode later on in life.
        // https://gameprogrammingpatterns.com/game-loop.html#stuck-in-the-middle
        let ns_per_update = 1_000_000_000 / 60;
        // completely arbitrary but would control how much lag is acceptable
        let max_loops_per_update = 10;
        let Some(clock) = game_context.clock.as_ref() else {
            return Err("a clock must be initialized to update".into());
        };

        let current = clock.elapsed_since_start();
        let elapsed = current - self.prev_tick;
        self.prev_tick = current;
        self.next_tick = current + ns_per_update;
        self.lag += elapsed;
        self.tick_loops = 0;
        while self.lag >= ns_per_update && self.tick_loops < max_loops_per_update {
            self.lag -= ns_per_update;
            self.tick_loops += 1;
        }
        if self.tick_loops == max_loops_per_update {
            eprintln!("Frame skipping detected, attempting to correct by skipping frames ahead");
            self.prev_tick = clock.elapsed_since_start();
            self.next_tick = self.prev_tick + ns_per_update;
            self.tick_loops = 0;
        }

        let scene = self.scene.as_mut();
        if let Some(scene) = scene {
            scene.update(self.tick_loops, game_context)?;
        }
        Ok(())
    }

    pub fn draw(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        self.should_draw = false;
        // Since game update ticks are independent from render ticks
        // we need to compute the proper amount of lag time for what
        // to show to the user.

        // TODO: Move to shared state assuming we'd update game and render per update the same.
        let ns_per_update = 1_000_000_000 / 60;
        let clock = game_context
            .clock
            .as_ref()
            .expect("a clock must be initialized to update");

        let current = clock.elapsed_since_start();

        // If we're already past the time to draw, then align the next tick to the current time
        if current > self.next_draw_tick + ns_per_update {
            self.next_draw_tick = current;
        }

        // if we're not past the time to draw, advance by update rate until it's time to render.
        while clock.elapsed_since_start() >= self.next_draw_tick {
            self.next_draw_tick += ns_per_update;
            self.should_draw = true;
        }

        if !self.should_draw {
            clock.sleep();
            return Ok(());
        }

        // This seems very silly and annoying to have to do this twice
        {
            let renderer = game_context.renderer.as_mut();
            if let Some(renderer) = renderer {
                renderer.clear(Color::black());
            }
        }

        let scene = self.scene.as_mut();
        if let Some(scene) = scene {
            scene.draw(game_context)?;
        }

        {
            let renderer = game_context.renderer.as_mut();
            if let Some(renderer) = renderer {
                renderer.present();
            }
        }
        Ok(())
    }
}
