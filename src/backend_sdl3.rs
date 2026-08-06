use crate::GameResult;
use crate::Rect;
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
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use sdl3::AudioSubsystem;
use sdl3::EventPump;
use sdl3::Sdl;
use sdl3::VideoSubsystem;
use sdl3::audio::AudioFormat;
use sdl3::audio::AudioSpec;
use sdl3::audio::AudioSpecWAV;
use sdl3::audio::AudioStreamOwner;
use sdl3::event::{Event, WindowEvent};
use sdl3::filesystem::get_current_directory;
use sdl3::filesystem::{GlobFlags, glob_directory};
use sdl3::image::LoadTexture;
use sdl3::keyboard::Keycode;
use sdl3::mixer::Audio as MixerAudio;
use sdl3::mixer::Mixer;
use sdl3::mouse::MouseButton;
use sdl3::render::BlendMode;
use sdl3::render::Texture;
use sdl3::render::TextureCreator;
use sdl3::render::WindowCanvas;
use sdl3::video::WindowContext;

pub struct BackendSDL3 {
    sdl: Sdl,
}

type SDL3Texture = Texture<'static>;

pub struct SDL3Context {
    // Note: textures MUST be declared ABOVE window_canvas because
    // drop order is top to bottom and all textures need to be dropped
    // BEFORE the canvas is dropped
    textures: SDL3Textures,
    window_canvas: WindowCanvas,
    _video: VideoSubsystem,
    audio: AudioSubsystem,
    mixer: Mixer,
    catalog: Rc<AssetCatalog>,
}

struct SoundData {
    spec: AudioSpecWAV,
    duration: Duration,
}

struct Bucket {
    spec: Spec,
    streams: Vec<SfxStream>,
}

struct SfxStream {
    stream: AudioStreamOwner,
    free_at: Option<Instant>,
}

// This hack brought to you by AudioSpec not implementing Hash.
#[derive(Hash, Eq, PartialEq, Copy, Clone, Debug)]
struct Spec {
    freq: i32,
    channels: i32,
    format: sdl3::audio::AudioFormat,
}

struct SDL3Sounds {
    sound_by_id: HashMap<SfxId, SoundData>,
    music_by_id: HashMap<MusicId, MixerAudio>,
    buckets: Vec<Bucket>,
    poolsize: usize,
    base_path: PathBuf,
    context: Rc<RefCell<SDL3Context>>,
}

fn to_hashable_spec(a: &AudioSpecWAV) -> Spec {
    Spec {
        freq: a.freq,
        channels: a.channels.into(),
        format: a.format,
    }
}

fn spec_duration(spec: &AudioSpecWAV) -> Duration {
    let bytes_per_sample = match spec.format {
        AudioFormat::U8 | AudioFormat::S8 => 1,
        AudioFormat::S16LE | AudioFormat::S16BE => 2,
        AudioFormat::S32LE | AudioFormat::S32BE | AudioFormat::F32LE | AudioFormat::F32BE => 4,
        _ => 2,
    };
    let total_samples = spec.buffer().len() / (bytes_per_sample * spec.channels as usize);
    let seconds = (total_samples as f64) / spec.freq as f64;
    Duration::from_secs_f64(seconds)
}

impl SDL3Sounds {
    fn new(context: Rc<RefCell<SDL3Context>>, game_options: &GameOptions) -> Self {
        let buckets = vec![];
        let base_path = get_current_directory()
            .expect("cant get base path for audio player")
            .join(game_options.assets_path.clone());

        Self {
            buckets,
            sound_by_id: HashMap::new(),
            music_by_id: HashMap::new(),
            poolsize: game_options.audio_pool_size,
            base_path,
            context,
        }
    }
}

impl SfxStream {
    fn is_free(&self, now: Instant) -> bool {
        self.free_at.is_none_or(|t| now >= t)
    }

    fn claim(&mut self, entry: &SoundData, now: Instant) -> Result<(), Box<dyn Error>> {
        self.stream.clear()?;
        self.stream.put_data(entry.spec.buffer())?;
        self.stream.resume()?;
        self.free_at = Some(now + entry.duration);
        Ok(())
    }
}

