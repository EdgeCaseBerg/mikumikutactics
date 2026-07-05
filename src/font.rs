use crate::Rect;
use crate::asset_loader::AssetLoader;
use crate::constants::{FontId, TEXTURE_ID_FONTSHEET, TextureId};
use crate::grid_layout::GridLayout;
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;

pub trait FontMetrics {
    fn line_height(&self) -> i32;
    fn measure_width(&self, s: &str) -> i32;
}

pub type FontResult<T> = Result<T, Box<dyn Error>>;

pub trait FontLibrary {
    fn ensure(&mut self, id: FontId, asset_loader: &mut dyn AssetLoader) -> FontResult<()>;
    fn get(&self, id: FontId) -> Option<&LoadedFont>;
}

#[derive(Debug, Clone)]
pub struct LoadedFont {
    pub id: FontId,
    /// Fonts are loaded into a texture atlas that should have a corresponding texture.
    pub texture_id: TextureId,
    /// Glyph locations within the atlus in ascii order 0..127 (Fonts without characters should pad out 0..32 with spaces or similar markers)
    pub ascii_glyphs: Vec<Rect>,
}

impl LoadedFont {
    pub fn glyphs_for(&self, s: &str) -> Vec<Rect> {
        s.chars()
            .map(|c| {
                let ascii = c as u32;
                if ascii > 127 {
                    self.ascii_glyphs[0].clone()
                } else {
                    self.ascii_glyphs[ascii as usize].clone()
                }
            })
            .collect()
    }
}

impl FontMetrics for LoadedFont {
    fn line_height(&self) -> i32 {
        // ascii_glyphs.iter().max_by_key(|r| r.height)
        if self.ascii_glyphs.len() <= 0 {
            return 0; // This should never happen. But still.
        }
        // As our own little convention, we'll use L as the height because its nice and tall and its arbitrary
        self.ascii_glyphs['L' as usize].height as i32
    }

    fn measure_width(&self, s: &str) -> i32 {
        self.glyphs_for(s)
            .into_iter()
            .fold(0, |acc, rect| acc + rect.width as i32)
    }
}

pub struct BitmapFontLibrary {
    pub fonts: HashMap<FontId, LoadedFont>,
}

impl BitmapFontLibrary {
    pub fn new() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }
}

impl FontLibrary for BitmapFontLibrary {
    fn ensure(&mut self, id: FontId, asset_loader: &mut dyn AssetLoader) -> FontResult<()> {
        let texture_id = match id {
            FontId::BoldPixels | FontId::BoldPixelScaled { .. } => TEXTURE_ID_FONTSHEET,
        };
        asset_loader.ensure_texture_spritesheet_loaded(texture_id);
        // NOT unsafe, we literally make the bytes here.
        let ascii = unsafe {
            let bytes: Vec<u8> = (0u8..=127).collect();
            String::from_utf8_unchecked(bytes)
        };
        // In a backend specific font library, we'd render the font to a surface, load it into a texture
        // and then create the appropriate rects to fetch each letter out as needed.
        let ascii_glyphs = get_rects_for_str(&ascii);
        let font = LoadedFont {
            id,
            texture_id,
            ascii_glyphs,
        };
        self.fonts.insert(id, font);
        Ok(())
    }
    fn get(&self, id: FontId) -> Option<&LoadedFont> {
        self.fonts.get(&id)
    }
}

/* BitmapFontMetrics is from experimentation with bitmaps before we added the font library in
 * and is preserved while we're working on migrating to the loaded font interface.
 */
pub struct BitmapFontMetrics {
    pub layout: GridLayout,
}

impl FontMetrics for BitmapFontMetrics {
    fn line_height(&self) -> i32 {
        let (_, h) = self.layout.cell_size();
        h as i32
    }

    fn measure_width(&self, s: &str) -> i32 {
        let (w, _) = self.layout.cell_size();
        (w * s.len() as isize) as i32
    }
}

pub const FONTSHEET_LAYOUT: GridLayout = {
    GridLayout {
        area: Rect {
            x: 0,
            y: 0,
            width: 145,
            height: 1412,
        },
        rows: 83,
        columns: 16,
        cell_gap: 1,
    }
};

pub fn get_rects_for_str(str: &str) -> Vec<Rect> {
    // font sheet starts at space, so -32 from the character's ascii value
    // to get the index. Then we need to convert that index into
    // This is horrifically inefficient, but for testing it should be okay:
    let cells: Vec<Rect> = FONTSHEET_LAYOUT.iter_cells().map(|(_, _, r)| r).collect();
    str.chars()
        .map(|c| {
            let ascii = c as u32;
            if ascii > 127 || ascii < 32 {
                cells[0].clone()
            } else {
                cells[ascii as usize - 32].clone()
            }
        })
        .collect()
}

pub fn center_font_in_tile(anchor: Rect, font_cell: Rect, str_index: isize) -> Rect {
    let (cx, cy) = anchor.center();
    Rect {
        x: cx + str_index * font_cell.width,
        y: cy - font_cell.height / 2,
        width: font_cell.width,
        height: font_cell.height,
    }
}
