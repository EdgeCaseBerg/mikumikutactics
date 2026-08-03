use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::asset_loader::AssetCatalog;
use crate::constants::{
    FontId, MUSIC_ID_PACHEBAL, SFX_BLIP, SfxId, TEXTURE_ID_LEEKSHEET, sprite_info_highlight,
    sprite_info_planning_layout_bg,
};
use crate::font::LoadedFont;
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::{RenderCommand, Renderer};
use crate::scene::button::Button;
use crate::scene::story::text::LineSpeed;
use crate::scene::story::text::TextBox;
use crate::scene::story::text::text_box_bounds_on_grid;
use crate::scene::story::visual_novel::{
    Character, Choice, NovelOps, PortraitPosition, VisualNovelMachine, parse_scene,
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
    // Cached asset ids
    ids: Option<StorySceneIds>,
}

struct StorySceneIds {
    btn_sfx_id: SfxId,
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
    fn should_wait_for_user_input(&self) -> bool {
        let has_choices = self.current_choices.as_ref().is_some_and(|v| !v.is_empty());
        let has_text = self.text_box.is_some();
        has_choices || has_text
    }
    fn draw_character_slot(
        slot: Option<&Character>,
        placement: Rect,
        renderer: &mut dyn Renderer,
        catalog: &AssetCatalog,
    ) {
        let Some(character) = slot.as_ref() else {
            return;
        };
        let Some(src) = catalog
            .get_sprite_in_state(&character.name, character.state.as_str())
            .map(|s| s.get_rect())
        else {
            return;
        };
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: character.texture_id,
            source: src,
            destination: placement,
        });
    }
    fn draw_portrait_outline(
        &self,
        at_position: PortraitPosition,
        renderer: &mut dyn Renderer,
        layout: &GridLayout,
    ) {
        let Some(vn) = self.vn.as_ref() else {
            return;
        };
        if vn.focused.as_ref().is_none_or(|pp| *pp != at_position) {
            return;
        }
        let dst = match at_position {
            PortraitPosition::Left => self.placement_left_portrait.relative_rect_to_layout(layout),
            PortraitPosition::Center => self
                .placement_center_portrait
                .relative_rect_to_layout(layout),
            PortraitPosition::Right => self
                .placement_right_portrait
                .relative_rect_to_layout(layout),
        };
        let src = sprite_info_highlight().get_rect();
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: TEXTURE_ID_LEEKSHEET,
            source: src,
            destination: dst,
        });
    }
    fn draw_vn_scene(&self, vn: &VisualNovelMachine, game_context: &mut GameContext) {
        let layout = StoryScene::layout(game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };
        let Some(ref asset_loader) = game_context.asset_loader else {
            return;
        };
        let catalog = asset_loader.get_catalog();

        if let Some(background_id) = vn.background {
            let src = self.bg.get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: background_id,
                source: src,
                destination: Rect::new(0, 0, layout.area.width, layout.area.height),
            });
        }
        Self::draw_character_slot(
            vn.slot_left.as_ref(),
            self.placement_left_portrait
                .relative_rect_to_layout(&layout),
            renderer.as_mut(),
            &catalog,
        );
        if vn.slot_left.is_some() {
            self.draw_portrait_outline(PortraitPosition::Left, renderer.as_mut(), &layout);
        }
        Self::draw_character_slot(
            vn.slot_center.as_ref(),
            self.placement_center_portrait
                .relative_rect_to_layout(&layout),
            renderer.as_mut(),
            &catalog,
        );
        if vn.slot_center.is_some() {
            self.draw_portrait_outline(PortraitPosition::Center, renderer.as_mut(), &layout);
        }
        Self::draw_character_slot(
            vn.slot_right.as_ref(),
            self.placement_right_portrait
                .relative_rect_to_layout(&layout),
            renderer.as_mut(),
            &catalog,
        );
        if vn.slot_right.is_some() {
            self.draw_portrait_outline(PortraitPosition::Right, renderer.as_mut(), &layout);
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
    fn next_btn() -> Button {
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
    }
    fn speed_btn(speed: LineSpeed) -> Button {
        Button::new(
            speed.to_string(),
            Rect {
                x: 14,
                y: 6,
                width: 1,
                height: 1,
            },
            BUTTON_FONT,
        )
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
            next_btn: Some(StoryScene::next_btn()),
            speed_btn: Some(StoryScene::speed_btn(default_speed)),
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
            ids: None,
        }
    }
}

