use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::constants::{
    FontId, MUSIC_ID_PACHEBAL, SFX_ID_BLIP, TEXTURE_ID_LEEKSHEET, TEXTURE_ID_PORTRAIT,
    TEXTURE_ID_STORY_BG, TEXTURE_ID_TITLE_BG, sprite_info_highlight,
    sprite_info_planning_layout_bg, sprite_info_portrait,
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
    vn: Option<VisualNovelMachine>,
    current_choices: Option<Vec<(Choice, Button)>>,
    // Placement locations (Grid unit space, used for converting to screen coordinates )
    placement_text: Rect,
    text_speed: LineSpeed,
    placement_left_portrait: Rect,
    placement_right_portrait: Rect,
    placement_center_portrait: Rect,
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

    fn choice_btn_layout(buttons: usize, game_context: &GameContext) -> GridLayout {
        let layout = StoryScene::layout(game_context);
        let area = Rect {
            x: 2,
            y: 6,
            width: 12,
            height: 3,
        }
        .relative_rect_to_layout(&layout);
        GridLayout {
            area,
            rows: buttons,
            columns: 1,
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

        // TODO: Hardcoded sprites are just because of hardcoded
        // scenes, we need a proper way to convert a Character to
        // a sprite and all that. Probably need to deal with .aes files
        // and frame tagging to do something like that. For now, this is
        // okay because we're just implementing the ops bits and testing
        if let Some(character) = vn.slot_left.as_ref() {
            let dst = self
                .placement_left_portrait
                .relative_rect_to_layout(&layout);
            let src = SpriteInfo {
                width: 417,
                height: 402,
                ..sprite_info_portrait()
            }
            .get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: character.texture_id,
                source: src,
                destination: dst,
            });
            if let Some(PortraitPosition::Left) = vn.focused {
                let src = sprite_info_highlight().get_rect();
                renderer.send_command(RenderCommand::DrawRect {
                    texture_id: TEXTURE_ID_LEEKSHEET,
                    source: src,
                    destination: dst,
                });
            }
        }
        if let Some(character) = vn.slot_center.as_ref() {
            let dst = self
                .placement_center_portrait
                .relative_rect_to_layout(&layout);
            let src = SpriteInfo {
                start_x: 906,
                width: 1143 - 906,
                height: 402,
                ..sprite_info_portrait()
            }
            .get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: character.texture_id,
                source: src,
                destination: dst,
            });
            if let Some(PortraitPosition::Center) = vn.focused {
                let src = sprite_info_highlight().get_rect();
                renderer.send_command(RenderCommand::DrawRect {
                    texture_id: TEXTURE_ID_LEEKSHEET,
                    source: src,
                    destination: dst,
                });
            }
        }
        if let Some(character) = vn.slot_right.as_ref() {
            let dst = self
                .placement_right_portrait
                .relative_rect_to_layout(&layout);
            let src = SpriteInfo {
                start_x: 462,
                width: 747 - 462,
                height: 402,
                ..sprite_info_portrait()
            }
            .get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: character.texture_id,
                source: src,
                destination: dst,
            });
            if let Some(PortraitPosition::Right) = vn.focused {
                let src = sprite_info_highlight().get_rect();
                renderer.send_command(RenderCommand::DrawRect {
                    texture_id: TEXTURE_ID_LEEKSHEET,
                    source: src,
                    destination: dst,
                });
            }
        }

        if let Some(choices) = &self.current_choices {
            let layout = StoryScene::choice_btn_layout(choices.len(), game_context);
            for (_, btn) in choices {
                btn.draw(game_context, &layout);
            }
        }
    }
    fn set_text_box_to_next_say_if_empty(&mut self, game_context: &mut GameContext) {
        let layout = StoryScene::layout(game_context);
        if let (None, Some(font_library)) = (&self.text_box, game_context.font_library.as_mut()) {
            let text_box_bounds = text_box_bounds_on_grid(&layout, &self.placement_text);
            let maybe_font: Option<&LoadedFont> = font_library.get(STORY_FONT);

            if let Some(text) = self.vn.as_mut().and_then(|vn| vn.next_say()) {
                self.text_box = maybe_font.map(|loaded_font| {
                    TextBox::new(text_box_bounds, text, loaded_font, self.text_speed)
                });
            }
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
            next_btn: Some({
                Button::new(
                    "Next".to_owned(),
                    Rect {
                        x: 14,
                        y: 7,
                        width: 1,
                        height: 1,
                    },
                    BUTTON_FONT,
                )
            }),
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
            current_choices: None,
            placement_left_portrait: Rect {
                x: 2,
                y: 0,
                width: 4,
                height: 6,
            },
            placement_center_portrait: Rect {
                x: 6,
                y: 0,
                width: 4,
                height: 6,
            },
            placement_right_portrait: Rect {
                x: 10,
                y: 0,
                width: 4,
                height: 6,
            },
        }
    }
}

