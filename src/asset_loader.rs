use crate::GameOptions;
use crate::SpriteInfo;
use crate::constants::TextureId;
use jzon::JsonValue;
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

// Note that these are the same (sans Serialize derivation) structs
// as what's in build.rs. since they're tiny, we won't bother separating
// them out into a separate crate since I don't think we can derive things
// for them like that anyway unless we learned how to do features in rust
// world in order to have the build dependencies use a version with serde
// and the library one to use one without them.
#[derive(Debug)]
struct Manifest {
    spritesheets: Vec<Spritesheet>,
}

#[derive(Debug)]
struct Spritesheet {
    id: usize,
    name: String,
    asset_path: String,
    frames: Vec<SpriteFrame>,
    tags: Vec<FrameTag>,
}

impl Spritesheet {
    fn lift(obj: &JsonValue) -> AssetResult<Spritesheet> {
        let mut errors = vec![];
        let id = match obj["id"].as_usize() {
            Some(id) => id,
            None => {
                errors.push("missing or invalid id field in Spritesheet".into());
                0
            }
        };
        let name = match obj["name"].as_str() {
            Some(name) => name,
            None => {
                errors.push("missing or invalid name field in Spritesheet".into());
                ""
            }
        };
        let asset_path = match obj["asset_path"].as_str() {
            Some(asset_path) => asset_path,
            None => {
                errors.push("missing or invalid asset_path field in Spritesheet".into());
                ""
            }
        };
        let raw_frames = match obj["frames"].as_array() {
            Some(raw) => raw,
            None => {
                errors.push("missing or invalid frames field in Spritesheet".into());
                &vec![]
            }
        };
        let raw_tags = match obj["tags"].as_array() {
            Some(raw) => raw,
            None => {
                errors.push("missing or invalid tags field in Spritesheet".into());
                &vec![]
            }
        };

        let frames = raw_frames
            .into_iter()
            .filter_map(|json_val| match SpriteFrame::lift(&json_val) {
                Ok(f) => Some(f),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect();

        let tags = raw_tags
            .into_iter()
            .filter_map(|json_val| match FrameTag::lift(&json_val) {
                Ok(t) => Some(t),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(" ")
                .into());
        }

        Ok(Spritesheet {
            id,
            name: name.to_owned(),
            asset_path: asset_path.to_owned(),
            frames,
            tags,
        })
    }
}

#[derive(Debug)]
struct SpriteFrame {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    duration_ms: u32,
}

impl SpriteFrame {
    fn lift(obj: &JsonValue) -> AssetResult<SpriteFrame> {
        let Some(x) = obj["x"].as_u32() else {
            return Err("missing or invalid x field in sprite frame".into());
        };
        let Some(y) = obj["y"].as_u32() else {
            return Err("missing or invalid y field in sprite frame".into());
        };
        let Some(w) = obj["w"].as_u32() else {
            return Err("missing or invalid w field in sprite frame".into());
        };
        let Some(h) = obj["h"].as_u32() else {
            return Err("missing or invalid h field in sprite frame".into());
        };
        let Some(duration_ms) = obj["duration_ms"].as_u32() else {
            return Err("missing or invalid duration_ms field in sprite frame".into());
        };

        Ok(SpriteFrame {
            x,
            y,
            w,
            h,
            duration_ms,
        })
    }
}

fn to_sprite_info(sprite_frame: &SpriteFrame, num_frames: u32) -> SpriteInfo {
    SpriteInfo {
        start_x: sprite_frame.x,
        start_y: sprite_frame.y,
        width: sprite_frame.w,
        height: sprite_frame.h,
        frames: num_frames,
        current_frame: 0,
        // rough napkin math, 60 ticks per s,
        framerate_per_second: (60.0 * sprite_frame.duration_ms as f32 / 1000.0) as u32,
        delta: 0,
    }
}

#[derive(Debug)]
struct FrameTag {
    name: String,
    from: u32,
    to: u32,
}

impl FrameTag {
    fn lift(obj: &JsonValue) -> AssetResult<FrameTag> {
        let Some(name) = obj["name"].as_str() else {
            return Err("missing or invalid name field in frame tag".into());
        };
        let Some(from) = obj["from"].as_u32() else {
            return Err("missing or invalid from field in frame tag".into());
        };
        let Some(to) = obj["to"].as_u32() else {
            return Err("missing or invalid to field in frame tag".into());
        };

        Ok(FrameTag {
            name: name.to_owned(),
            from,
            to,
        })
    }
}

impl AssetCatalog {
    pub fn from_manifest_str(manifest_json: &str) -> AssetResult<AssetCatalog> {
        let manifest_root = parse(manifest_json)?;
        // see build.rs for shape.
        let Some(root) = manifest_root.as_object() else {
            return Err("unexpected shape of manifest file, root object was not an object".into());
        };

        let Some(spritesheets) = manifest_root["spritesheets"].as_array() else {
            return Err(
                "unexpected shape of manifest file, root object missing spritesheets list".into(),
            );
        };
        let mut errors = vec![];
        let spritesheets: Vec<Spritesheet> = spritesheets
            .into_iter()
            .filter_map(|spritesheet| match Spritesheet::lift(&spritesheet) {
                Ok(s) => Some(s),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect();

        if !errors.is_empty() {
            return Err(errors
                .into_iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(" ")
                .into());
        }

        let character_to_sprite_id = spritesheets
            .iter()
            .enumerate()
            .map(|(idx, sheet)| (sheet.name.to_owned(), idx))
            .collect();

        Ok(AssetCatalog {
            sprites: spritesheets,
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
    }
}
