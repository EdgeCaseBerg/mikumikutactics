use crate::Rect;
use crate::SpriteInfo;
use crate::constants::{
    FontId, TEXTURE_ID_FONTSHEET, TEXTURE_ID_LEEKSHEET, sprite_info_highlight,
    sprite_info_topbar_bg,
};
use crate::font::FONTSHEET_LAYOUT;
use crate::font::FontAlign;
use crate::font::FontMetrics;
use crate::font::LoadedFont;
use crate::font::draw_text_anchored;
use crate::font::get_rects_for_str;
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;

pub struct Button {
    pub(crate) text: String,
    pub(crate) rect: Rect,
    pub(crate) hovered: bool,
    pub(crate) clicked: bool,
    pub(crate) bg: SpriteInfo,
    pub(crate) highlight: SpriteInfo,
    pub(crate) font_id: FontId,
}

impl Button {
    pub(crate) fn new(text: String, rect: Rect) -> Self {
        Self {
            text,
            rect,
            hovered: false,
            clicked: false,
            bg: sprite_info_topbar_bg(),
            highlight: sprite_info_highlight(),
            font_id: FontId::BoldPixelScaled { pixel_size: 32 },
        }
    }

    pub(crate) fn relative_layout(&self, parent_layout: &GridLayout) -> GridLayout {
        let anchor = parent_layout.cell_rect(self.rect.y as usize, self.rect.x as usize);
        let width = anchor.width * self.rect.width;
        let height = anchor.height * self.rect.height;
        GridLayout {
            area: Rect {
                x: anchor.x,
                y: anchor.y,
                width,
                height,
            },
            rows: 1,
            columns: 1,
            cell_gap: 0,
        }
    }

    pub(crate) fn update(
        &mut self,
        ticks: u32,
        game_context: &GameContext,
        parent_layout: &GridLayout,
    ) {
        let layout = self.relative_layout(parent_layout);
        let Some(_) = layout.cell_for_mouse(game_context.mouse_context.position) else {
            self.hovered = false;
            self.clicked = false;
            return;
        };

        self.hovered = true;
        self.highlight.advance(ticks);

        if game_context.mouse_context.left_clicked {
            self.clicked = true;
        }
    }

    pub(crate) fn draw(&mut self, game_context: &mut GameContext, parent_layout: &GridLayout) {
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };
        let layout = self.relative_layout(parent_layout);
        let src = self.bg.get_rect();
        let anchor = layout.cell_rect(0, 0);
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: TEXTURE_ID_LEEKSHEET,
            source: src,
            destination: anchor,
        });

        if self.hovered {
            let src = self.highlight.get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: TEXTURE_ID_LEEKSHEET,
                source: src,
                destination: anchor,
            });
        }

        if let Some(font_library) = game_context.font_library.as_mut() {
            let maybe_font: Option<&LoadedFont> = font_library.get(self.font_id);
            if let Some(font) = maybe_font {
                let (_, cy) = anchor.center();
                let anchor = Rect {
                    x: anchor.x,
                    y: cy - (font.line_height() / 2) as isize,
                    width: anchor.width,
                    height: anchor.height,
                };
                draw_text_anchored(
                    &self.text,
                    &anchor,
                    &font,
                    renderer.as_mut(),
                    FontAlign::Center,
                );
            }
        }

        // let glyphs = get_rects_for_str(&self.text);
        // let (_, cy) = anchor.center();
        // for (c, src) in glyphs.iter().enumerate() {
        //     // leave room for a glpyh on either side
        //     let glyph_display_width = anchor.width / (glyphs.len() as isize + 2);
        //     // mono font is same height as width (16x16 native)
        //     let glyph_display_height = glyph_display_width;
        //     let start_offset = anchor.x + glyph_display_width;
        //     let cell = Rect {
        //         x: start_offset + c as isize * glyph_display_width as isize,
        //         y: cy - glyph_display_height as isize / 2,
        //         width: glyph_display_width,
        //         height: glyph_display_height,
        //     };
        //     renderer.send_command(RenderCommand::DrawRect {
        //         texture_id: TEXTURE_ID_FONTSHEET,
        //         source: *src,
        //         destination: cell,
        //     });
        // }
    }
}