impl Scene for StoryScene {
    fn init(&mut self, game_context: &mut GameContext) {
        self.fade = TransitionScene::new_fade_in().into();

        if let Some(ref mut asset_loader) = game_context.asset_loader {
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_STORY_BG);
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_LEEKSHEET);
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_PORTRAIT);
        }

        if let Some(ref mut audio) = game_context.audio {
            let _ = audio.load_sfx(SFX_ID_BLIP);
            let _ = audio.load_music(MUSIC_ID_PACHEBAL);
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
            NovelOps::PlaySfx(SFX_ID_BLIP),
            NovelOps::NewCharacterState(
                Character{
                    name: "miku".to_owned(),
                    state: CharacterState::Mad,
                    texture_id: TEXTURE_ID_PORTRAIT,
                }
            ),
            NovelOps::Label("repeat".to_owned()),
            NovelOps::Focus(PortraitPosition::Left),
            NovelOps::Say("Finally... it is time.".to_owned()),
            NovelOps::PlaySfx(SFX_ID_BLIP),
            NovelOps::Show(
                Character {
                    name: "rin".to_owned(),
                    state: CharacterState::Idle,
                    texture_id: TEXTURE_ID_PORTRAIT,
                },
                PortraitPosition::Right,
            ),
            NovelOps::Focus(PortraitPosition::Right),
            NovelOps::Choices(vec![
                Choice { label: "repeat".to_owned(), text: "What was that?".to_owned() },
                Choice { label: "next".to_owned(), text: "Indeed, we shall have our revenge!".to_owned() },
            ]),
            NovelOps::Label("next".to_owned()),
            NovelOps::Hide(PortraitPosition::Right),
            NovelOps::Focus(PortraitPosition::Left),
            NovelOps::Say("\nIndeed my friend, it is finally time to begin\nour long awaited adventure...\n\nIn tactics!".to_owned()),
            NovelOps::PlaySfx(SFX_ID_BLIP),
            NovelOps::Hide(PortraitPosition::Left),
            NovelOps::Background(TEXTURE_ID_TITLE_BG),
            NovelOps::PlayMusic(MUSIC_ID_PACHEBAL),
            NovelOps::Say(
                concat!(
                    "Hello there this is a very long string of text I'm hoping will be long enough to be interesting ",
                    "or something I guess though who knows lalalalalal yay this is so fun. ",
                    "There are no line breaks in the text at all, so you could load this from a file or similar and then ",
                    "it would happily display on the screen with no issues at all. Isn't that just lovely. Thought this ",
                    "also means that if you want a newline that's explicit, I suppose you'd need to start a new page or something",
                    "... how bothersome I suppose."
                ).to_string(),
            ),
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

        if let Some(text_box) = self.text_box.as_mut() {
            text_box.text.update(ticks);
        }

        if let Some(btn) = self.next_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked {
                // TODO: this is just temporary while we work on the vn
                // we likely need some sort of indicator on if the operation
                // is waiting for us to continue or not at some point.
                if let Some(vn) = self.vn.as_mut() {
                    vn.advance();
                }
                if let Some(b) = self.text_box.as_mut() {
                    if b.text.is_visible() {
                        b.next_page();
                        b.set_speed(self.text_speed);

                        // If we just advanced but the text is still visible that means
                        // we've hit the end of a Say operation and should request the next one.
                        if b.text.is_visible() {
                            self.text_box = None;
                        }
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
                self.text_speed = self.text_speed.cycle_speeds();
                if let Some(text_box) = self.text_box.as_mut() {
                    text_box.set_speed(self.text_speed)
                };
                btn.text = self.text_speed.to_string();
                btn.clicked = false;
                game_context.mouse_context.consume_left_click();
            }
        };

        // Ensure that if we should be showing a textbox, we show one.
        // if there is one already displaying it won't be changed.
        self.set_text_box_to_next_say_if_empty(game_context);

        // Chaining pattern matches with an && inbetween is gross
        // looking. Kindly go do something more useful plz clippy.
        #[allow(clippy::collapsible_if)]
        if let Some(vn) = self.vn.as_mut() {
            if let Some(audio) = game_context.audio.as_mut() {
                if let Some(sfx_id) = vn.next_sfx() {
                    let _ = audio.play_sfx(sfx_id);
                };
                if let Some(music_id) = vn.music.take() {
                    let _ = audio.play_music(music_id);
                };
            }
            if let Some(choices) = vn.active_choice.take() {
                let new_choices = choices
                    .into_iter()
                    .enumerate()
                    .map(|(idx, choice)| {
                        let text = choice.text.clone();
                        let b = Button::new(
                            text,
                            Rect {
                                x: 0,
                                y: idx as isize,
                                width: 1,
                                height: 1,
                            },
                            BUTTON_FONT,
                        );
                        (choice, b)
                    })
                    .collect();
                self.current_choices = Some(new_choices);
            }
        }

        if let Some(choices) = self.current_choices.as_mut() {
            let layout = StoryScene::choice_btn_layout(choices.len(), game_context);
            let mut clicked = false;
            for (choice, btn) in choices {
                btn.update(ticks, game_context, &layout);
                if btn.clicked {
                    if let Some(vn) = self.vn.as_mut() {
                        clicked = true;
                        vn.jump(&choice.label);
                    }
                    btn.clicked = false;
                    game_context.mouse_context.consume_left_click();
                }
            }
            if clicked {
                self.current_choices = None;
            }
        }
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        let layout = StoryScene::layout(game_context);
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
