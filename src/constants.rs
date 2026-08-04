use crate::SpriteInfo;

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct TextureId(pub usize);

// These match manifest file texture names, stored in one place
// so that we don't have to deal with magic string sprawl. We
// only need these for non-dynamically loaded scenes where a
// scene script can't handle things for us.
// Though perhaps we should consider re-using the scenes as a way to state the textures we need.
// maybe a bit of a hack, but having a txt file with BACKGROUND ui\nBACKGROUND titlescreen listed in it
// and just not using it as a vn machine program would be a little bit simpler in the world and we could
// just bake them into the binary or something via include_str in the init + ensure_ops works and a test that
// validates that it passes so we don't hit runtime problems from tweaks.
pub const TEXTURE_UI: &str = "ui";
pub const TEXTURE_UI_BTN_TAG: &str = "button";
pub const TEXTURE_UI_FOCUS_TAG: &str = "focus";
pub const TEXTURE_TITLE_BG: &str = "titlescreen";
pub const TEXTURE_PLANNING_LAYOUT_BG: &str = "planning-layout";

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct SfxId(pub usize);

// These match the manifest file's texture names
pub const SFX_MEME: &str = "meme";
pub const SFX_BLIP: &str = "blipSelect";

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct MusicId(pub usize);

// This must match a manifest name in sounds array
pub const MUSIC_PACHEBAL: &str = "pachebal";

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub enum FontId {
    BoldPixels, // 16px monospace.
    BoldPixelScaled { pixel_size: u32 },
    // it'd be nice to do this, but PathBuf does not implement Copy
    // Dynamic { path: PathBuf, pixel_size: u32 },
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
