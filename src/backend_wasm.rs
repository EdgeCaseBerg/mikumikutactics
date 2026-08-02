use crate::asset_loader::{AssetCatalog, AssetLoader};
use crate::audio::{Audio, AudioResult};
use crate::backend::*;
use crate::clock::Clock;
use crate::constants::*;
use crate::constants::{MusicId, SfxId};
use crate::font::{BitmapFontLibrary, FontLibrary};
use crate::game::Game;
use crate::game::GameContext;
use crate::game_options::GameOptions;
use crate::renderer::{Color, RenderCommand, Renderer};

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;

use std::time::Duration; // this is probably going to bite us later.

use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{
    CanvasRenderingContext2d, Document, HtmlAudioElement, HtmlCanvasElement, HtmlDivElement,
    HtmlElement, HtmlImageElement, MouseEvent, Window,
};

pub struct WasmSounds {
    sound_by_id: HashMap<SfxId, HtmlAudioElement>,
    music_by_id: HashMap<MusicId, HtmlAudioElement>,
    base_path: PathBuf,
    storage: HtmlDivElement,
}

impl WasmSounds {
    fn new(game_options: &GameOptions) -> Self {
        let path = game_options.assets_path.clone();
        let storage = document()
            .create_element("div")
            .expect("could not create div")
            .dyn_into::<HtmlDivElement>()
            .expect("could not dyn_into HtmlDivElement");
        storage
            .style()
            .set_property("display", "none")
            .expect("couldnt hide div to load audio into");
        body()
            .append_child(&storage)
            .expect("could not add audio storage to body");

        Self {
            base_path: path,
            sound_by_id: HashMap::new(),
            music_by_id: HashMap::new(),
            storage,
        }
    }
}

impl Audio for WasmSounds {
    fn play_sfx(&mut self, sound_id: SfxId) -> AudioResult<()> {
        let Some(audio) = self.sound_by_id.get(&sound_id) else {
            web_sys::console::log_1(&format!("sound id {} not loaded", sound_id.0).into());
            return Ok(());
        };
        // audio isn't particularly critical if it doesn't work, so swallow errors
        // an obvious error will occur in the console if the player hasn't clicked yet
        // in the canvas which allows the audio to play in the first place.
        let _ = audio.play();
        Ok(())
    }
    fn load_sfx(&mut self, sound_id: SfxId) -> AudioResult<()> {
        if let Some(_) = self.sound_by_id.get(&sound_id) {
            web_sys::console::log_1(&format!("sound id {} already loaded", sound_id.0).into());
            return Ok(());
        }

        web_sys::console::log_1(&format!("loading sound id {}", sound_id.0).into());
        let document = document();
        let audio = document
            .create_element("audio")
            .expect("could not create audio")
            .dyn_into::<HtmlAudioElement>()
            .expect("could not dyn_into HtmlAudioElement");

        let path = sfx_id_to_relative_path(sound_id);
        let path = self.base_path.join(path);
        audio.set_src(&pathbuf_to_url(&path));

        let result = self.storage.append_child(&audio);
        if let Ok(_) = result {
            self.sound_by_id.insert(sound_id, audio);
            web_sys::console::log_1(
                &format!("texture loaded for texture id {}", sound_id.0).into(),
            );
        } else if let Err(e) = result {
            web_sys::console::log_1(
                &format!(
                    "error loading texture id {} {:?}",
                    sound_id.0,
                    e.as_string()
                )
                .into(),
            );
        }
        Ok(())
    }
    fn play_music(&mut self, id: MusicId) -> Result<(), Box<dyn std::error::Error>> {
        let Some(audio) = self.music_by_id.get(&id) else {
            web_sys::console::log_1(&format!("sound id {} not loaded", id.0).into());
            return Ok(());
        };
        // We need to pause anything that might be playing.
        for audio in self.music_by_id.values() {
            let _ = audio.pause();
            audio.set_current_time(0.0);
        }

        // audio isn't particularly critical if it doesn't work, so swallow errors
        // an obvious error will occur in the console if the player hasn't clicked yet
        // in the canvas which allows the audio to play in the first place.
        let _ = audio.play();
        Ok(())
    }

