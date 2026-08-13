use crate::GameResult;
use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::asset_loader::AssetCatalog;
use crate::asset_loader::ensure_texture_by_name;
use crate::constants::{
    FontId, ProgramCounter, SFX_BLIP, SceneId, SfxId, TEXTURE_PLANNING_LAYOUT_BG, TEXTURE_UI,
    TEXTURE_UI_FOCUS_TAG, TextureId, VN_QUIT_FLAG,
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
    bg: Option<SpriteInfo>,
    highlight: Option<SpriteInfo>,
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
    ui_textures_id: TextureId,
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
        let Some(ids) = self.ids.as_ref() else {
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
        if let Some(sprite_info) = self.highlight.as_ref() {
            let src = sprite_info.get_rect();
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: ids.ui_textures_id,
                source: src,
                destination: dst,
            });
        }
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

        if let (Some(background_id), Some(sprite_info)) = (vn.background, self.bg.as_ref()) {
            let src = sprite_info.get_rect();
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
    fn next_btn(&self) -> Option<Button> {
        let ids = self.ids.as_ref()?;
        // when we need to grab the sprite info: catalog.get_sprite_in_state(TEXTURE_UI, "button")
        let b = Button::new(
            "Next".to_owned(),
            Rect {
                x: 14,
                y: 7,
                width: 1,
                height: 1,
            },
            BUTTON_FONT,
            ids.ui_textures_id,
        );
        Some(b)
    }
    fn speed_btn(&self) -> Option<Button> {
        let ids = self.ids.as_ref()?;
        let b = Button::new(
            self.text_speed.to_string(),
            Rect {
                x: 14,
                y: 6,
                width: 1,
                height: 1,
            },
            BUTTON_FONT,
            ids.ui_textures_id,
        );
        Some(b)
    }
    fn load_story_scene_contents(
        &mut self,
        scene_id: SceneId,
        pc: ProgramCounter,
        game_context: &mut GameContext,
    ) -> GameResult<()> {
        let Some(asset_loader) = game_context.asset_loader.as_mut() else {
            return Err("cannot load scene contents if asset loader is not configured".into());
        };
        let catalog = asset_loader.get_catalog();
        let scene_path = catalog.get_scene_path(scene_id)?;
        let raw_scene = asset_loader.load_story_scene_contents(scene_path)?;
        let mut vn = parse_scene(&raw_scene, catalog.as_ref())?;
        ensure_ops_loaded(&vn.program, game_context);
        // TODO: restore state safely (validate bounds, fast forward the script to determine what the current state of the system should be, etc)
        // we'll likely need to have some kind of thing that also sets the story_screen state fields based on what was loaded as well, maybe. it
        // might be enough to just run through and advance the pc and let the various queues fill up (and ensure we DRAIN the audio ones where appropriate)
        vn.program_counter = pc.0;
        if let Some(existing_vn) = self.vn.take() {
            vn.flags.extend(existing_vn.flags);
        }
        self.vn = Some(vn);
        Ok(())
    }
}

impl Default for StoryScene {
    fn default() -> StoryScene {
        let default_speed = LineSpeed::Normal;
        StoryScene {
            bg: None,
            highlight: None,
            fade: None,
            text_box: None,
            placement_text: Rect {
                x: 2,
                y: 6,
                width: 12,
                height: 3,
            },
            next_btn: None,
            speed_btn: None,
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

impl Scene for StoryScene {
    fn init(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        self.fade = TransitionScene::new_fade_in().into();
        let Some(audio) = game_context.audio.as_mut() else {
            return Err("could not init scene, audio not configured".into());
        };
        let Some(asset_loader) = game_context.asset_loader.as_mut() else {
            return Err("could not init scene, asset loader not set".into());
        };
        let catalog = asset_loader.get_catalog();

        let _ = ensure_texture_by_name(TEXTURE_PLANNING_LAYOUT_BG, asset_loader.as_mut())?;
        self.bg = catalog.get_sprite(TEXTURE_PLANNING_LAYOUT_BG);

        let ui_textures_id = ensure_texture_by_name(TEXTURE_UI, asset_loader.as_mut())?;
        self.highlight = catalog.get_sprite_in_state(TEXTURE_UI, TEXTURE_UI_FOCUS_TAG);

        let Some(blip) = catalog.get_sfx_id_for_name(SFX_BLIP) else {
            return Err("cannot init scene if sound effect not available: blip".into());
        };
        let _ = audio.load_sfx(blip);

        let ids = StorySceneIds {
            btn_sfx_id: blip,
            ui_textures_id,
        };
        self.ids = ids.into();

        let Some(font_library) = game_context.font_library.as_mut() else {
            return Err("could not init scene, font library not set".into());
        };
        font_library.ensure(STORY_FONT, asset_loader.as_mut())?;
        font_library.ensure(BUTTON_FONT, asset_loader.as_mut())?;

        let Some((scene_id, pc)) = game_context.story_scene.as_ref() else {
            return Err("No scene set to load during init of story.".into());
        };
        self.load_story_scene_contents(*scene_id, *pc, game_context)?;
        Ok(())
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) -> GameResult<()> {
        if let Some(mut fade) = self.fade.take() {
            fade.update(ticks, game_context)?;
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
                if let Some(audio) = game_context.audio.as_mut()
                    && let Some(id) = self.ids.as_ref().map(|ids| ids.btn_sfx_id)
                {
                    let _ = audio.play_sfx(id);
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
            if vn.is_flag_set(VN_QUIT_FLAG) {
                game_context.shutdown();
                self.fade = Some(TransitionScene::new_fade_out());
            }
            if let Some(next_scene) = vn.to_scene.take() {
                game_context.queue_story_scene(next_scene);
            }
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
                    .filter_map(|(idx, choice)| {
                        let text = choice.text.clone();
                        let ids = self.ids.as_ref()?;
                        let mut b = Button::new(
                            text,
                            Rect {
                                x: 0,
                                y: idx as isize,
                                width: 1,
                                height: 1,
                            },
                            BUTTON_FONT,
                            ids.ui_textures_id,
                        );
                        let _ = b.init(game_context);
                        Some((choice, b))
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
                self.next_btn = self.next_btn();
                let _ = self.next_btn.as_mut().map(|b| b.init(game_context));
            }
            if self.speed_btn.is_none() && self.text_box.is_some() {
                self.speed_btn = self.speed_btn();
                let _ = self.speed_btn.as_mut().map(|b| b.init(game_context));
            }
        }
        if let Some((scene_id, pc)) = game_context.story_scene.take() {
            self.load_story_scene_contents(scene_id, pc, game_context)?
        }
        Ok(())
    }
    fn draw(&mut self, game_context: &mut GameContext) -> GameResult<()> {
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
            fade.draw(game_context)?;
        };
        Ok(())
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