impl Audio for SDL3Sounds {
    fn play_sfx(&mut self, id: SfxId) -> AudioResult<()> {
        let now = Instant::now();
        let Some(sound_data) = self.sound_by_id.get(&id) else {
            return Ok(());
        };
        // find the bucket
        let bucket_key = to_hashable_spec(&sound_data.spec);
        let Some(bucket) = self.buckets.iter_mut().find(|b| b.spec == bucket_key) else {
            let err = format!(
                "no bucket found for spec {:?}, ensure you called prepare after load_sfx",
                bucket_key
            );
            return Err(Box::<dyn Error>::from(err));
        };
        let stream = if let Some(stream) = bucket.streams.iter_mut().find(|s| s.is_free(now)) {
            stream
        } else {
            // All busy — steal the one that will free soonest
            bucket.streams.iter_mut().min_by_key(|s| s.free_at).unwrap()
        };
        stream.claim(sound_data, now)?;
        Ok(())
    }
    fn load_sfx(&mut self, sound_id: SfxId) -> AudioResult<()> {
        if self.sound_by_id.contains_key(&sound_id) {
            return Ok(());
        }

        let ctx = self.context.borrow();
        let Some(path) = ctx.catalog.get_sfx_path(sound_id) else {
            return Err(format!("cannot load sfx {}, not found in catalog", sound_id.0).into());
        };
        let spec = AudioSpecWAV::load_wav(path)?;
        let data = SoundData {
            duration: spec_duration(&spec),
            spec,
        };
        self.sound_by_id.insert(sound_id, data);
        Ok(())
    }
    fn play_music(&mut self, id: MusicId) -> Result<(), Box<dyn std::error::Error>> {
        let Some(mixer_audio) = self.music_by_id.get(&id) else {
            let err = format!("Could not play music with id {}, music not loaded.", id.0);
            return Err(Box::<dyn Error>::from(err));
        };

        let ctx = &mut *self.context.borrow_mut();
        ctx.mixer.pause_all()?;
        ctx.mixer.play_audio(mixer_audio)?;
        Ok(())
    }

    /// Calling this method with the same id multiple times will only load the music once.
    fn load_music(&mut self, id: MusicId) -> AudioResult<()> {
        if self.music_by_id.contains_key(&id) {
            return Ok(());
        }
        let ctx = &mut *self.context.borrow_mut();
        let Some(path) = ctx.catalog.get_music_path(id) else {
            return Err(format!("cannot load sfx {}, not found in catalog", id.0).into());
        };
        let audio = ctx.mixer.load_audio(path, true)?;
        self.music_by_id.insert(id, audio);
        Ok(())
    }
    fn load_bg_music(&mut self) -> Vec<AudioResult<MusicId>> {
        fn print_available_decoders() {
            let n = sdl3::mixer::get_num_audio_decoders();
            println!("available audio decoders: {n}");
            for i in 0..n {
                if let Some(name) = sdl3::mixer::get_audio_decoder(i) {
                    println!("  decoder {i} => {name}");
                }
            }
        }

        let supported_extensions_file = self.base_path.join("supported-audio-formats.csv");
        let read_result = fs::read_to_string(supported_extensions_file);
        let Ok(full_csv) = read_result else {
            return vec![Err(read_result.unwrap_err().into())];
        };
        let allowed_extensions: HashSet<String> =
            full_csv.split(",").map(|s| s.trim().to_string()).collect();

        let user_wav_folder = self.base_path.join("audio").join("cc-vocaloid");
        let mut ids = Vec::new();
        if let Ok(globbed) = glob_directory(&user_wav_folder, Some("*"), GlobFlags::CASEINSENSITIVE)
        {
            for path in &globbed {
                let filename = path.file_name();
                if filename.is_none() {
                    continue;
                }
                let Some(filename) = filename.unwrap().to_str() else {
                    continue;
                };

                if filename == "README.txt" {
                    continue;
                }

                let Some(extension) = path.extension().and_then(|os_str| os_str.to_str()) else {
                    continue;
                };
                if !allowed_extensions.contains(extension) {
                    eprintln!(
                        "cannot load {:?} file type not supported by available decoders or extension list, see supported-audio-formats.csv and decoder list below",
                        filename
                    );
                    print_available_decoders();
                    continue;
                }

                let desired_id = filename[0..filename.len() - extension.len() - 1].parse::<usize>();
                if let Ok(desired_id) = desired_id {
                    let music_id = MusicId(desired_id);
                    if self.music_by_id.contains_key(&music_id) {
                        // Tricky tricky, make sure you put the music id if we're re-loading the level scene
                        // and generating the list of music ids again. Just because we don't need to load the
                        // music audio into the hashmap, doesn't mean we don't need to return the id here!
                        ids.push(Ok(music_id));
                    }
                    let ctx = &mut *self.context.borrow_mut();
                    match ctx.mixer.load_audio(user_wav_folder.join(path), true) {
                        Ok(audio) => {
                            self.music_by_id.insert(music_id, audio);
                            ids.push(Ok(music_id));
                        }
                        Err(e) => {
                            ids.push(Err(e.into()));
                        }
                    }
                } else {
                    let msg = format!(
                        "cannot load music file {} please name it numerically in the order you want played",
                        filename
                    );
                    let e = Box::<dyn Error>::from(msg);
                    ids.push(Err(e));
                }
            }
        }
        ids
    }
    fn music_duration_seconds(&self, id: MusicId) -> AudioResult<Duration> {
        let Some(mixer_audio) = self.music_by_id.get(&id) else {
            let err = format!(
                "Could not compute duration of music with id {}, music not loaded.",
                id.0
            );
            return Err(Box::<dyn Error>::from(err));
        };
        let duration = mixer_audio.frames_to_ms(mixer_audio.duration());
        Ok(Duration::from_millis(duration.try_into()?))
    }