    /// Calling this method with the same id multiple times will only load the music once.
    fn load_music(&mut self, id: MusicId) -> AudioResult<()> {
        if let Some(_) = self.music_by_id.get(&id) {
            web_sys::console::log_1(&format!("sound id {} already loaded", id.0).into());
            return Ok(());
        }

        web_sys::console::log_1(&format!("loading sound id {}", id.0).into());
        let document = document();
        let audio = document
            .create_element("audio")
            .expect("could not create audio")
            .dyn_into::<HtmlAudioElement>()
            .expect("could not dyn_into HtmlAudioElement");

        // TODO: use asset catalog to load?
        let path = music_id_to_relative_path(id);
        let path = self.base_path.join(path);
        audio.set_src(&pathbuf_to_url(&path));

        let result = self.storage.append_child(&audio);
        if let Ok(_) = result {
            self.music_by_id.insert(id, audio);
            web_sys::console::log_1(&format!("texture loaded for texture id {}", id.0).into());
        } else if let Err(e) = result {
            web_sys::console::log_1(
                &format!("error loading texture id {} {:?}", id.0, e.as_string()).into(),
            );
        }
        Ok(())
    }
    fn load_bg_music(&mut self) -> Vec<AudioResult<MusicId>> {
        let mut ids = Vec::new();
        let music_folder = PathBuf::new().join("audio").join("cc-vocaloid");
        let document = document();
        // These are unfortunately hardcoded in the wasm world, it is what it is.
        for id in [MusicId(1), MusicId(2)] {
            web_sys::console::log_1(&format!("loading sound id {}", id.0).into());
            let audio = document
                .create_element("audio")
                .expect("could not create audio")
                .dyn_into::<HtmlAudioElement>()
                .expect("could not dyn_into HtmlAudioElement");

            let path = music_folder.clone().join(format!("{}.mp3", id.0));
            let src = self.base_path.join(path);
            let path = pathbuf_to_url(&src);
            audio.set_src(&pathbuf_to_url(&PathBuf::from(path)));

            let result = self.storage.append_child(&audio);
            if let Ok(_) = result {
                self.music_by_id.insert(id, audio);
                web_sys::console::log_1(&format!("texture loaded for texture id {}", id.0).into());
                ids.push(Ok(id));
            } else if let Err(e) = result {
                web_sys::console::log_1(
                    &format!("error loading texture id {} {:?}", id.0, e.as_string()).into(),
                );
                ids.push(Err(format!(
                    "error loading texture id {} {:?}",
                    id.0,
                    e.as_string()
                )
                .into()));
            }
        }
        ids
    }

    fn music_duration_seconds(&self, id: MusicId) -> AudioResult<Duration> {
        let Some(audio) = self.music_by_id.get(&id) else {
            return Ok(Duration::from_millis(0));
        };
        Ok(Duration::from_secs(audio.duration() as u64))
    }

    fn prepare(&mut self) -> Vec<AudioResult<()>> {
        vec![]
    }
}

struct WasmContext {
    storage: Rc<HtmlDivElement>,
    context: Rc<CanvasRenderingContext2d>,
    canvas: Rc<HtmlCanvasElement>,
    document: Rc<Document>,
    texture_id_to_image: HashMap<TextureId, HtmlImageElement>,
}

impl WasmContext {
    fn get_image(&self, id: TextureId) -> Option<&HtmlImageElement> {
        let Some(html_image_element) = self.texture_id_to_image.get(&id) else {
            return None;
        };
        if !html_image_element.complete() {
            return None;
        }
        Some(html_image_element)
    }
}

struct AssetLoaderWasm {
    base_path: PathBuf,
    wasm_context: Rc<RefCell<WasmContext>>,
    catalog: Rc<AssetCatalog>,
}

