use crate::SpriteInfo;
use crate::constants::{MusicId, SceneId, SfxId, TextureId, WeaponId};
use crate::manifest::{Manifest, SpriteFrame, Weapon};
use crate::scene::story::visual_novel::SceneLoadingContext;
use jzon::parse;
use std::collections::HashMap;
use std::error::Error;
use std::ops::RangeInclusive;
use std::path::PathBuf;
use std::rc::Rc;

pub type AssetResult<T> = Result<T, Box<dyn Error>>;

pub trait AssetLoader {
    fn load_story_scene_contents(&mut self, path: PathBuf) -> AssetResult<String>;
    fn ensure_texture_spritesheet_loaded(&mut self, sheet_id: TextureId) -> AssetResult<()>;
    fn get_catalog(&self) -> Rc<AssetCatalog>;
}

pub fn ensure_texture_by_name(
    name: &str,
    asset_loader: &mut dyn AssetLoader,
) -> AssetResult<TextureId> {
    let catalog = asset_loader.get_catalog();
    let id = catalog.get_texture_id_for_sprite(name)?;
    asset_loader.ensure_texture_spritesheet_loaded(id)?;
    Ok(id)
}

#[derive(Debug)]
pub struct AssetCatalog {
    manifest: Manifest,
    character_to_sprite_id: HashMap<String, usize>,
    music_to_music_idx: HashMap<String, usize>,
    sound_to_sfx_idx: HashMap<String, usize>,
    scene_to_scene_idx: HashMap<String, usize>,
    weapon_to_weapon_idx: HashMap<String, usize>,
    character_id_to_sprite_idx: HashMap<usize, usize>,
    music_id_to_music_idx: HashMap<usize, usize>,
    sound_id_to_sfx_idx: HashMap<usize, usize>,
    scene_id_to_scene_idx: HashMap<usize, usize>,
    weapon_id_to_weapon_idx: HashMap<usize, usize>,
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
        let music_to_music_idx = manifest
            .music
            .iter()
            .enumerate()
            .map(|(idx, music)| (music.name.to_owned(), idx))
            .collect();
        let sound_to_sfx_idx = manifest
            .sounds
            .iter()
            .enumerate()
            .map(|(idx, sound)| (sound.name.to_owned(), idx))
            .collect();
        let scene_to_scene_idx = manifest
            .scenes
            .iter()
            .enumerate()
            .map(|(idx, scene)| (scene.name.to_owned(), idx))
            .collect();
        let weapon_to_weapon_idx = manifest
            .weapons
            .iter()
            .enumerate()
            .map(|(idx, weapon)| (weapon.name.to_owned(), idx))
            .collect();

        let character_id_to_sprite_idx = manifest
            .spritesheets
            .iter()
            .enumerate()
            .map(|(idx, sheet)| (sheet.id, idx))
            .collect();
        let music_id_to_music_idx = manifest
            .music
            .iter()
            .enumerate()
            .map(|(idx, music)| (music.id, idx))
            .collect();
        let sound_id_to_sfx_idx = manifest
            .sounds
            .iter()
            .enumerate()
            .map(|(idx, sound)| (sound.id, idx))
            .collect();
        let scene_id_to_scene_idx = manifest
            .scenes
            .iter()
            .enumerate()
            .map(|(idx, scene)| (scene.id, idx))
            .collect();
        let weapon_id_to_weapon_idx = manifest
            .weapons
            .iter()
            .enumerate()
            .map(|(idx, weapon)| (weapon.id, idx))
            .collect();

