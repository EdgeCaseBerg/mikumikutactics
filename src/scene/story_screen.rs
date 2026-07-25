use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::constants::{
    FontId, SFX_ID_BLIP, TEXTURE_ID_LEEKSHEET, TEXTURE_ID_PORTRAIT, TEXTURE_ID_STORY_BG,
    TEXTURE_ID_TITLE_BG, sprite_info_planning_layout_bg,
};
use crate::font::LoadedFont;
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::scene::button::Button;
use crate::scene::story::text::LineSpeed;
use crate::scene::story::text::TextBox;
use crate::scene::story::text::text_box_bounds_on_grid;
use crate::scene::story::visual_novel::{
    Character, CharacterState, Choice, NovelOps, PortraitPosition, VisualNovelMachine,
};
use crate::scene::transitions::{Transition, TransitionScene};

const STORY_FONT: FontId = FontId::BoldPixelScaled { pixel_size: 24 };
const BUTTON_FONT: FontId = FontId::BoldPixels;

pub struct StoryScene {
    bg: SpriteInfo,
    fade: Option<TransitionScene>,
    text_box: Option<TextBox>,
    next_btn: Option<Button>,
    speed_btn: Option<Button>,
    // Placement locations (Grid unit space, used for converting to screen coordinates )
    placement_text: Rect,
    text_speed: LineSpeed,
    vn: Option<VisualNovelMachine>,
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

    fn draw_vn_scene(&self, vn: &VisualNovelMachine, game_context: &mut GameContext) {
        let layout = StoryScene::layout(game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };

        if let Some(background_id) = vn.background {
            let src = self.bg.get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: background_id,
                source: src,
                destination: Rect::new(0, 0, layout.area.width, layout.area.height),
            });
        }
    }
}

impl Default for StoryScene {
    fn default() -> StoryScene {
        let default_speed = LineSpeed::Normal;
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
            next_btn: None,
            speed_btn: Some(Button::new(
                default_speed.to_string(),
                Rect {
                    x: 14,
                    y: 6,
                    width: 1,
                    height: 1,
                },
                BUTTON_FONT,
            )),
            text_speed: default_speed,
            vn: None,
        }
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
            let _ = font_library.ensure(STORY_FONT, asset_loader.as_mut());
            let _ = font_library.ensure(BUTTON_FONT, asset_loader.as_mut());
        }

        // Temporarily create a story scene from scratch
        let ops = vec![
            NovelOps::Background(TEXTURE_ID_STORY_BG),
            NovelOps::Show(
                Character{
                    name: "miku".to_owned(),
                    state: CharacterState::Idle,
                    texture_id: TEXTURE_ID_PORTRAIT,
                }, PortraitPosition::Left),
            NovelOps::NewCharacterState(
                Character{
                    name: "miku".to_owned(),
                    state: CharacterState::Mad,
                    texture_id: TEXTURE_ID_PORTRAIT,
                }
            ),
            NovelOps::Label("repeat".to_owned()),
            NovelOps::Say("Finally... it is time.".to_owned()),
            NovelOps::Show(
                Character {
                    name: "rin".to_owned(),
                    state: CharacterState::Idle,
                    texture_id: TEXTURE_ID_PORTRAIT,
                },
                PortraitPosition::Right,
            ),
            NovelOps::Choices(vec![
                Choice { label: "repeat".to_owned(), text: "What was that?".to_owned() },
                Choice { label: "next".to_owned(), text: "Indeed, we shall have our revenge!".to_owned() },
            ]),
            NovelOps::Label("next".to_owned()),
            NovelOps::Focus(PortraitPosition::Left),
            NovelOps::Hide(PortraitPosition::Right),
            NovelOps::Say("\nIndeed my friend, it is finally time to begin\nour long awaited adventure...\n\nIn tactics!".to_owned()),
            NovelOps::Hide(PortraitPosition::Left),
            NovelOps::Background(TEXTURE_ID_TITLE_BG),
        ];
        self.vn = Some(ops.into());
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

        let layout = StoryScene::layout(game_context);
        // Update is where we'd deal with our VN Machine and get the next set of pages or
        // similar, for now, hard coded it for testing.
        if let (None, Some(font_library)) = (&self.text_box, game_context.font_library.as_mut()) {
            let text_box_bounds = text_box_bounds_on_grid(&layout, &self.placement_text);
            let maybe_font: Option<&LoadedFont> = font_library.get(STORY_FONT);

            self.text_box = maybe_font.map(|loaded_font| {
                // Temp for now, but lets wire in the next button as well for testing purposes?
                self.next_btn = Some(
                    {
                        Button::new("Next".to_owned(), Rect {
                            x: 14,
                            y: 7,
                            width: 1,
                            height: 1,
                        }, BUTTON_FONT)
                    }
                );

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
                    self.text_speed,
                )
            });
        }

        if let Some(text_box) = self.text_box.as_mut() {
            text_box.text.update(ticks);
        }

        if let Some(btn) = self.next_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked {
                if let Some(b) = self.text_box.as_mut() {
                    if b.text.is_visible() {
                        b.next_page();
                        b.set_speed(self.text_speed);
                    } else {
                        b.set_speed(LineSpeed::Instant);
                    }
                };
                btn.clicked = false;
                game_context.mouse_context.consume_left_click();
            }
        };
        if let Some(btn) = self.speed_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked {
                // TODO: this is just temporary while we work on the vn
                if let Some(vn) = self.vn.as_mut() {
                    vn.advance();
                }
                self.text_speed = self.text_speed.cycle_speeds();
                if let Some(text_box) = self.text_box.as_mut() {
                    text_box.set_speed(self.text_speed)
                };
                btn.text = self.text_speed.to_string();
                btn.clicked = false;
                game_context.mouse_context.consume_left_click();
            }
        };
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        let layout = StoryScene::layout(game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };

        if let Some(vn) = self.vn.as_ref() {
            self.draw_vn_scene(vn, game_context);
        }

        if let Some(text_box) = self.text_box.as_mut() {
            text_box.draw(game_context);
        }
        if let Some(btn) = self.next_btn.as_mut() {
            btn.draw(game_context, &layout);
        };
        if let Some(btn) = self.speed_btn.as_mut() {
            btn.draw(game_context, &layout);
        };
        if let Some(fade) = self.fade.as_mut() {
            fade.draw(game_context);
        };
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