impl AssetLoaderWasm {
    fn new(game_options: &GameOptions, wasm_context: Rc<RefCell<WasmContext>>) -> Self {
        let path = game_options.assets_path.clone();
        let manifest_str = include_str!("../assets/manifest");
        let catalog = AssetCatalog::from_manifest_str(&manifest_str)
            .expect("cannot create asset loader, manifest was not included in application");

        Self {
            base_path: path,
            wasm_context: wasm_context.clone(),
            catalog: catalog.into(),
        }
    }
}

fn pathbuf_to_url(p: &PathBuf) -> String {
    let mut s = String::new();
    let len = p.iter().count();
    for (index, path_part) in p.iter().enumerate() {
        s.push_str(&path_part.to_string_lossy());
        if index < len - 1 {
            s.push_str("/");
        }
    }
    s
}

impl AssetLoader for AssetLoaderWasm {
    fn ensure_texture_spritesheet_loaded(&mut self, id: TextureId) {
        let context = &mut *self.wasm_context.borrow_mut();

        if let Some(_) = context.texture_id_to_image.get(&id) {
            web_sys::console::log_1(&format!("texture id {} already loaded", id.0).into());
            return;
        }
        web_sys::console::log_1(&format!("loading texture id {}", id.0).into());
        let img = context
            .document
            .create_element("img")
            .expect("could not create img")
            .dyn_into::<HtmlImageElement>()
            .expect("could not dyn_into HtmlImageElement");
        img.set_name(&format!("texture-{}", id.0));
        let path = self
            .catalog
            .get_asset_path(id)
            .unwrap_or(id_to_relative_path(id));
        let path = self.base_path.join(path);
        img.set_src(&pathbuf_to_url(&path));
        let result = context.storage.append_child(&img);
        if let Ok(_) = result {
            context.texture_id_to_image.insert(id, img);
            web_sys::console::log_1(&format!("texture loaded for texture id {}", id.0).into());
        } else if let Err(e) = result {
            web_sys::console::log_1(
                &format!("error loading texture id {} {:?}", id.0, e.as_string()).into(),
            );
        }
    }

    fn get_catalog(&self) -> Rc<AssetCatalog> {
        self.catalog.clone()
    }
}

struct WasmClock {
    start: u128,
}

fn milli_to_nano(milliseconds: f64) -> u128 {
    (milliseconds * 1_000_000.0) as u128
}

impl WasmClock {
    fn new() -> Self {
        let window = web_sys::window().expect("no browser window found");
        let milliseconds = window
            .performance()
            .expect("no performance in browser defined")
            .now();
        let nanos = milli_to_nano(milliseconds);
        WasmClock { start: nanos }
    }
}

impl Clock for WasmClock {
    fn elapsed_since_start(&self) -> u128 {
        let window = web_sys::window().expect("no browser window found");
        let now = window
            .performance()
            .expect("no performance in browser defined")
            .now();
        let now = milli_to_nano(now);
        let nanos = now - self.start;
        nanos
    }
    fn sleep(&self) {
        // do nothing. the request animation frame loop sleeps for us.
    }
}

