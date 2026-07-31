use crate::SpriteInfo;
use crate::constants::TextureId;
use std::collections::HashMap;

pub trait AssetLoader {
    fn ensure_texture_spritesheet_loaded(&mut self, sheet_id: TextureId);
}

/* SCRATCH PAD ZONE BEWARE. ROUGHING OUT IDEAS TO SEE WHAT WILL WORK OR NOT */

type TaggedSprite = (String, SpriteInfo, TextureId);

struct SpriteCatalog {
    // Tagged sprite infos, so {(miku, {..}, 1), (idle, {..}, 1), ...}
    sprites: Vec<TaggedSprite>,
    // Name to indices in sprites that correspond to the given name.
    // For example, miku might map to [0,1,2], then to fetch the idle state you filter down to [0] since it has the tag
    character_to_sprite_id: HashMap<String, Vec<usize>>,
}

trait SpriteCatalogLoader {
    fn load_catalog(&mut self, catalog: &mut SpriteCatalog);
}

impl SpriteCatalog {
    pub fn load_ase_into_catalog(_raw_json: &str) -> Option<TextureId> {
        // TODO parse the json data into the maps
        // looking at https://docs.rs/jsonic/latest/jsonic/fn.parse.html since it has 0 dependencies
        None
    }
    pub fn get_sprite(&self, name: &str) -> Option<&SpriteInfo> {
        let indices = self.character_to_sprite_id.get(name)?;
        let idx = indices.iter().min()?;
        self.sprites.get(*idx).map(|(_, s, _)| s)
    }
    pub fn get_sprite_in_state(&self, name: &str, state: &str) -> Option<&SpriteInfo> {
        let indices = self.character_to_sprite_id.get(name)?;
        // this should be quick in practice since the lists will be very small
        for i in indices {
            if self.sprites[*i].0 == state {
                return self.sprites.get(*i).map(|(_, s, _)| s);
            }
        }
        // if no state found, fallback to idle (1st sprite of name by convention)
        self.get_sprite(name)
    }
    pub fn get_texture_id_for_sprite(&self, name: &str) -> Option<TextureId> {
        let indices = self.character_to_sprite_id.get(name)?;
        let idx = indices.iter().min()?;
        self.sprites
            .get(*idx)
            .map(|(_, _, texture_id)| texture_id)
            .copied()
    }
}