    fn prepare(&mut self) -> Vec<AudioResult<()>> {
        let specs_to_prepare: HashSet<Spec> = self
            .sound_by_id
            .values()
            .map(|v| to_hashable_spec(&v.spec))
            .collect();

        // Clean out anything we DONT need anymore:
        let mut already_exist = HashSet::new();
        self.buckets.retain_mut(|bucket| {
            let exists = specs_to_prepare.contains(&bucket.spec);
            if exists {
                already_exist.insert(bucket.spec);
            }
            for SfxStream { stream, .. } in &bucket.streams {
                let _ = stream.pause();
            }
            exists
        });

        let mut stream_failures = vec![];
        let ctx = &mut *self.context.borrow_mut();
        for spec_needs_bucket in specs_to_prepare.difference(&already_exist) {
            let mut streams = Vec::with_capacity(self.poolsize);
            for _ in 0..self.poolsize {
                let device = ctx.audio.default_playback_device();
                let stream = device.open_device_stream(
                    Some(AudioSpec {
                        freq: Some(spec_needs_bucket.freq),
                        channels: Some(spec_needs_bucket.channels),
                        format: Some(spec_needs_bucket.format),
                    })
                    .as_ref(),
                );
                if let Ok(stream) = stream {
                    let stream = SfxStream {
                        stream,
                        free_at: None,
                    };
                    streams.push(stream);
                } else {
                    // "could not open logical device for spec"
                    stream_failures.push(stream.map(|_| ()).map_err(|e| e.into()));
                }
            }
            self.buckets.push(Bucket {
                spec: *spec_needs_bucket,
                streams,
            })
        }
        stream_failures
    }
}

pub struct SDL3Textures {
    texture_by_id: HashMap<TextureId, SDL3Texture>,
    // ORDER OF STRUCT IS IMPORTANT BECAUSE OF DROP ORDER
    // WE DROP THE TEXTURES PRIOR TO THE CREATOR GOING AWAY
    texture_creator: TextureCreator<WindowContext>,
}

impl SDL3Textures {
    fn from(texture_creator: TextureCreator<WindowContext>) -> Self {
        SDL3Textures {
            texture_creator,
            texture_by_id: HashMap::new(),
        }
    }

    fn get_texture(&self, texture_id: TextureId) -> Option<&SDL3Texture> {
        self.texture_by_id.get(&texture_id)
    }

    fn load(&mut self, id: TextureId, path: PathBuf) {
        let tex = self.texture_creator.load_texture(path).unwrap();
        let tex = make_static(tex);
        self.texture_by_id.insert(id, tex);
    }
}

struct AssetLoaderSDL3 {
    context: Rc<RefCell<SDL3Context>>,
    base_path: PathBuf,
}

impl AssetLoaderSDL3 {
    fn new(context: Rc<RefCell<SDL3Context>>, game_options: &GameOptions) -> Self {
        let base = get_current_directory().expect("cant get base path");
        let base = base.join(game_options.assets_path.clone());
        Self {
            context,
            base_path: base,
        }
    }
}

