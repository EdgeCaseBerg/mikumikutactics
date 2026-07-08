use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::constants::{
    FontId, SFX_ID_BLIP, TEXTURE_ID_LEEKSHEET, TEXTURE_ID_STORY_BG, sprite_info_planning_layout_bg,
};
use crate::font::FONTSHEET_LAYOUT;
use crate::font::LoadedFont;
use crate::font::draw_text_anchored;
use crate::font::FontAlign;
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::scene::story::text::LineSpeed;
use crate::scene::story::text::TextBox;
use crate::scene::transitions::{Transition, TransitionScene};

pub struct StoryScene {
    bg: SpriteInfo,
    fade: Option<TransitionScene>,
    text_box: Option<TextBox>,
    // Placement locations (Grid unit space, used for converting to screen coordinates )
    placement_text: Rect,
    // placement_left_portrait: Rect,
    // placement_right_portrait: Rect,
    // placement_center_portrait: Rect,
}

impl StoryScene {
    fn layout(game_context: &GameContext) -> GridLayout {
        let (screen_width, screen_height) = game_context.screen_size;
        GridLayout {
            area: Rect::new(0, 0, screen_width as isize, screen_height as isize),
            rows: 9,
            columns: 16,
            cell_gap: 0,
        }
    }
}

impl Default for StoryScene {
    fn default() -> StoryScene {
        StoryScene {
            bg: sprite_info_planning_layout_bg(),
            fade: None,
            text_box: None,
            placement_text: Rect {
                x: 2,
                y: 6,
                width: 12,
                height: 3,
            },
        }
    }
}

fn text_box_bounds_on_grid(layout: &GridLayout, grid_bounds: &Rect) -> Rect {
    let top_left = layout.cell_rect(grid_bounds.y as usize, grid_bounds.x as usize);
    let (cw, ch) = layout.cell_size();
    Rect {
        x: top_left.x,
        y: top_left.y,
        width: cw * grid_bounds.width,
        height: ch * grid_bounds.height,
    }
}

impl Scene for StoryScene {
    fn init(&mut self, game_context: &mut GameContext) {
        self.fade = TransitionScene::new_fade_in().into();

        if let Some(ref mut asset_loader) = game_context.asset_loader {
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_STORY_BG);
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_LEEKSHEET);
        }

        if let Some(ref mut audio) = game_context.audio {
            let _ = audio.load_sfx(SFX_ID_BLIP);
        }

        if let (Some(font_library), Some(asset_loader)) = (
            game_context.font_library.as_mut(),
            game_context.asset_loader.as_mut(),
        ) {
            let _ = font_library.ensure(FontId::BoldPixels, asset_loader.as_mut());
        }
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) {
        if let Some(mut fade) = self.fade.take() {
            fade.update(ticks, game_context);
            let fade = match (fade.is_done(), &fade.transition) {
                (true, Transition::FadeIn { .. }) => None,
                _ => Some(fade), // When we do a scene transition out, update here
            };
            self.fade = fade;
        }

        let layout = StoryScene::layout(&game_context);
        // Update is where we'd deal with our VN Machine and get the next set of pages or
        // similar, for now, hard coded it for testing.
        if let (None, Some(font_library)) = (&self.text_box, game_context.font_library.as_mut()) {
            let text_box_bounds = text_box_bounds_on_grid(&layout, &self.placement_text);
            let maybe_font: Option<&LoadedFont> = font_library.get(FontId::BoldPixels);

            self.text_box = maybe_font.map(|loaded_font| {
                TextBox::new(
                    text_box_bounds,
                    concat!(
                        "Hello there this is a very long string of text I'm hoping will be long enough to be interesting ",
                        "or something I guess though who knows lalalalalal yay this is so fun. ",
                        "There are no line breaks in the text at all, so you could load this from a file or similar and then ",
                        "it would happily display on the screen with no issues at all. Isn't that just lovely. Thought this ",
                        "also means that if you want a newline that's explicit, I suppose you'd need to start a new page or something",
                        "... how bothersome I suppose."
                    ).to_string(),
                    loaded_font,
                    LineSpeed::Faster,
                )
            });
        }
        self.text_box
            .as_mut()
            .map(|text_box| text_box.text.update(ticks));
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        let layout = StoryScene::layout(&game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };

        let src = self.bg.get_rect();
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: TEXTURE_ID_STORY_BG,
            source: src,
            destination: Rect::new(0, 0, layout.area.width, layout.area.height),
        });

        let maybe_font: Option<&LoadedFont> = game_context
            .font_library
            .as_mut()
            .map(|font_library| font_library.get(FontId::BoldPixels))
            .flatten();

        if let (Some(font), Some(text_box)) = (maybe_font, &self.text_box) {
            let anchor = text_box.bounds;
            for (line_idx, line) in text_box.text.lines().iter().enumerate() {
                let leading = text_box.leading;
                let offset_y = leading as usize
                    + line_idx * (leading + text_box.line_height) as usize
                    + leading as usize * line_idx;
                let anchor = Rect {
                    x: anchor.x,
                    y: anchor.y as isize + offset_y as isize,
                    width: anchor.width,
                    height: text_box.line_height as isize,
                };
                draw_text_anchored(
                    &line,
                    &anchor,
                    &font,
                    renderer.as_mut(),
                    FontAlign::Left,
                );
            }
        }
        self.fade.as_mut().map(|f| f.draw(game_context));
    }
}

#[cfg(test)]
mod dialogue_scene_tests {
    // use super::*;
    // use crate::backend_dummy::FakeRenderer;
    // use crate::backend_dummy::click_button_with_left_mouse;

    #[test]
    fn we_should_test_interactions() {
        assert!(true);
    }
}