        Ok(AssetCatalog {
            manifest,
            character_to_sprite_id,
            music_to_music_idx,
            sound_to_sfx_idx,
            scene_to_scene_idx,
            weapon_to_weapon_idx,
            character_id_to_sprite_idx,
            music_id_to_music_idx,
            sound_id_to_sfx_idx,
            scene_id_to_scene_idx,
            weapon_id_to_weapon_idx,
        })
    }
    pub fn get_sprite_name_for_id(&self, texture_id: TextureId) -> AssetResult<&str> {
        let Some(idx) = self.character_id_to_sprite_idx.get(&texture_id.0) else {
            return Err(format!("no sprite with id {} found", texture_id.0).into());
        };
        let Some(sheet) = &self.manifest.spritesheets.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no sprite name for texture id {} exists at index {idx}",
                texture_id.0
            )
            .into());
        };
        Ok(&sheet.name)
    }
    pub fn get_sprite(&self, name: &str) -> AssetResult<SpriteInfo> {
        let Some(idx) = self.character_to_sprite_id.get(name) else {
            return Err(format!("no sprite with name {name} found").into());
        };
        let Some(sheet) = self.manifest.spritesheets.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no sprite with name {name} exists at index {idx}",
            )
            .into());
        };
        if sheet.frames.is_empty() {
            return Err("cannot get sprite from empty frames listing".into());
        }
        Ok(to_sprite_info(&sheet.frames[0], 1))
    }
    pub fn get_sprite_in_state(&self, name: &str, state: &str) -> AssetResult<SpriteInfo> {
        let Some(idx) = self.character_to_sprite_id.get(name) else {
            return Err(format!("no sprite with name {name} found").into());
        };
        let Some(sheet) = self.manifest.spritesheets.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no sprite with name {name} exists at index {idx}",
            )
            .into());
        };
        let Some(tag) = sheet.tags.iter().find(|t| t.name == state) else {
            // if no state found, fallback to idle (1st sprite of name by convention)
            return self.get_sprite(name);
        };
        if sheet.frames.is_empty() {
            return Err("cannot get sprite from empty frames listing".into());
        }
        let num_frames = tag.to - tag.from;
        Ok(to_sprite_info(&sheet.frames[tag.from as usize], num_frames))
    }
    // things like get (portraits [miku, talking-neutral])
    pub fn get_sprite_with_tags(
        &self,
        sheet_name: &str,
        matching_all_tags: &[&str],
    ) -> AssetResult<SpriteInfo> {
        if matching_all_tags.is_empty() {
            return Err("cannot narrow sprite if no tags are provided".into());
        }
        let Some(idx) = self.character_to_sprite_id.get(sheet_name) else {
            return Err(
                format!("sheet name {} does not exist in loaded assets", sheet_name).into(),
            );
        };
        let Some(sheet) = self.manifest.spritesheets.get(*idx) else {
            return Err(format!(
                "sheet name {} does not exist in manifest assets",
                sheet_name
            )
            .into());
        };
        let mut matching = RangeInclusive::new(0, sheet.frames.len() - 1);
        for &narrowing in matching_all_tags {
            let Some(t) = sheet.tags.iter().find(|tag| {
                matching.contains(&(tag.from as usize))
                    && matching.contains(&(tag.to as usize))
                    && tag.name == narrowing
            }) else {
                return Err(format!(
                    "invalid narrowing path {:?} provided for sheet {:?}",
                    narrowing, sheet_name
                )
                .into());
            };

            matching = RangeInclusive::new(
                *matching.start().max(&(t.from as usize)),
                *matching.end().min(&(t.to as usize)),
            );
        }

        let num_frames = matching.end() - matching.start();
        Ok(to_sprite_info(
            &sheet.frames[*matching.start()],
            1 + num_frames as u32,
        ))
    }
    pub fn get_texture_id_for_sprite(&self, name: &str) -> AssetResult<TextureId> {
        let Some(idx) = self.character_to_sprite_id.get(name) else {
            return Err(format!("could not find texture id with name {name}").into());
        };
        let Some(sheet) = self.manifest.spritesheets.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no texture for name {name} exists at index {idx}"
            )
            .into());
        };
        Ok(TextureId(sheet.id))
    }
    pub fn get_asset_path(&self, id: TextureId) -> AssetResult<PathBuf> {
        let Some(idx) = self.character_id_to_sprite_idx.get(&id.0) else {
            return Err(format!("could not find asset path with id {}", id.0).into());
        };
        let Some(sheet) = self.manifest.spritesheets.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no texture for id {} exists at index {idx}",
                id.0
            )
            .into());
        };
        Ok(PathBuf::from(&sheet.asset_path))
    }
    pub fn get_music_path(&self, id: MusicId) -> AssetResult<PathBuf> {
        let Some(idx) = self.music_id_to_music_idx.get(&id.0) else {
            return Err(format!("could not find music path with id {}", id.0).into());
        };
        let Some(music) = &self.manifest.music.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no music path for id {} exists at index {idx}",
                id.0
            )
            .into());
        };
        Ok(PathBuf::from(&music.asset_path))
    }
    pub fn get_music_id_for_name(&self, name: &str) -> AssetResult<MusicId> {
        let Some(idx) = self.music_to_music_idx.get(name) else {
            return Err(format!("could not find music id with name {name}").into());
        };
        let Some(sheet) = self.manifest.music.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no music for name {name} exists at index {idx}"
            )
            .into());
        };
        Ok(MusicId(sheet.id))
    }
    pub fn get_sfx_path(&self, id: SfxId) -> AssetResult<PathBuf> {
        let Some(idx) = self.sound_id_to_sfx_idx.get(&id.0) else {
            return Err(format!("could not find sfx path with id {}", id.0).into());
        };
        let Some(sound) = self.manifest.sounds.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no sfx for id {} exists at index {idx}",
                id.0
            )
            .into());
        };
        Ok(PathBuf::from(&sound.asset_path))
    }
    pub fn get_sfx_id_for_name(&self, name: &str) -> AssetResult<SfxId> {
        let Some(idx) = self.sound_to_sfx_idx.get(name) else {
            return Err(format!("could not find sfx id with name {name}").into());
        };
        let Some(sheet) = self.manifest.sounds.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no sfx for name {name} exists at index {idx}"
            )
            .into());
        };
        Ok(SfxId(sheet.id))
    }
    pub fn get_scene_path(&self, id: SceneId) -> AssetResult<PathBuf> {
        let Some(idx) = self.scene_id_to_scene_idx.get(&id.0) else {
            return Err(format!("could not find scene asset path for id {}", id.0).into());
        };
        let scene = &self.manifest.scenes[*idx];
        Ok(PathBuf::from(&scene.asset_path))
    }
    pub fn get_scene_id_for_name(&self, name: &str) -> AssetResult<SceneId> {
        let Some(idx) = self.scene_to_scene_idx.get(name) else {
            return Err(format!("could not find scene with name {name}").into());
        };
        let Some(scene_file_asset) = self.manifest.scenes.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no scene for name {name} exists at index {idx}"
            )
            .into());
        };
        Ok(SceneId(scene_file_asset.id))
    }
    pub fn name_to_weapon(&self, name: &str) -> AssetResult<Weapon> {
        let Some(idx) = self.weapon_to_weapon_idx.get(name) else {
            return Err(format!("could not find weapon id with name {name}").into());
        };
        let Some(weapon) = self.manifest.weapons.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no weapon for name {name} exists at index {idx}"
            )
            .into());
        };
        // TODO: wonder if we should return &Weapon instead...
        Ok(weapon.clone())
    }
    pub fn id_to_weapon(&self, id: WeaponId) -> AssetResult<Weapon> {
        let Some(idx) = self.weapon_id_to_weapon_idx.get(&id.0) else {
            return Err(format!("could not find weapon with id {}", id.0).into());
        };
        let Some(weapon) = self.manifest.weapons.get(*idx) else {
            return Err(format!(
                "manifest corrupted, no weapon for id {} exists at index {idx}",
                id.0
            )
            .into());
        };
        Ok(weapon.clone())
    }
}