impl AssetLoader for AssetLoaderSDL3 {
    fn ensure_texture_spritesheet_loaded(&mut self, id: TextureId) {
        let ctx = &mut *self.context.borrow_mut();
        if ctx.textures.get_texture(id).is_some() {
            return;
        }
        let asset_path = ctx
            .catalog
            .get_asset_path(id)
            .unwrap_or_else(|| panic!("could not ensure texture would be loaded for id {:?}", id));
        let asset_path = self.base_path.join(asset_path);
        ctx.textures.load(id, asset_path);
    }

    fn get_catalog(&self) -> Rc<AssetCatalog> {
        let ctx = &*self.context.borrow_mut();
        ctx.catalog.clone()
    }
}

// Alchemy! we do this to shunt off the lifetime the sdl3 lib sets on the textures.
// both it and we know that its lifetime is tied to the texture_creator, but they
// didn't represent this by defining a lifetime on the creator, and we don't need to
// care or worry about this because the texture_creator is owned by SDL3Textures and
// so when it goes out of scope it can drop everything. I imagine I might need to
// implement a Drop for SDL3Textures to make sure that happens, but then again, its
// dropping point is _probably_ going to be the end of the program so... eh.
// The SDL3 docs says we should destroy it when we're done https://wiki.libsdl.org/SDL3_image/IMG_LoadTexture
//
fn make_static(tex: Texture) -> Texture<'static> {
    unsafe { std::mem::transmute(tex) }
}

impl BackendSDL3 {
    pub fn new(_game_options: &GameOptions) -> Self {
        let sdl_handle = sdl3::init().expect("failed to init SDL");
        BackendSDL3 { sdl: sdl_handle }
    }
}

struct StandardClock {
    start: Instant,
}

impl StandardClock {
    fn new() -> StandardClock {
        Self {
            start: Instant::now(),
        }
    }
}

impl Clock for StandardClock {
    fn elapsed_since_start(&self) -> u128 {
        self.start.elapsed().as_nanos()
    }
    fn sleep(&self) {
        // Arbitrary constant for now.
        ::std::thread::sleep(Duration::from_millis(2));
    }
}

impl Backend for BackendSDL3 {
    fn create_clock(&self) -> Box<dyn Clock> {
        Box::new(StandardClock::new())
    }

    fn create_event_loop(&self, game_options: &GameOptions) -> Box<dyn BackendEventLoop> {
        // Load the manifest before starting sdl stuff because if we have no manifest everything
        // will be wonky later.
        let manifest = fs::read_to_string(game_options.assets_path.join("manifest"))
            .expect("could not load manifest file for assets");
        let catalog =
            AssetCatalog::from_manifest_str(&manifest).expect("corrupt manifest: failed to parse");

        let event_pump = self.sdl.event_pump().unwrap();

        let video_subsystem = self.sdl.video().expect("failed to get video context");
        let audio_subsystem = self.sdl.audio().expect("failed to get audio context");
        let mixer_subsystem = Mixer::open_device(None).expect("failed to create mixer context");
        // Side note, window to borderless and all that would need to re-create window and derived canvases
        let window = video_subsystem
            .window(
                &game_options.name,
                game_options.window_width,
                game_options.window_height,
            )
            .position_centered()
            .resizable()
            .build()
            .expect("failed to build window");

        let canvas = window.into_canvas();
        let textures = SDL3Textures::from(canvas.texture_creator());

        // If we end up having some custom form of cursor for each scene then we can do this
        // self.sdl.mouse().show_cursor(false);

        let e = EventLoopSDL3 {
            event_pump,
            context: Rc::new(RefCell::new(SDL3Context {
                _video: video_subsystem,
                window_canvas: canvas,
                textures,
                audio: audio_subsystem,
                mixer: mixer_subsystem,
                catalog: catalog.into(),
            })),
        };
        Box::new(e)
    }
}

pub struct EventLoopSDL3 {
    event_pump: EventPump,
    context: Rc<RefCell<SDL3Context>>, // in a rc + refcell because we need to be able to pass around &mut for shared stuff.
}

