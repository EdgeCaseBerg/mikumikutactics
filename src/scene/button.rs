use crate::Rect;
use crate::SpriteInfo;
use crate::constants::{
    TEXTURE_ID_FONTSHEET, TEXTURE_ID_LEEKSHEET, sprite_info_highlight, sprite_info_topbar_bg,
};
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

        // how do we write our renderer such that we can support things like this
        // where we're being a bit dynamic in the size like this? Or is that a fools
        // errand and we shouldn't be trying to make the font fit the button, but the
        // other way around?
        // renderer.send_command(RenderCommand::DrawText {
        //     font_id: FONT_ID,
        //     destination: anchor
        // })

        let glyphs = get_rects_for_str(&self.text);
        let (_, cy) = anchor.center();
        for (c, src) in glyphs.iter().enumerate() {
            // leave room for a glpyh on either side
            let glyph_display_width = anchor.width / (glyphs.len() as isize + 2);
            // mono font is same height as width (16x16 native)
            let glyph_display_height = glyph_display_width;
            let start_offset = anchor.x + glyph_display_width;
            let cell = Rect {
                x: start_offset + c as isize * glyph_display_width as isize,
                y: cy - glyph_display_height as isize / 2,
                width: glyph_display_width,
                height: glyph_display_height,
            };
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: TEXTURE_ID_FONTSHEET,
                source: *src,
                destination: cell,
            });
        }
    }
}