impl SceneLoadingContext for AssetCatalog {
    fn name_to_texture(&self, name: &str) -> AssetResult<TextureId> {
        self.get_texture_id_for_sprite(name)
    }
    fn name_to_sfx(&self, name: &str) -> AssetResult<SfxId> {
        self.get_sfx_id_for_name(name)
    }
    fn name_to_music(&self, name: &str) -> AssetResult<MusicId> {
        self.get_music_id_for_name(name)
    }
    fn name_to_scene(&self, name: &str) -> AssetResult<SceneId> {
        self.get_scene_id_for_name(name)
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
        let Ok(rin) = catalog.get_sprite("rin") else {
            panic!("expected to contain rin but got {:?}", catalog);
        };
        assert_eq!(0, rin.start_x);
    }

    #[test]
    fn get_sprite_with_tags_narrows_correctly() {
        let catalog = AssetCatalog::from_manifest_str(
            r#"{
            "spritesheets": [{
                "id": 0,
                "name": "not_it",
                "asset_path": "b/p.png",
                "frames": [],
                "tags": []
            },{
                "id": 0,
                "name": "portraits",
                "asset_path": "c/m.png",
                "frames": [
                    { "x": 0, "y": 0, "w": 64, "h": 64, "duration_ms": 100 },
                    { "x": 64, "y": 0, "w": 64, "h": 64, "duration_ms": 100 },
                    { "x": 128, "y": 0, "w": 64, "h": 64, "duration_ms": 100 },
                    { "x": 192, "y": 0, "w": 64, "h": 64, "duration_ms": 100 }
                ],
                "tags": [
                    {
                      "name": "miku",
                      "from": 0,
                      "to": 1
                    },
                    {
                      "name": "meiko",
                      "from": 2,
                      "to": 3
                    },
                    {
                      "name": "talking-idle",
                      "from": 0,
                      "to": 0
                    },
                    {
                      "name": "talking-idle",
                      "from": 2,
                      "to": 2
                    },
                    {
                      "name": "talking-mad",
                      "from": 1,
                      "to": 1
                    },
                    {
                      "name": "talking-mad",
                      "from": 3,
                      "to": 3
                    },
                    {
                      "name": "talking",
                      "from": 2,
                      "to": 3
                    }
                ]
            }],
            "sounds": [],
            "music": [],
            "scenes": [],
            "weapons": [],
            "maps": []
        }"#,
        );
        let Ok(catalog) = catalog else {
            panic!("expected to parse manifest but got {:?}", catalog);
        };
        // Finds a range of frames
        let Ok(sp) = catalog.get_sprite_with_tags("portraits", &["miku"]) else {
            panic!("expected to find sprite info but got None");
        };
        assert_eq!(sp.start_x, 0);
        assert_eq!(sp.frames, 2);

        // Finds a narrowed set of frames
        let Ok(sp) = catalog.get_sprite_with_tags("portraits", &["miku", "talking-mad"]) else {
            panic!("expected to find sprite info but got None");
        };
        assert_eq!(sp.start_x, 64);
        assert_eq!(sp.frames, 1);

        // Finds nothing on bad narrowing path
        let found = catalog.get_sprite_with_tags("portraits", &["miku", "doesnt exist"]);
        if let Err(error) = found {
            assert_eq!(
                "invalid narrowing path \"doesnt exist\" provided for sheet \"portraits\"",
                error.to_string()
            );
        } else {
            panic!("expected to not find sprite info {:?}", found);
        };

        // Finds a narrowed set of frames
        let Ok(sp) = catalog.get_sprite_with_tags("portraits", &["meiko", "talking"]) else {
            panic!("expected to find sprite info but got None");
        };
        assert_eq!(sp.start_x, 128);
        assert_eq!(sp.frames, 2);
    }
}