/////////////////////////////////////////////////////////////////////
// Helpers shameless lifted out of wasm-bingen.github.io
//
//
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣿⣿⣷⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⣴⣿⣷⣶⣦⣤⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⢟⣯⣭⣝⠻⣿⣿⣷⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⢀⡿⠟⠿⢿⣿⣿⣿⣿⣷⣄⠀⠀⠀⠀⠀⣰⣿⣿⢣⣿⠟⠀⠈⢻⡘⣿⣿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⢸⡇⣟⠛⠓⠮⣝⣻⣿⡿⠟⠛⠛⠛⠛⠛⣿⣿⣏⣿⣟⣀⠀⠀⠀⣷⢹⣿⣿⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⢸⣇⢿⡀⠀⣠⠞⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠉⠛⠶⢦⣽⣸⣿⣿⣷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠸⣿⣜⣷⠞⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠛⣿⣿⣿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⢧⡀⠀⠀⠀⠹⣿⣿⣿⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⣾⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢳⡄⠀⠀⠀⢻⣿⣿⣿⣷⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⢰⡇⠀⠀⠀⠀⢰⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢻⡄⠀⠀⢸⣿⣿⡿⣫⣿⣧⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⢸⠁⠀⠀⠀⠀⢸⠋⢧⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⡄⠀⠸⣿⣿⠡⣿⣿⣿⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀It's⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⢸⡄⠀⠀⢀⠀⡾⠀⠀⠑⢦⡀⠀⠀⠀⠀⠀⠀⣀⡤⠀⠀⠀⠀⢀⡀⠀⠀⠙⣆⠀⠹⣿⣷⣝⠿⣿⣿⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀mine⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠘⣇⠀⠀⢸⣿⠃⣀⡀⠀⠀⠙⢆⠀⠀⠀⠀⠚⣩⣀⠀⠀⠀⢀⣘⣿⠶⣄⣠⡽⠄⠀⠹⣿⣿⣷⣶⠋⠻⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀now!⠀⠀⠀⠀⠀
//⠀⠀⠀⠘⡀⠀⠈⡟⠀⠀⠙⠦⠀⠀⠈⠛⢦⣀⣀⣠⣽⠍⠳⠒⠛⣭⣵⠆⠀⡇⠀⠀⠀⠀⠀⣿⣿⡿⠃⠀⠀⠸⡆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⣇⠐⠦⣷⣤⣤⣀⡀⠀⠀⠀⠀⠀⠁⠀⠀⢀⣀⣀⣴⣿⠟⠁⠀⠀⡇⠀⠀⠀⠀⠀⢸⠋⠀⠀⠀⠀⣠⠟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⣹⠤⠶⠋⠉⠙⠛⠛⠷⠄⠀⠀⠀⠀⠀⠀⠀⠘⠛⠉⠀⠀⠀⠀⠀⠧⣴⣶⣶⣤⣤⣿⡆⣀⢾⠥⠞⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⢧⡀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⡦⣄⣀⣰⠂⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⠟⠻⢯⡿⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠻⣿⣳⣦⣄⣀⠀⠀⠀⠀⠀⠘⣧⡤⠶⠿⠦⣤⣤⠴⠖⠛⠦⣤⠴⣾⠁⠀⠀⠀⠀⣆⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠙⠻⡏⠀⠈⠉⢻⡗⢲⠒⡶⠁⠀⠀⠀⠀⠀⢿⠳⣦⠀⠀⠈⢳⣿⣦⣤⣀⣠⣤⠼⢿⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠲⠧⣤⣤⣤⣾⡯⢭⣻⠁⠀⠀⠀⠀⠀⠀⣼⠀⢸⠀⠀⠀⠀⢻⡋⠀⣠⣤⠴⡶⠒⠻⢦⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⢰⠋⢸⣿⣁⠀⠀⠀⠳⣤⣀⣀⣠⡴⠋⠀⠀⢸⠀⠀⠀⢀⣠⣿⣀⡠⠴⢎⠀⠙⠢⣼⡁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⢸⡀⠘⣇⣘⣿⠋⠋⠛⣳⠦⠤⣉⣛⠶⢶⣶⣛⣤⣶⣾⡿⣿⣞⠃⠀⠀⠀⠙⠒⠤⣀⣉⣙⣶⣦⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠳⢤⣤⣋⡁⠀⠀⣰⠃⠀⠀⠀⣀⣼⣿⣿⡿⠛⠋⠉⠙⠻⣿⣿⣶⣤⣄⠀⠀⠀⠀⣹⣿⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠰⣿⣿⣿⣿⣿⣶⣧⣄⣀⣤⣾⣿⣿⣿⠟⠁⣀⣀⣠⣤⣶⣾⣿⣿⣿⣿⣿⣷⣶⣾⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠛⠛⠻⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⢿⣻⣟⠛⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠹⣿⣿⣿⣿⣭⣭⣽⠛⠛⠛⠛⠛⠛⠉⠙⠯⣭⣶⣾⣿⣿⣿⣦⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢹⣿⣻⣿⣿⣿⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⢿⣿⣿⣿⣿⣷⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⠿⠿⠿⠛⠛⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠛⠛⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
////////////////////////////////////////////////////////////////////////
fn window() -> Window {
    web_sys::window().expect("no global `window` exists")
}

