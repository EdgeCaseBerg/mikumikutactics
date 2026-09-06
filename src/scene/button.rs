use crate::GameResult;
use crate::ReadyState;
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

#[derive(Clone, Debug)]
pub struct Button {
    pub text: String,
    pub rect: Rect,
    pub hovered: bool,
    pub clicked: bool,
    pub bg: Option<SpriteInfo>,
    pub highlight: Option<SpriteInfo>,
    pub font_id: FontId,
    pub texture_id: TextureId,
    pub click_state: ReadyState,
    pub cooldown_ticks: u32,
    pub waiting_for_release: bool,
}

// TODO idea:
//
// what if Button { text, rect, hovered, clicked } -> as_drawable(game_content) -> DrawableButton { b: Button, bg, highlight, font, texture }
//
// I suppose font could still be set today since they're somewhat special as far as Ids go, but texture could be loaded durign conversion.
// or I suppose rather than as_drawable, something like .as_large() .as_small() could set different fonts or textures if we want different
// style buttons and all that

impl Button {
    /// Define buttons in terms of their parent layout's grid unit space.
    /// When update and draw are called the parent layout will convert the
    /// button unit space into screen coordinates for interaction purposes
    pub fn new(text: String, rect: Rect, font_id: FontId, texture_id: TextureId) -> Self {
        Self {
            text,
            rect,
            hovered: false,
            clicked: false,
            bg: None,
            highlight: None,
            font_id,
            texture_id,
            click_state: ReadyState::Ready,
            cooldown_ticks: 10, // TODO: take it as new if we ever need buttons to feel different from each other.
            waiting_for_release: false,
        }
    }

    pub fn init(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        let Some(asset_loader) = game_context.asset_loader.as_mut() else {
            return Err("cannot init button if asset_loader is not present in game_context".into());
        };
        let catalog = asset_loader.get_catalog();
        let bg = catalog.get_sprite_in_state(TEXTURE_UI, TEXTURE_UI_BTN_TAG)?;
        let highlight = catalog.get_sprite_in_state(TEXTURE_UI, TEXTURE_UI_FOCUS_TAG)?;
        self.bg = Some(bg);
        self.highlight = Some(highlight);
        Ok(())
    }

    pub fn relative_layout(&self, parent_layout: &GridLayout) -> GridLayout {
        let area = self.rect.relative_rect_to_layout(parent_layout);
        GridLayout {
            area,
            rows: 1,
            columns: 1,
            cell_gap: 0,
        }
    }

    pub fn update(
        &mut self,
        ticks: u32,
        game_context: &mut GameContext,
        parent_layout: &GridLayout,
    ) {
        let layout = self.relative_layout(parent_layout);
        let Some(_) = layout.cell_for_mouse(game_context.mouse_context.position) else {
            self.hovered = false;
            self.clicked = false;
            self.click_state = ReadyState::Ready;
            return;
        };

        if let ReadyState::Ready = self.click_state
            && !self.waiting_for_release
        {
            self.hovered = true;
        } else {
            self.hovered = false;
        }
        if let Some(sprite_info) = self.highlight.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some(sprite_info) = self.bg.as_mut() {
            sprite_info.advance(ticks);
        }

        if !game_context.mouse_context.left_clicked {
            self.waiting_for_release = false;
        }
        if let ReadyState::Ready = self.click_state
            && game_context.mouse_context.left_clicked
            && !self.waiting_for_release
        {
            self.clicked = true;
            self.waiting_for_release = true;
            self.click_state = ReadyState::Cooldown {
                ticks_waited: 0,
                wait_for: self.cooldown_ticks,
            };
        } else {
            self.clicked = false;
        }
        self.click_state = match self.click_state {
            ReadyState::Ready => self.click_state,
            ReadyState::Cooldown {
                wait_for,
                ticks_waited,
            } => {
                let ticks_waited = ticks_waited.saturating_add(ticks);
                if ticks_waited >= wait_for && !self.waiting_for_release {
                    ReadyState::Ready
                } else {
                    ReadyState::Cooldown {
                        wait_for,
                        ticks_waited,
                    }
                }
            }
        };
    }

    pub fn draw(&self, game_context: &mut GameContext, parent_layout: &GridLayout) {
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

        if self.hovered
            && let Some(highlight) = self.highlight.as_ref()
        {
            let src = highlight.get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: self.texture_id,
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
                    font,
                    renderer.as_mut(),
                    FontAlign::Center,
                );
            }
        }
    }
}
