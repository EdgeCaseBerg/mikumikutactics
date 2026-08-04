use crate::GameResult;
use crate::Rect;
use crate::SpriteInfo;
use crate::constants::{FontId, TEXTURE_UI, TEXTURE_UI_BTN_TAG, TEXTURE_UI_FOCUS_TAG, TextureId};
use crate::font::FontAlign;
use crate::font::FontMetrics;
use crate::font::LoadedFont;
use crate::font::draw_text_anchored;
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;

pub struct Button {
    pub(crate) text: String,
    pub(crate) rect: Rect,
    pub(crate) hovered: bool,
    pub(crate) clicked: bool,
    pub(crate) bg: Option<SpriteInfo>,
    pub(crate) highlight: Option<SpriteInfo>,
    pub(crate) font_id: FontId,
    pub(crate) texture_id: TextureId,
}

impl Button {
    /// Define buttons in terms of their parent layout's grid unit space.
    /// When update and draw are called the parent layout will convert the
    /// button unit space into screen coordinates for interaction purposes
    pub(crate) fn new(text: String, rect: Rect, font_id: FontId, texture_id: TextureId) -> Self {
        Self {
            text,
            rect,
            hovered: false,
            clicked: false,
            bg: None,
            highlight: None,
            font_id,
            texture_id,
        }
    }

    pub(crate) fn init(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        let Some(asset_loader) = game_context.asset_loader.as_mut() else {
            return Err("cannot init button if asset_loader is not present in game_context".into());
        };
        let catalog = asset_loader.get_catalog();
        let Some(bg) = catalog.get_sprite_in_state(TEXTURE_UI, TEXTURE_UI_BTN_TAG) else {
            let msg =
                "cannot init button if catalog is missing button tag in ui texture sprite info";
            return Err(msg.into());
        };
        let Some(highlight) = catalog.get_sprite_in_state(TEXTURE_UI, TEXTURE_UI_FOCUS_TAG) else {
            let msg =
                "cannot init button if catalog is missing focus tag in ui texture sprite info";
            return Err(msg.into());
        };
        self.bg = Some(bg);
        self.highlight = Some(highlight);
        Ok(())
    }

    pub(crate) fn relative_layout(&self, parent_layout: &GridLayout) -> GridLayout {
        let area = self.rect.relative_rect_to_layout(parent_layout);
        GridLayout {
            area,
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
        if let Some(sprite_info) = self.highlight.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some(sprite_info) = self.bg.as_mut() {
            sprite_info.advance(ticks);
        }

        if game_context.mouse_context.left_clicked {
            self.clicked = true;
        }
    }

    pub(crate) fn draw(&self, game_context: &mut GameContext, parent_layout: &GridLayout) {
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };
        let layout = self.relative_layout(parent_layout);
        let anchor = layout.cell_rect(0, 0);
        if let Some(bg) = self.bg.as_ref() {
            let src = bg.get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: self.texture_id,
                source: src,
                destination: anchor,
            });
        }

        if self.hovered {
            if let Some(highlight) = self.highlight.as_ref() {
                let src = highlight.get_rect();
                renderer.send_command(RenderCommand::DrawRect {
                    texture_id: self.texture_id,
                    source: src,
                    destination: anchor,
                });
            }
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
                    font,
                    renderer.as_mut(),
                    FontAlign::Center,
                );
            }
        }
    }
}