fn ensure_ops_loaded(ops: &[NovelOps], game_context: &mut GameContext) {
    for op in ops {
        if let Some(id) = op.as_texture_id() {
            let Some(ref mut asset_loader) = game_context.asset_loader else {
                return;
            };
            asset_loader.ensure_texture_spritesheet_loaded(id);
        }
        if let NovelOps::PlaySfx(sfx_id) = op {
            if let Some(ref mut audio) = game_context.audio {
                let _ = audio.load_sfx(*sfx_id);
            }
        } else if let NovelOps::PlayMusic(music_id) = op {
            if let Some(ref mut audio) = game_context.audio {
                let _ = audio.load_music(*music_id);
            }
        } else {
            continue;
        }
    }
}

fn test_scene_ops(catalog: &AssetCatalog) -> Vec<NovelOps> {
    let vn = parse_scene(&include_str!("../../assets/scenes/noaudio.scene"), catalog);
    vn.unwrap().program
}

impl Scene for StoryScene {
    fn init(&mut self, game_context: &mut GameContext) {
        self.fade = TransitionScene::new_fade_in().into();

        if let Some(ref mut asset_loader) = game_context.asset_loader {
            // TODO: this now breaks since its outside of the asset pipeline
            // and the leeksheet id belongs to something loaded elsewhere.
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_LEEKSHEET);
        }

        if let Some(ref mut audio) = game_context.audio {
            let _ = audio.load_music(MUSIC_ID_PACHEBAL);
        }

        if let (Some(audio), Some(asset_loader)) = (
            game_context.audio.as_mut(),
            game_context.asset_loader.as_mut(),
        ) {
            let catalog = asset_loader.get_catalog();
            if let Some(blip) = catalog.get_sfx_id_for_name(SFX_BLIP) {
                let _ = audio.load_sfx(blip);
                let ids = StorySceneIds { btn_sfx_id: blip };
                self.ids = ids.into();
            }
        };

        if let (Some(font_library), Some(asset_loader)) = (
            game_context.font_library.as_mut(),
            game_context.asset_loader.as_mut(),
        ) {
            let _ = font_library.ensure(STORY_FONT, asset_loader.as_mut());
            let _ = font_library.ensure(BUTTON_FONT, asset_loader.as_mut());
        }

        // Temporarily create a story scene from scratch
        if let Some(ref mut asset_loader) = game_context.asset_loader {
            let ops = test_scene_ops(&asset_loader.get_catalog());
            ensure_ops_loaded(&ops, game_context);
            self.vn = Some(ops.into());
        };
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
                if let Some(audio) = game_context.audio.as_mut() {
                    if let Some(id) = self.ids.as_ref().map(|ids| ids.btn_sfx_id) {
                        let _ = audio.play_sfx(id);
                    }
                }
                if let Some(b) = self.text_box.as_mut() {
                    if b.text.is_visible() {
                        b.next_page();
                        b.set_speed(self.text_speed);

                        // If we just advanced but the text is still visible that means
                        // we've hit the end of a Say operation and should request the next one.
                        if b.text.is_visible() {
                            // requesting the next say is done automatically below by no longer
                            // waiting on user input since the text box is now none and the
                            // call to should_wait_for_user_input will see that we no longer have
                            // a live text box and will advance the vn to the next say or choice
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

        // auto advance the vn program until it populates something for
        // that requires input from the user to dismiss or continue with
        if !self.should_wait_for_user_input()
            && let Some(vn) = self.vn.as_mut()
        {
            self.next_btn = None;
            self.speed_btn = None;
            vn.advance();
        } else {
            if self.next_btn.is_none() && self.text_box.is_some() {
                self.next_btn = Some(StoryScene::next_btn());
            }
            if self.speed_btn.is_none() && self.text_box.is_some() {
                self.speed_btn = Some(StoryScene::speed_btn(self.text_speed));
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