fn document() -> Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn body() -> HtmlElement {
    document().body().expect("document should have a body")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
/////////////////////////////////////////////////////////////////////
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣀⣀⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣴⣿⣿⣿⣿⣿⣿⣷⣶⣤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣀⣀⣀⣀⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣾⣿⣿⣿⢛⣛⡛⠿⣿⣿⣿⣿⣷⣤⣠⣤⣀⣀⠀⣠⣤⣴⣾⣿⣿⣿⣿⣿⣿⣿⣷⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⡇⣿⡿⠿⠿⣶⣭⣛⠿⣿⣿⠗⠀⠈⠉⠙⠻⢿⣿⣿⣿⣿⣿⣿⣿⣿⡿⢿⣷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢠⣿⣿⣿⣧⢻⡇⠀⠀⠈⢙⣿⠟⠋⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠿⣿⣿⡿⢛⠱⡞⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣰⠏⢻⣿⣿⣿⡌⣧⠀⢀⡶⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⣦⡁⢸⢣⣿⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠃⠀⢈⣿⣿⣿⣿⣜⣷⠟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠻⣇⣾⡟⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⡆⠀⢠⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠳⣄⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣼⠃⠀⣾⣿⣿⣿⡿⠁⠀⠀⢀⠏⠀⠀⢀⣤⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠘⠇⢻⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢰⠇⠀⢰⣿⣿⣿⣿⠃⠀⠀⠀⡾⠀⠀⠀⣼⠉⢷⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⡄⢷⠀⠀⠀⠀⠀⠀⠀⠀⠀Now⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⠀⣰⣿⣿⣿⣿⡏⠀⠀⠀⢸⠃⠀⠀⣸⠃⠀⠈⢧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢷⠈⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀on⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣽⣾⣯⣍⠻⣿⡟⠀⠀⠀⠀⣿⠀⣠⠞⠁⠀⠀⠀⠈⠳⣄⠀⠀⠀⠀⠀⠀⢰⡆⠀⠀⠀⠀⢸⡇⢹⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀to
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⢸⣿⣿⣼⠁⠀⠀⠀⠀⣿⣠⠃⠀⠀⠀⢠⡀⠀⠀⠘⢦⠀⢀⣀⠀⠠⠞⠀⣶⣄⣤⡔⠈⠀⢸⡇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀ my
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡴⠻⣿⣼⣿⣿⣧⠀⠀⠀⠀⠀⢹⡄⠀⠀⠀⠀⠀⠙⠦⣄⠀⠀⠉⠛⠉⠓⠒⠛⠛⠉⣀⣀⢠⡀⠀⢸⠃⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀ code!
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⣅⠀⠘⢷⣝⣛⣻⡄⠀⠀⠀⠀⠀⢳⠛⠿⢷⣶⣤⣤⡀⠀⠀⠀⠀⠀⠀⠀⣠⣴⣾⡿⠿⢯⡄⠉⠳⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠳⠄⣘⣻⡿⠿⣿⣶⣶⣤⣤⣴⣯⡀⠀⠀⠀⠀⠙⠃⢠⣀⣠⣀⠀⢀⡀⠉⠉⠀⠀⠀⠀⠀⠀⠀⣹⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠓⠟⣩⣿⣿⣿⢿⣿⣧⣄⣀⣀⠀⢀⡴⠛⠉⠉⠙⠻⣏⠀⠀⠀⠀⠀⠀⠀⣀⡤⠾⢧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⡼⠋⠈⡿⠁⠀⠈⠻⣄⡴⣋⣻⡟⠀⠀⠀⠀⠀⠀⠈⡷⣶⠶⢶⡾⣿⣉⡇⠀⠀⠘⣇⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⢳⣦⣄⣿⠀⠀⠀⢠⡟⠋⠉⣼⢻⡄⠀⠀⠀⠀⢀⡼⠟⠉⠀⠈⡇⠀⠈⠱⣄⣀⣀⣘⣦⠄⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣤⣶⡶⠞⠋⠀⠰⠿⢤⡶⠖⣿⠀⠀⠀⢻⡀⠙⠓⠶⠶⣶⣿⡧⠴⠶⣾⠛⠁⠀⠀⠀⢨⣿⡉⠉⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢸⣿⣿⣷⠀⠀⠀⠀⠀⣠⠊⠙⢷⠀⠀⠀⠀⠘⢷⣄⣀⣠⣿⣿⡄⢀⣼⢻⠀⠀⠀⠀⢰⠃⡟⠈⣧⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⣿⣿⣿⣧⡀⢀⣤⠊⠁⠀⠀⠘⡆⠀⠀⠀⠀⣀⣩⣽⡿⢹⣿⡟⠁⠘⡌⢧⠀⠀⣀⣇⣴⠇⢀⡿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠸⣿⣿⣿⣿⣿⣅⠀⠀⠀⠀⣠⣿⣶⡶⠚⠋⠉⠈⠉⠀⢸⣿⣷⠀⠀⠹⡌⣻⡛⠻⣅⣠⡤⠞⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠻⣿⣿⣿⣿⣿⣶⣤⣤⣿⣿⣿⣷⣶⣤⣤⣤⣤⣤⣾⣿⣿⣤⡀⢀⣿⣿⣿⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⢻⣿⣿⣿⣿⡿⠿⢿⣿⣿⣿⣿⣿⣿⣿⠿⠿⣻⣿⣿⣿⣿⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣿⣿⣿⣿⣷⣤⣴⠏⠀⠀⠀⠀⠀⠀⠀⢳⣿⣿⣿⣿⣿⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣴⣿⣿⣿⣿⣿⠟⠁⠀⠀⠀⠀⠀⠀⠀⠀⠘⣿⣿⣿⣿⣿⡄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
//⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠻⠿⠿⠿⠟⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠛⠛⠛⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀ ⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀
////////////////////////////////////////////////////////////////////////

pub struct BackendWasm {
    canvas: Rc<HtmlCanvasElement>,
}

impl BackendWasm {
    pub fn new(game_options: &GameOptions) -> Self {
        let document = document();
        let canvas = document
            .create_element("canvas")
            .expect("could not create canvas")
            .dyn_into::<HtmlCanvasElement>()
            .expect("could not dyn_into HtmlCanvasElement");
        body()
            .append_child(&canvas)
            .expect("could not add canvas to body");
        canvas.set_width(game_options.window_width);
        canvas.set_height(game_options.window_height);
        canvas
            .style()
            .set_property("border", "solid")
            .expect("cant style canvas");
        canvas
            .style()
            .set_property("background", "black")
            .expect("cant style canvas");
        let canvas = Rc::new(canvas);
        BackendWasm { canvas }
    }
}

impl Backend for BackendWasm {
    fn create_clock(&self) -> Box<dyn Clock> {
        Box::new(WasmClock::new())
    }
    fn create_event_loop(&self, _game_options: &GameOptions) -> Box<dyn BackendEventLoop> {
        let document = document();
        let div = document
            .create_element("div")
            .expect("could not create div")
            .dyn_into::<HtmlDivElement>()
            .expect("could not dyn_into HtmlDivElement");
        div.style()
            .set_property("display", "none")
            .expect("couldnt hide div to load images into");
        body()
            .append_child(&div)
            .expect("could not add canvas to body");
        let context = (*self.canvas)
            .get_context("2d")
            .expect("could not find 2d context")
            .expect("could not get context, was none")
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .expect("could not dyn_into element into CanvasRenderingContext2d");

        let wasm_context = Rc::new(RefCell::new(WasmContext {
            context: context.into(),
            canvas: self.canvas.clone(),
            document: document.into(),
            storage: div.into(),
            texture_id_to_image: HashMap::new(),
        }));
        let e = EventLoopWasm {
            canvas: self.canvas.clone(),
            wasm_context: wasm_context,
        };
        Box::new(e)
    }
}

struct MouseInfo {
    left_pressed: Option<bool>,
    right_pressed: Option<bool>,
    mouse_coordinates: (f32, f32),
}

impl MouseInfo {
    fn from(event: &MouseEvent, canvas: &HtmlCanvasElement) -> Self {
        let (left_pressed, right_pressed) = match event.button() {
            0 => (Some(true), None),
            2 => (None, Some(true)),
            _ => (None, None),
        };
        let rect = canvas.get_bounding_client_rect();
        let x = (event.client_x() as f64 - rect.left()) * (canvas.width() as f64 / rect.width());
        let y = (event.client_y() as f64 - rect.top()) * (canvas.height() as f64 / rect.height());
        Self {
            left_pressed,
            right_pressed,
            mouse_coordinates: (x as f32, y as f32),
        }
    }
}

pub struct EventLoopWasm {
    canvas: Rc<HtmlCanvasElement>,
    wasm_context: Rc<RefCell<WasmContext>>,
}

impl BackendEventLoop for EventLoopWasm {
    fn run(&mut self, mut game: Game, mut game_context: GameContext) {
        web_sys::console::log_1(&"starting game loop".into());

        // First time load setup
        let scene = game.scene.as_mut();
        if let Some(scene) = scene {
            scene.init(&mut game_context);
        }

        // setup input processing
        // initialize the audio pool if the scene has queued things up
        let audio = game_context.audio.as_mut();
        if let Some(audio) = audio {
            let _ = audio.prepare();
        }

        let resize_time = Rc::new(RefCell::new(false));
        {
            let resize_time = resize_time.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |_event: web_sys::Event| {
                *resize_time.clone().borrow_mut() = true.into();
            });
            window().set_onresize(Some(closure.as_ref().unchecked_ref()));
            closure.forget()
        }

        let mouse = MouseInfo {
            left_pressed: None,
            right_pressed: None,
            mouse_coordinates: (0., 0.),
        };
        let mouse = Rc::new(RefCell::new(mouse));
        {
            let mouse = mouse.clone();
            let canvas = self.canvas.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let mouse_info = MouseInfo::from(&event, &canvas);
                *mouse.borrow_mut() = mouse_info;
            });
            self.canvas
                .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
        {
            let mouse = mouse.clone();
            let canvas = self.canvas.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let mouse_info = MouseInfo::from(&event, &canvas);
                mouse.borrow_mut().mouse_coordinates = mouse_info.mouse_coordinates;
            });
            self.canvas
                .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
        {
            let mouse = mouse.clone();
            let canvas = self.canvas.clone();
            let closure = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MouseEvent| {
                let mut mouse_info = MouseInfo::from(&event, &canvas);
                // invert because we're unclicking
                mouse_info.left_pressed = mouse_info.left_pressed.map(|_| false);
                mouse_info.right_pressed = mouse_info.right_pressed.map(|_| false);
                *mouse.borrow_mut() = mouse_info;
            });
            self.canvas
                .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }

        let self_referencing_function: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
            Rc::new(RefCell::new(None));
        let srf_handle = self_referencing_function.clone();
        let canvas = self.canvas.clone();
        let closure =
            Closure::wrap(Box::new(move || {
                // Any events?
                game_context.mouse_context.update(
                    mouse.borrow().left_pressed.unwrap_or(false),
                    mouse.borrow().right_pressed.unwrap_or(false),
                    Some(mouse.borrow().mouse_coordinates),
                );
                if *resize_time.clone().borrow() {
                    *resize_time.borrow_mut() = false;
                    game_context.screen_size = (canvas.width() as u32, canvas.height() as u32);
                }

                // Run updates.
                game.update(&mut game_context);
                if let Some(mut next_scene) = game_context.next_scene.take() {
                    next_scene.init(&mut game_context);
                    game.scene = Some(next_scene);
                    game.reset_for_next_scene();
                    let audio = game_context.audio.as_mut();
                    if let Some(audio) = audio {
                        audio.prepare();
                    }
                }
                game.draw(&mut game_context);
                if game_context.shutdown_flag {
                    return;
                }

                request_animation_frame(srf_handle.borrow().as_ref().expect(
                    "closure dropped before expected self referenced callback expected it",
                ));
            }) as Box<dyn FnMut()>);
        *self_referencing_function.borrow_mut() = Some(closure);

        // LOOOOOOOOOP
        request_animation_frame(
            self_referencing_function
                .borrow()
                .as_ref()
                .expect("code drift! closure just made is suddenly gone!"),
        );

        web_sys::console::log_1(&"goodbye wasm".into());
    }

    fn new_renderer(&self, _game_options: &GameOptions) -> Box<dyn Renderer> {
        let r = RendererWasm {
            commands: vec![],
            wasm_context: self.wasm_context.clone(),
        };
        Box::new(r)
    }

    fn create_asset_loader(&self, game_options: &GameOptions) -> Box<dyn AssetLoader> {
        let a = AssetLoaderWasm::new(game_options, self.wasm_context.clone());
        Box::new(a)
    }

    fn create_audio(&self, game_options: &GameOptions) -> Box<dyn Audio> {
        let s = WasmSounds::new(game_options);
        Box::new(s)
    }

    fn create_font_library(&self, _game_options: &GameOptions) -> Box<dyn FontLibrary> {
        Box::new(BitmapFontLibrary::default())
    }
}

