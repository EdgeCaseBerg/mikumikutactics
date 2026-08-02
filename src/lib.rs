pub mod asset_loader;
pub mod audio;
pub mod backend;
pub mod clock;

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
pub mod backend_sdl3;

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub mod backend_wasm;

#[cfg(test)]
pub mod backend_dummy;

pub mod constants;
pub mod font;
pub mod game;
pub mod game_options;
pub mod grid_layout;
pub mod manifest;
pub mod renderer;
pub mod scene;

use std::ops::{Add, Div, Sub};

use crate::backend::init_backend;
use crate::game::Game;
use crate::game_options::GameOptions;
use crate::grid_layout::GridLayout;
use crate::scene::Scene;
use crate::scene::title_screen::TitleScene;

#[cfg(not(all(target_family = "wasm", target_os = "unknown")))]
extern crate sdl3;

#[derive(Debug, Clone, Copy)]
pub enum ReadyState {
    Ready,
    Cooldown { wait_for: u32, ticks_waited: u32 },
}

impl ReadyState {
    fn advance(self, ticks: u32) -> ReadyState {
        advance_ready_state(self, ticks)
    }
}

fn advance_ready_state(ready_state: ReadyState, ticks: u32) -> ReadyState {
    match ready_state {
        ReadyState::Ready => ready_state,
        ReadyState::Cooldown {
            wait_for,
            ticks_waited,
        } => {
            let ticks_waited = ticks_waited.saturating_add(ticks);
            if ticks_waited >= wait_for {
                ReadyState::Ready
            } else {
                ReadyState::Cooldown {
                    wait_for,
                    ticks_waited,
                }
            }
        }
    }
}

// wikipedia.
fn interpolate(z0: f32, z1: f32, alpha: f32) -> f32 {
    z0 - z0 * alpha + z1 * alpha
}

#[derive(Debug, Clone)]
pub struct SpriteInfo {
    start_x: u32,
    start_y: u32,
    width: u32,
    height: u32,
    frames: u32,
    current_frame: u32,
    framerate_per_second: u32,
    delta: u32,
}

impl SpriteInfo {
    fn advance(&mut self, delta: u32) {
        self.delta = self.delta.wrapping_add(delta);
        if self.delta > self.framerate_per_second {
            self.current_frame += 1;
            self.delta = 0;
        }
        // Always loop for now.
        if self.current_frame >= self.frames {
            self.current_frame = 0;
        }
    }

    fn get_rect_for(&self) -> [u32; 4] {
        let x_offset = self.start_x + self.width * self.current_frame;
        [x_offset, self.start_y, self.width, self.height]
    }

    fn get_rect(&self) -> Rect {
        let [x, y, w, h] = self.get_rect_for();
        Rect::new(x as isize, y as isize, w as isize, h as isize)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rect<T: PartialOrd + Copy + Add<Output = T> + Sub<Output = T> + Div<Output = T> = isize>
{
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl Rect {
    pub fn relative_rect_to_layout(&self, layout: &GridLayout) -> Rect {
        let top_left = layout.cell_rect(self.y as usize, self.x as usize);
        let (cw, ch) = layout.cell_size();
        Rect {
            x: top_left.x,
            y: top_left.y,
            width: cw * self.width,
            height: ch * self.height,
        }
    }
}

impl<T: PartialOrd + Copy + Add<Output = T> + Sub<Output = T> + Div<Output = T>> Rect<T> {
    // (x, y) are the corner from which width and height expand from (x + width = x2, similar for y)
    // so its the bottom left corner in a positive coordinate space and the top left in a quadrant 4 space (like most images)
    #[inline(always)]
    pub fn new(x: T, y: T, width: T, height: T) -> Rect<T> {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, x2: T, y2: T) -> bool {
        let in_x = self.x < x2 && x2 < self.x + self.width;
        let in_y = self.y < y2 && y2 < self.y + self.height;
        in_x && in_y
    }

    pub fn center(&self) -> (T, T) {
        // clippy is stupid and thinks this is illegal and bad.
        // clippy is a FOOL who doesn't appreciate my GENIUS at
        // understanding that any number divided by itself is 1
        // and such cleverness sparks immense JOY at using this
        // to create the ability to say 1 + 1 = 2 generically.
        #[allow(clippy::eq_op)]
        let one = self.x / self.x;
        let two = one + one;
        let cx = self.x + self.width / two;
        let cy = self.y + self.height / two;
        (cx, cy)
    }
}

pub fn run(game_options: &GameOptions, mut game: Game) {
    let backend = init_backend(game_options);
    let mut event_loop = backend.create_event_loop(game_options);
    let mut game_context = crate::game::GameContext::default();
    let renderer = event_loop.new_renderer(game_options);
    let asset_loader = event_loop.create_asset_loader(game_options);
    let font_library = event_loop.create_font_library(game_options);
    let audio = event_loop.create_audio(game_options);
    let clock = backend.create_clock();
    game_context.font_library = Some(font_library);
    game_context.screen_size = (game_options.window_width, game_options.window_height);
    game_context.renderer = Some(renderer);
    game_context.asset_loader = Some(asset_loader);
    game_context.audio = Some(audio);
    game_context.clock = Some(clock);
    game.scene = Some(Box::new(TitleScene::default()));
    event_loop.run(game, game_context);
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let options = GameOptions::default();
    let game = Game::default();
    run(&options, game);
}
