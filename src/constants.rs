use crate::SpriteInfo;
use std::path::PathBuf;

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct TextureId(pub usize);

pub const TEXTURE_ID_MIKU: TextureId = TextureId(0);
pub const TEXTURE_ID_PORTRAIT: TextureId = TextureId(1);
pub const TEXTURE_ID_LEEKSHEET: TextureId = TextureId(2);
pub const TEXTURE_ID_FONTSHEET: TextureId = TextureId(3);
pub const TEXTURE_ID_GAMEOVER: TextureId = TextureId(4);
pub const TEXTURE_ID_MIKU_WAVE: TextureId = TextureId(5);
pub const TEXTURE_ID_TITLE_BG: TextureId = TextureId(6);
pub const TEXTURE_ID_STORY_BG: TextureId = TextureId(7);

pub fn id_to_relative_path(id: TextureId) -> PathBuf {
    match id {
        TEXTURE_ID_LEEKSHEET => PathBuf::new().join("made-by-me").join("leek-bg1-bg2.png"),
        TEXTURE_ID_GAMEOVER => PathBuf::new().join("made-by-me").join("GameOver.png"),
        TEXTURE_ID_PORTRAIT => PathBuf::new()
            .join("chaim-vester")
            .join("portraits-spritesheet.png"),
        TEXTURE_ID_MIKU => PathBuf::new().join("dance.png"),
        TEXTURE_ID_FONTSHEET => PathBuf::new()
            .join("webfontkit-BoldPixels")
            .join("BoldPixels-edit.png"),
        TEXTURE_ID_MIKU_WAVE => PathBuf::new().join("made-by-me").join("miku-wave.png"),
        TEXTURE_ID_TITLE_BG => PathBuf::new().join("made-by-me").join("titlescreen.png"),
        TEXTURE_ID_STORY_BG => PathBuf::new()
            .join("made-by-me")
            .join("planning-layout.png"),
        _ => PathBuf::new(), // could maybe panic here, though stuff like unreachable! isnt a const function
    }
}

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct SfxId(pub usize);

pub const SFX_ID_BLIP: SfxId = SfxId(0);

// These match the manifest file's texture names
pub const SFX_MEME: &str = "meme";
pub const SFX_BLIP: &str = "blipSelect";

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct MusicId(pub usize);
pub const MUSIC_ID_PACHEBAL: MusicId = MusicId(0);
pub const MUSIC_ID_MOON: MusicId = MusicId(1000);
pub const MUSIC_ID_QUIT: MusicId = MusicId(1001);
pub const MUSIC_ID_TETO: MusicId = MusicId(1002);

// Enable loading arbitrary songs via ids above 1 (wav only)
pub fn music_id_to_relative_path(id: MusicId) -> PathBuf {
    let base = PathBuf::new().join("audio");
    let wavs = PathBuf::new().join("audio").join("cc-vocaloid");
    match id {
        MUSIC_ID_PACHEBAL => base.join("Miku Pachebal.wav"),
        MUSIC_ID_MOON => base.join("miku fly to moon.wav"),
        MUSIC_ID_QUIT => base.join("selectedQuit.wav"),
        MUSIC_ID_TETO => base.join("tetowins.wav"),
        _ => wavs.join(format!("{}.wav", id.0)),
    }
}

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub enum FontId {
    BoldPixels, // 16px monospace.
    BoldPixelScaled { pixel_size: u32 },
    // it'd be nice to do this, but PathBuf does not implement Copy
    // Dynamic { path: PathBuf, pixel_size: u32 },
}

pub const fn sprite_info_leek() -> SpriteInfo {
    SpriteInfo {
        start_x: 0,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_grass() -> SpriteInfo {
    SpriteInfo {
        start_x: 64,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_road() -> SpriteInfo {
    SpriteInfo {
        start_x: 32,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_miku() -> SpriteInfo {
    SpriteInfo {
        start_x: 0,
        start_y: 0,
        width: 71,
        height: 54,
        frames: 6,
        current_frame: 0,
        framerate_per_second: 10,
        delta: 0,
    }
}

pub const fn sprite_info_highlight() -> SpriteInfo {
    SpriteInfo {
        start_x: 96,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 4,
        current_frame: 0,
        framerate_per_second: 4,
        delta: 0,
    }
}

pub const fn sprite_info_miku_tower() -> SpriteInfo {
    SpriteInfo {
        start_x: 32 * 7,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 2,
        current_frame: 0,
        framerate_per_second: 8,
        delta: 0,
    }
}

pub const fn sprite_info_rin_tower() -> SpriteInfo {
    SpriteInfo {
        start_x: 32 * 9,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 2,
        current_frame: 0,
        framerate_per_second: 4,
        delta: 0,
    }
}

pub const fn sprite_info_luka_tower() -> SpriteInfo {
    SpriteInfo {
        start_x: 32 * 11,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 2,
        current_frame: 0,
        framerate_per_second: 16,
        delta: 0,
    }
}

pub const fn sprite_info_teto() -> SpriteInfo {
    SpriteInfo {
        start_x: 32 * 13,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 16,
        delta: 0,
    }
}

pub const fn sprite_info_teto_walking() -> SpriteInfo {
    SpriteInfo {
        start_x: 32 * 14,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 2,
        current_frame: 0,
        framerate_per_second: 16,
        delta: 0,
    }
}

pub const fn sprite_info_energy() -> SpriteInfo {
    SpriteInfo {
        start_x: 32 * 16,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_topbar_bg() -> SpriteInfo {
    SpriteInfo {
        start_x: 32 * 17,
        start_y: 0,
        width: 32,
        height: 32,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_gameover_miku() -> SpriteInfo {
    SpriteInfo {
        start_x: 0,
        start_y: 0,
        width: 480,
        height: 320,
        frames: 3,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_portrait() -> SpriteInfo {
    SpriteInfo {
        start_x: 0,
        start_y: 0,
        width: 2478,
        height: 402,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_miku_wave() -> SpriteInfo {
    SpriteInfo {
        start_x: 0,
        start_y: 0,
        width: 320,
        height: 320,
        frames: 2,
        current_frame: 0,
        framerate_per_second: 30,
        delta: 0,
    }
}

pub const fn sprite_info_title() -> SpriteInfo {
    SpriteInfo {
        start_x: 0,
        start_y: 0,
        width: 480,
        height: 270,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}

pub const fn sprite_info_planning_layout_bg() -> SpriteInfo {
    SpriteInfo {
        start_x: 0,
        start_y: 0,
        width: 1280,
        height: 720,
        frames: 1,
        current_frame: 0,
        framerate_per_second: 60,
        delta: 0,
    }
}
