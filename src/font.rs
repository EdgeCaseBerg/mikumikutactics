use crate::Rect;
use crate::asset_loader::AssetLoader;
use crate::constants::{FontId, TextureId};
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::renderer::Renderer;
use std::collections::HashMap;
use std::error::Error;

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
    /// Scale for font, 1.0 is 1:1 to the font original size
    pub scale: f32,
}

impl LoadedFont {
    pub fn glyphs_for(&self, s: &str) -> Vec<Rect> {
        s.chars()
            .map(|c| {
                let ascii = c as u32;
                if ascii > 127 {
                    self.ascii_glyphs[0]
                } else {
                    self.ascii_glyphs[ascii as usize]
                }
            })
            .collect()
    }
}

impl FontMetrics for LoadedFont {
    fn line_height(&self) -> i32 {
        if self.ascii_glyphs.is_empty() {
            return 0; // This should never happen. But still.
        }
        // As our own little convention, we'll use L as the height because its nice and tall and its arbitrary
        // if we ever needed to deal with non mono fonts, then we'd probably want to do something like
        // ascii_glyphs.iter().max_by_key(|r| r.height)
        let glyph_height = self.ascii_glyphs['L' as usize].height as i32;
        (self.scale * glyph_height as f32) as i32
    }

    fn measure_width(&self, s: &str) -> i32 {
        let scaled = self.scale
            * self
                .glyphs_for(s)
                .into_iter()
                .fold(0.0, |acc, rect| acc + rect.width as f32);
        scaled as i32
    }
}

pub struct BitmapFontLibrary {
    pub fonts: HashMap<FontId, LoadedFont>,
}

impl Default for BitmapFontLibrary {
    fn default() -> Self {
        Self {
            fonts: HashMap::new(),
        }
    }
}

const FONT_NAME: &str = "BoldPixels";
impl FontLibrary for BitmapFontLibrary {
    fn ensure(&mut self, id: FontId, asset_loader: &mut dyn AssetLoader) -> FontResult<()> {
        let scale = match id {
            FontId::BoldPixels => 1.0,
            FontId::BoldPixelScaled { pixel_size } => pixel_size as f32 / 16.0,
        };
        let catalog = asset_loader.get_catalog();
        let texture_id = catalog.get_texture_id_for_sprite(FONT_NAME)?;
        asset_loader.ensure_texture_spritesheet_loaded(texture_id);
        // NOT unsafe, we literally make the bytes here.
        let ascii = unsafe {
            let bytes: Vec<u8> = (0u8..=127).collect();
            String::from_utf8_unchecked(bytes)
        };
        // In a backend specific font library, we'd render the font to a surface, load it into a texture
        // and then create the appropriate rects to fetch each letter out as needed. Right now the method
        // here ONLY handles the BoldPixels properly since get_rects_for_str is using the layout for that
        // but that's ok. We can address the issue when we add a new font.
        let ascii_glyphs = get_rects_for_str(&ascii);
        let font = LoadedFont {
            id,
            texture_id,
            ascii_glyphs,
            scale,
        };
        self.fonts.insert(id, font);
        Ok(())
    }
    fn get(&self, id: FontId) -> Option<&LoadedFont> {
        self.fonts.get(&id)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FontAlign {
    Left,
    Center,
    Right,
}

pub fn draw_text_anchored(
    text: &str,
    anchor: &Rect,
    font: &LoadedFont,
    renderer: &mut dyn Renderer,
    align: FontAlign,
) {
    let glyphs = font.glyphs_for(text);
    let glyph_display_height = font.line_height() as isize;
    let align_offset = match align {
        FontAlign::Left => 0,
        FontAlign::Center => {
            let fw = font.measure_width(text) as isize;
            let center = anchor.width / 2;
            center - fw / 2
        }
        FontAlign::Right => {
            // [ offset | --- text width --- ] right side of anchor
            let fw = font.measure_width(text) as isize;
            anchor.width - fw
        }
    };

    for (c, src) in glyphs.iter().enumerate() {
        let glyph_display_width = (src.width as f32 * font.scale) as isize;
        let x_offset = align_offset + c as isize * glyph_display_width;
        let cell = Rect {
            x: anchor.x + x_offset,
            y: anchor.y,
            width: glyph_display_width,
            height: glyph_display_height,
        };
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: font.texture_id,
            source: *src,
            destination: cell,
        });
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
                cells[0]
            } else {
                cells[ascii as usize - 32]
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
