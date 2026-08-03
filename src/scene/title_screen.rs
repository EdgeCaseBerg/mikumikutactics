use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::asset_loader::ensure_texture_by_name;
use crate::constants::{
    FontId, SFX_BLIP, SFX_MEME, SfxId, TEXTURE_ID_LEEKSHEET, TEXTURE_TITLE_BG, TextureId,
    sprite_info_title,
};
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::scene::button::Button;
use crate::scene::story_screen::StoryScene;
use crate::scene::transitions::{Transition, TransitionScene};
use crate::{ReadyState, advance_ready_state};

pub struct TitleScene {
    bg: SpriteInfo,
    quit_btn: Button,
    played_intro: ReadyState,
    fade: Option<TransitionScene>,
    ids: Option<TitleSceneIds>,
}

struct TitleSceneIds {
    btn_sfx_id: SfxId,
    meme_id: SfxId,
    bg_id: TextureId,
}

impl TitleScene {
    fn layout(game_context: &GameContext) -> GridLayout {
        let (screen_width, screen_height) = game_context.screen_size;
        GridLayout {
            area: Rect::new(0, 0, screen_width as isize, screen_height as isize),
            rows: 27,
            columns: 48,
            cell_gap: 0,
        }
    }
}

impl Default for TitleScene {
    fn default() -> TitleScene {
        TitleScene {
            bg: sprite_info_title(),
            //320, 150
            quit_btn: Button::new(
                "Test".to_string(),
                Rect {
                    x: 32,
                    y: 18,
                    width: 10,
                    height: 2,
                },
                FontId::BoldPixelScaled { pixel_size: 24 },
            ),
            played_intro: ReadyState::Ready,
            fade: None,
            ids: None,
        }
    }
}

impl Scene for TitleScene {
    fn init(&mut self, game_context: &mut GameContext) {
        self.fade = TransitionScene::new_fade_in().into();

        if let (Some(audio), Some(ref mut asset_loader)) = (
            game_context.audio.as_mut(),
            game_context.asset_loader.as_mut(),
        ) {
            let bg_id = ensure_texture_by_name(TEXTURE_TITLE_BG, asset_loader.as_mut()).expect("failed to load title background");
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_LEEKSHEET);
            let catalog = asset_loader.get_catalog();
            if let (Some(meme), Some(blip)) = (
                catalog.get_sfx_id_for_name(SFX_MEME),
                catalog.get_sfx_id_for_name(SFX_BLIP),
            ) {
                let _ = audio.load_sfx(meme);
                let _ = audio.load_sfx(blip);
                let ids = TitleSceneIds {
                    meme_id: meme,
                    btn_sfx_id: blip,
                    bg_id,
                };
                self.ids = ids.into();
            }
        };

        if let (Some(font_library), Some(asset_loader)) = (
            game_context.font_library.as_mut(),
            game_context.asset_loader.as_mut(),
        ) {
            let _ = font_library.ensure(FontId::BoldPixels, asset_loader.as_mut());
            let _ = font_library.ensure(self.quit_btn.font_id, asset_loader.as_mut());
        }
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) {
        if let Some(mut fade) = self.fade.take() {
            fade.update(ticks, game_context);
            let fade = match (fade.is_done(), &fade.transition) {
                (true, Transition::FadeIn { .. }) => None,
                (true, Transition::FadeOut { .. }) => {
                    game_context.next_scene = Some(Box::new(StoryScene::default()));
                    // don't remove the fade on fadeout because we're going to swap scenes
                    // anyway and we need to be at the ReadyState::Ready to trigger fully
                    // opaque draws.
                    Some(fade)
                }
                _ => Some(fade),
            };
            self.fade = fade;
        }

        let layout = TitleScene::layout(game_context);
        self.quit_btn.update(ticks, game_context, &layout);

        if let ReadyState::Ready = self.played_intro {
            self.played_intro = ReadyState::Cooldown {
                wait_for: u32::MAX,
                ticks_waited: 0,
            };
            if let Some(audio) = game_context.audio.as_mut() {
                if let Some(id) = self.ids.as_ref().map(|ids| ids.meme_id) {
                    let _ = audio.play_sfx(id);
                }
            };
        }

        self.played_intro = advance_ready_state(self.played_intro, ticks);

        if self.quit_btn.clicked && game_context.next_scene.is_none() {
            if let Some(audio) = game_context.audio.as_mut() {
                if let Some(id) = self.ids.as_ref().map(|ids| ids.btn_sfx_id) {
                    let _ = audio.play_sfx(id);
                }
            }
            // TODO restore shut down once we have another scene to swap to
            // game_context.shutdown();
            self.quit_btn.clicked = false;
            game_context.mouse_context.consume_left_click();
            self.fade = Some(TransitionScene::new_fade_out());
        }
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        let layout = TitleScene::layout(game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };
        let Some(ids) = self.ids.as_ref() else {
            return;
        };

        let src = self.bg.get_rect();
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: ids.bg_id,
            source: src,
            destination: Rect::new(0, 0, layout.area.width, layout.area.height),
        });

        self.quit_btn.draw(game_context, &layout);
        if let Some(fade) = self.fade.as_mut() {
            fade.draw(game_context);
        }
    }
}

#[cfg(test)]
mod transition_scene_tests {
    use super::*;
    use crate::backend_dummy::FakeRenderer;
    use crate::backend_dummy::click_button_with_left_mouse;
    use crate::constants::{SfxId, TextureId};

    #[test]
    fn clicking_quit_triggers_fade_out_then_next_scene_set() {
        let (renderer, actions) = FakeRenderer::new();
        let mut scene = TitleScene::default();
        scene.ids = Some(TitleSceneIds {
            bg_id: TextureId(0),
            btn_sfx_id: SfxId(1),
            meme_id: SfxId(2),
        });

        let mut game_context = GameContext::default();
        game_context.renderer = Some(Box::new(renderer));

        let layout = TitleScene::layout(&game_context);

        scene.init(&mut game_context);
        click_button_with_left_mouse(&scene.quit_btn, &mut game_context, &layout);
        let game_ticks = 60;
        for _ in 0..game_ticks {
            scene.update(1, &mut game_context);
            // The first update should consume the click
            assert_eq!(false, game_context.mouse_context.left_clicked);
            scene.draw(&mut game_context);
        }

        // Then after the fade finishes up, the next_scene will be set
        // and wouldn't be unset because that's the job of the event loop
        assert!(game_context.next_scene.is_some());

        // And confirm there was a fade at some point here.
        let binding = actions.borrow();
        let times_faded = binding
            .iter()
            .filter(|action| match action {
                RenderCommand::FillRect { color, .. } => color.a == 1.0,
                _ => false,
            })
            .count();
        assert!(times_faded >= 1);
    }
}