struct RendererWasm {
    wasm_context: Rc<RefCell<WasmContext>>,
    commands: Vec<RenderCommand>,
}

impl RendererWasm {
    fn process_commands(&mut self) {
        for cmd in self.commands.drain(..) {
            match cmd {
                RenderCommand::DrawRect {
                    texture_id,
                    source,
                    destination,
                } => {
                    let ctx = &mut *self.wasm_context.borrow_mut();
                    if let Some(html_image_element) = ctx.get_image(texture_id) {
                        let (sx, sy, sw, sh) = (source.x, source.y, source.width, source.height);
                        let (dx, dy, dw, dh) = (
                            destination.x,
                            destination.y,
                            destination.width,
                            destination.height,
                        );
                        let context2d = ctx.context.clone();
                        let result = context2d.draw_image_with_html_image_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                            html_image_element,
                            sx as f64,
                            sy as f64,
                            sw as f64,
                            sh as f64,
                            dx as f64,
                            dy as f64,
                            dw as f64,
                            dh as f64,
                        );
                        if let Err(e) = result {
                            let msg = format!(
                                "failed to draw texture {} {:?}",
                                texture_id.0,
                                e.as_string()
                            );
                            web_sys::console::log_1(&msg.into());
                        }
                    }
                }
                RenderCommand::FillRect { color, destination } => {
                    let ctx = &mut *self.wasm_context.borrow_mut();
                    let context2d = ctx.context.clone();
                    context2d.save();
                    context2d.set_fill_style_str(&format!(
                        "rgba({},{},{},{})",
                        color.r * 255.0,
                        color.g * 255.0,
                        color.b * 255.0,
                        color.a
                    ));
                    context2d.fill_rect(
                        destination.x as f64,
                        destination.y as f64,
                        destination.width as f64,
                        destination.height as f64,
                    );
                    context2d.restore();
                }
            }
        }
    }
}

impl Renderer for RendererWasm {
    fn name(&self) -> String {
        "WASM Renderer".to_string()
    }

    fn clear(&mut self, _color: Color) {
        let ctx = &mut *self.wasm_context.borrow_mut();
        let context2d = ctx.context.clone();
        context2d.clear_rect(
            0.,
            0.,
            (*ctx.canvas).width().into(),
            (*ctx.canvas).height().into(),
        );
    }

    fn present(&mut self) {
        self.process_commands();
    }

    fn send_command(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }
}
