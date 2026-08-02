use crate::GameOptions;
use crate::SpriteInfo;
use crate::constants::TextureId;
use crate::manifest::{Manifest, SpriteFrame, Spritesheet};
use jzon::parse;
use std::collections::HashMap;
use std::error::Error;

pub trait AssetLoader {
    fn ensure_texture_spritesheet_loaded(&mut self, sheet_id: TextureId);
}

/* SCRATCH PAD ZONE BEWARE. ROUGHING OUT IDEAS TO SEE WHAT WILL WORK OR NOT */

type AssetResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
struct AssetCatalog {
    sprites: Vec<Spritesheet>,
    character_to_sprite_id: HashMap<String, usize>,
}

// This should maybe be over on the Backend or BackendEventLoop
trait SpriteCatalogLoader {
    fn load_catalog(&mut self, game_options: &GameOptions) -> AssetCatalog;
}

fn to_sprite_info(sprite_frame: &SpriteFrame, num_frames: u32) -> SpriteInfo {
    SpriteInfo {
        start_x: sprite_frame.x,
        start_y: sprite_frame.y,
        width: sprite_frame.w,
        height: sprite_frame.h,
        frames: num_frames,
        current_frame: 0,
        // rough napkin math, 60 ticks per s, (we should move this to constants)
        framerate_per_second: (60.0 * sprite_frame.duration_ms as f32 / 1000.0) as u32,
        delta: 0,
    }
}

impl AssetCatalog {
    pub fn from_manifest_str(manifest_json: &str) -> AssetResult<AssetCatalog> {
        let manifest_root = parse(manifest_json)?;
        let manifest: Manifest = manifest_root.try_into()?;

        let character_to_sprite_id = manifest
            .spritesheets
            .iter()
            .enumerate()
            .map(|(idx, sheet)| (sheet.name.to_owned(), idx))
            .collect();

        Ok(AssetCatalog {
            sprites: manifest.spritesheets,
            character_to_sprite_id,
        })
    }
    pub fn get_sprite(&self, name: &str) -> Option<SpriteInfo> {
        let idx = self.character_to_sprite_id.get(name)?;
        let sheet = self.sprites.get(*idx)?;
        Some(to_sprite_info(&sheet.frames[0], 1))
    }
    pub fn get_sprite_in_state(&self, name: &str, state: &str) -> Option<SpriteInfo> {
        let idx = self.character_to_sprite_id.get(name)?;
        let sheet = self.sprites.get(*idx)?;
        let Some(tag) = sheet.tags.iter().find(|t| t.name == state) else {
            // if no state found, fallback to idle (1st sprite of name by convention)
            return self.get_sprite(name);
        };
        let num_frames = tag.from - tag.to;
        Some(to_sprite_info(&sheet.frames[tag.from as usize], num_frames))
    }
    pub fn get_texture_id_for_sprite(&self, name: &str) -> Option<TextureId> {
        let idx = self.character_to_sprite_id.get(name)?;
        let sheet = self.sprites.get(*idx)?;
        Some(TextureId(sheet.id))
    }
}

#[cfg(test)]
mod transition_scene_tests {
    use super::*;

    #[test]
    fn can_parse_the_manifest_shipped_with_the_game() {
        let raw = include_str!("../assets/manifest");
        let catalog = AssetCatalog::from_manifest_str(&raw);
        let Ok(catalog) = catalog else {
            panic!("expected to parse manifest but got {:?}", catalog);
        };
        let Some(rin) = catalog.get_sprite("rin") else {
            panic!("expected to contain rin but got {:?}", catalog);
        };
        assert_eq!(0, rin.start_x);
    }
}