impl BackendEventLoop for EventLoopSDL3 {
    fn run(&mut self, mut game: Game, mut game_context: GameContext) -> GameResult<()> {
        let scene = game.scene.as_mut();
        if let Some(scene) = scene {
            scene.init(&mut game_context);
        }

        // initialize the audio pool if the scene has queued things up
        let audio = game_context.audio.as_mut();
        if let Some(audio) = audio {
            let _ = audio.prepare();
        }

        'running: loop {
            for event in self.event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::MouseMotion {
                        mousestate, x, y, ..
                    } => {
                        game_context.mouse_context.update(
                            mousestate.left(),
                            mousestate.right(),
                            Some((x, y)),
                        );
                    }
                    Event::MouseButtonDown {
                        mouse_btn, x, y, ..
                    } => {
                        game_context.mouse_context.update(
                            mouse_btn == MouseButton::Left,
                            mouse_btn == MouseButton::Right,
                            Some((x, y)),
                        );
                    }
                    Event::Window {
                        win_event: WindowEvent::Resized(w, h),
                        ..
                    } => {
                        game_context.screen_size = (w as u32, h as u32);
                    }
                    _ => {}
                }
            }
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
                break;
            }
        }
        Ok(())
    }

    fn new_renderer(&self, _game_options: &GameOptions) -> Box<dyn Renderer> {
        let r = RendererSDL3 {
            context: self.context.clone(),
            commands: Vec::with_capacity(32),
        };
        Box::new(r)
    }

    fn create_asset_loader(&self, game_options: &GameOptions) -> Box<dyn AssetLoader> {
        let a = AssetLoaderSDL3::new(self.context.clone(), game_options);
        Box::new(a)
    }

    fn create_audio(&self, game_options: &GameOptions) -> Box<dyn Audio> {
        let s = SDL3Sounds::new(self.context.clone(), game_options);
        Box::new(s)
    }

    fn create_font_library(&self, _game_options: &GameOptions) -> Box<dyn FontLibrary> {
        Box::new(BitmapFontLibrary::default())
    }
}

struct RendererSDL3 {
    context: Rc<RefCell<SDL3Context>>,
    commands: Vec<RenderCommand>,
}

impl RendererSDL3 {
    // Internally used before presenting. Drains all commands
    // in order to enque all the work to SDL3 that we want done
    // per frame.
    fn process_commands(&mut self) {
        for cmd in self.commands.drain(..) {
            match cmd {
                RenderCommand::DrawRect {
                    texture_id,
                    source,
                    destination,
                } => {
                    let ctx = &mut *self.context.borrow_mut();
                    if let Some(texture) = ctx.textures.get_texture(texture_id) {
                        let src: sdl3::rect::Rect = source.into();
                        let dst: sdl3::rect::Rect = destination.into();
                        ctx.window_canvas
                            .copy(texture, src, dst)
                            .unwrap_or_else(|_| {
                                // TODO: when we have a real GameError, return an error!
                                let s = &format!("failed to draw texture {}", texture_id.0);
                                eprintln!("{}", s);
                            });
                    }
                }
                RenderCommand::FillRect { color, destination } => {
                    let ctx = &mut *self.context.borrow_mut();
                    let previous_color = ctx.window_canvas.draw_color();
                    let previous_mode = ctx.window_canvas.blend_mode();
                    let dst: sdl3::rect::Rect = destination.into();
                    ctx.window_canvas.set_draw_color(color.to_sdl3());
                    ctx.window_canvas.set_blend_mode(BlendMode::Blend);
                    ctx.window_canvas.fill_rect(dst).unwrap_or_else(|_| {
                        // TODO: when we have a real GameError, return an error!
                        let s =
                            &format!("failed to fill rect {:?}", (destination.x, destination.y));
                        eprintln!("{}", s);
                    });
                    ctx.window_canvas.set_draw_color(previous_color);
                    ctx.window_canvas.set_blend_mode(previous_mode);
                }
            }
        }
    }
}

impl Renderer for RendererSDL3 {
    fn name(&self) -> String {
        "SDL3 Renderer".to_string()
    }

    fn clear(&mut self, color: Color) {
        let mut ctx = self.context.borrow_mut();
        ctx.window_canvas.set_draw_color(color.to_sdl3());
        ctx.window_canvas.clear();
    }

    fn present(&mut self) {
        self.process_commands();
        let mut ctx = self.context.borrow_mut();
        ctx.window_canvas.present();
    }

    fn send_command(&mut self, cmd: RenderCommand) {
        self.commands.push(cmd);
    }
}

impl Color {
    pub fn to_sdl3(&self) -> sdl3::pixels::Color {
        let (r, g, b, a) = (*self).into();
        sdl3::pixels::Color::RGBA(r, g, b, a)
    }
}

impl From<Rect> for sdl3::rect::Rect {
    fn from(r: Rect) -> Self {
        sdl3::rect::Rect::new(r.x as i32, r.y as i32, r.width as u32, r.height as u32)
    }
}
