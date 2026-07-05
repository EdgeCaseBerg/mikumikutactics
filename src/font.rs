use crate::Rect;
use crate::asset_loader::AssetLoader;
use crate::grid_layout::GridLayout;
use std::path::PathBuf;

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub enum FontId {
    BoldPixels, // 16px monospace.
    BoldPixelScaled { pixel_size: u32 },
    // it'd be nice to do this, but PathBuf does not implement Copy
    // Dynamic { path: PathBuf, pixel_size: u32 },
}

pub trait FontMetrics {
    fn line_height(&self) -> i32;
    fn measure_width(&self, s: &str) -> i32;
}

pub type FontResult<T> = Result<T, Box<dyn Error>>;

trait FontLibrary {
    fn ensure(&mut self, id: FontId, asset_loader: &mut dyn AssetLoader) -> FontResult<()>;
    fn get(&self, id: FontId) -> Option<&LoadedFont>;
}

#[derive(Debug, Clone)]
pub struct LoadedFont {
    /// Fonts are loaded into a texture atlas that should have a corresponding texture.
    texture_id: TextureId,
    /// Glyph locations within the atlus in ascii order 0..127
    ascii_glyphs: Vec<Rect>,
}

impl LoadedFont {
    fn glyphs_for(&self, s: &str) -> Vec<Rect> {
        s.chars()
            .map(|c| {
                let ascii = c as u32;
                if ascii > 127 {
                    self.ascii_glyphs[0].clone()
                } else {
                    self.ascii_glyphs[ascii as usize - 32].clone()
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
        // As our own little convention, we'll use L as the height
        self.ascii_glyphs['L' as usize].height as i32
    }

    fn measure_width(&self, s: &str) -> i32 {
        self.glyphs_for(s)
            .into_iter()
            .fold(0, |acc, rect| acc + rect.width as i32)
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
            if ascii > 127 {
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
