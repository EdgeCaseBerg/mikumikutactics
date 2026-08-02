use crate::SpriteInfo;
use crate::constants::{MusicId, SfxId, TextureId};
use crate::manifest::{Manifest, SpriteFrame, Spritesheet};
use crate::scene::story::visual_novel::SceneLoadingContext;
use jzon::parse;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::rc::Rc;

pub type AssetResult<T> = Result<T, Box<dyn Error>>;

pub trait AssetLoader {
    fn ensure_texture_spritesheet_loaded(&mut self, sheet_id: TextureId);
    fn get_catalog(&self) -> Rc<AssetCatalog>;
}

#[derive(Debug)]
pub struct AssetCatalog {
    manifest: Manifest,
    character_to_sprite_id: HashMap<String, usize>,
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
            manifest,
            character_to_sprite_id,
        })
    }
    pub fn get_sprite(&self, name: &str) -> Option<SpriteInfo> {
        let idx = self.character_to_sprite_id.get(name)?;
        let sheet = self.manifest.spritesheets.get(*idx)?;
        Some(to_sprite_info(&sheet.frames[0], 1))
    }
    pub fn get_sprite_in_state(&self, name: &str, state: &str) -> Option<SpriteInfo> {
        let idx = self.character_to_sprite_id.get(name)?;
        let sheet = self.manifest.spritesheets.get(*idx)?;
        let Some(tag) = sheet.tags.iter().find(|t| t.name == state) else {
            // if no state found, fallback to idle (1st sprite of name by convention)
            return self.get_sprite(name);
        };
        let num_frames = tag.from - tag.to;
        Some(to_sprite_info(&sheet.frames[tag.from as usize], num_frames))
    }
    pub fn get_texture_id_for_sprite(&self, name: &str) -> Option<TextureId> {
        let idx = self.character_to_sprite_id.get(name)?;
        let sheet = self.manifest.spritesheets.get(*idx)?;
        Some(TextureId(sheet.id))
    }
    pub fn get_asset_path(&self, id: TextureId) -> Option<PathBuf> {
        let sheet = self
            .manifest
            .spritesheets
            .iter()
            .find(|sprite| sprite.id == id.0)?;
        Some(PathBuf::from(&sheet.asset_path))
    }
    pub fn get_music_path(&self, id: MusicId) -> Option<PathBuf> {
        None
    }
    pub fn get_sfx_path(&self, id: SfxId) -> Option<PathBuf> {
        None
    }
    // TODO: think about if we want SceneId at all and if that would make any sense.
    pub fn get_scene_path(&self, name: &str) -> Option<PathBuf> {
        None
    }
}

impl SceneLoadingContext for AssetCatalog {
    fn name_to_texture(&self, name: &str) -> Option<TextureId> {
        self.get_texture_id_for_sprite(name)
    }
    fn name_to_sfx(&self, name: &str) -> Option<SfxId> {
        // TODO: implement once manifest loads sfx
        None
    }
    fn name_to_music(&self, name: &str) -> Option<MusicId> {
        // TODO: implement once manifest loads music
        None
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
