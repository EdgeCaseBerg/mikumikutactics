use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::constants::{
    SFX_ID_BLIP, SFX_ID_MEME, TEXTURE_ID_FONTSHEET, TEXTURE_ID_LEEKSHEET, TEXTURE_ID_STORY_BG,
    sprite_info_planning_layout_bg,
};
use crate::font::BitmapFontMetrics;
use crate::font::FONTSHEET_LAYOUT;
use crate::font::FontMetrics;
use crate::font::get_rects_for_str;
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::Color;
use crate::renderer::RenderCommand;
use crate::scene::button::Button;
use crate::scene::story::text::LineSpeed;
use crate::scene::story::text::TextBox;
use crate::scene::transitions::{Transition, TransitionScene};
use crate::{ReadyState, advance_ready_state}; // if this goes well, onsider font renderer methods to avoid importing this

pub struct StoryScene {
    bg: SpriteInfo,
    fade: Option<TransitionScene>,
    // font: Box<dyn FontMetrics>,
    text_box_loaded: bool,
    text_box: TextBox,
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
            // font: Box::new(BitmapFontMetrics {
            //     layout: FONTSHEET_LAYOUT,
            // }),
            text_box_loaded: false,
            text_box: TextBox::new(
                Rect {
                    x: 100,
                    y: 100,
                    width: 10000, // huge values to avoid math that we'll redo during .init
                    height: 10000,
                },
                concat!(
                    "Hello there this is a very long string of text I'm hoping will be long enough to be interesting ",
                    "or something I guess though who knows lalalalalal yay this is so fun. ",
                    "There are no line breaks in the text at all, so you could load this from a file or similar and then ",
                    "it would happily display on the screen with no issues at all. Isn't that just lovely. Thought this ",
                    "also means that if you want a newline that's explicit, I suppose you'd need to start a new page or something",
                    "... how bothersome I suppose."
                ).to_string(),
                 
                &BitmapFontMetrics {
                    layout: FONTSHEET_LAYOUT,
                },
                LineSpeed::Normal,
            ),
        }
    }
}

impl Scene for StoryScene {
    fn init(&mut self, game_context: &mut GameContext) {
        self.fade = TransitionScene::new_fade_in().into();

        if let Some(ref mut asset_loader) = game_context.asset_loader {
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_STORY_BG);
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_LEEKSHEET);
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_FONTSHEET);
        }

        if let Some(ref mut audio) = game_context.audio {
            let _ = audio.load_sfx(SFX_ID_BLIP);
        }

        // Now that we have the game context and know how large the screen is, resize the
        // text layout so that it aligns with the scenes grid and doesnt have to use abs position
        let layout = StoryScene::layout(&game_context);
        if !self.text_box_loaded {
            self.text_box_loaded = true;
            let top_left = layout.cell_rect(6, 2);
            let (cw, ch) = layout.cell_size();
            self.text_box = TextBox::new(
                Rect {
                    x: top_left.x,
                    y: top_left.y,
                    width: cw * 10,
                    height: ch * 3,
                },
                self.text_box.text.raw_text(),
                &BitmapFontMetrics {
                    layout: FONTSHEET_LAYOUT,
                },
                LineSpeed::Normal,
            );
            eprintln!("{:?}", self.text_box);
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
        self.text_box.text.update(ticks);
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

        // TODO: make a text renderer we can shuffle this off to
        let anchor = self.text_box.bounds;
        for (line_idx, line) in self.text_box.text.lines().iter().enumerate() {
            let glyphs = get_rects_for_str(&line);
            for (c, src) in glyphs.iter().enumerate() {
                let glyph_display_width = src.width;
                let glyph_display_height = self.text_box.line_height;
                let leading = self.text_box.leading;
                let offset_y = leading as usize
                    + line_idx * (leading + self.text_box.line_height) as usize
                    + leading as usize * line_idx;
                let start_offset = anchor.x;
                let cell = Rect {
                    x: start_offset + c as isize * glyph_display_width as isize,
                    y: anchor.y as isize + offset_y as isize,
                    width: glyph_display_width,
                    height: glyph_display_height as isize,
                };
                renderer.send_command(RenderCommand::DrawRect {
                    texture_id: TEXTURE_ID_FONTSHEET,
                    source: *src,
                    destination: cell,
                });
            }
        }
        self.fade.as_mut().map(|f| f.draw(game_context));
    }
}

#[cfg(test)]
mod dialogue_scene_tests {
    use super::*;
    use crate::backend_dummy::FakeRenderer;
    use crate::backend_dummy::click_button_with_left_mouse;

    #[test]
    fn we_should_test_interactions() {
        assert!(true);
    }
}
