use crate::GameResult;
use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::asset_loader::ensure_texture_by_name;
use crate::constants::{
    FontId, SCENE_START_NAME, SFX_BLIP, SFX_MEME, SceneId, SfxId, TEXTURE_TITLE_BG, TEXTURE_UI,
    TextureId,
};
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::scene::button::Button;
use crate::scene::story_screen::StoryScene;
use crate::scene::transitions::{Transition, TransitionScene};
use crate::{ReadyState, advance_ready_state};

pub struct TitleScene {
    bg: Option<SpriteInfo>,
    quit_btn: Option<Button>,
    start_btn: Option<Button>,
    played_intro: ReadyState,
    fade: Option<TransitionScene>,
    ids: Option<TitleSceneIds>,
}

struct TitleSceneIds {
    btn_sfx_id: SfxId,
    meme_id: SfxId,
    bg_id: TextureId,
    start_scene_id: SceneId,
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
    fn start_btn(font_id: FontId, ui_texture_id: TextureId) -> Button {
        Button::new(
            "Start Game".to_string(),
            Rect {
                x: 32,
                y: 18,
                width: 10,
                height: 2,
            },
            font_id,
            ui_texture_id,
        )
    }
    fn quit_btn(font_id: FontId, ui_texture_id: TextureId) -> Button {
        Button::new(
            "Quit".to_string(),
            Rect {
                x: 32,
                y: 21,
                width: 10,
                height: 2,
            },
            font_id,
            ui_texture_id,
        )
    }
}

impl Default for TitleScene {
    fn default() -> TitleScene {
        TitleScene {
            bg: None,
            //320, 150
            start_btn: None,
            quit_btn: None,
            played_intro: ReadyState::Ready,
            fade: None,
            ids: None,
        }
    }
}

impl Scene for TitleScene {
    fn init(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        self.fade = TransitionScene::new_fade_in().into();
        let Some(asset_loader) = game_context.asset_loader.as_mut() else {
            return Err("cannot init scene if asset_loader is not set".into());
        };

        let catalog = asset_loader.get_catalog();
        self.bg = Some(catalog.get_sprite(TEXTURE_TITLE_BG)?);

        let Some(audio) = game_context.audio.as_mut() else {
            return Err("cannot init scene if audio context is not set".into());
        };
        let meme = catalog.get_sfx_id_for_name(SFX_MEME)?;
        audio.load_sfx(meme)?;
        let blip = catalog.get_sfx_id_for_name(SFX_BLIP)?;
        audio.load_sfx(blip)?;

        let start_scene_id = catalog.get_scene_id_for_name(SCENE_START_NAME)?;
        let bg_id = ensure_texture_by_name(TEXTURE_TITLE_BG, asset_loader.as_mut())?;
        let ids = TitleSceneIds {
            meme_id: meme,
            btn_sfx_id: blip,
            bg_id,
            start_scene_id,
        };
        self.ids = ids.into();

        let Some(font_library) = game_context.font_library.as_mut() else {
            return Err("cannot init scene if font_library is not set".into());
        };
        let ui_id = ensure_texture_by_name(TEXTURE_UI, asset_loader.as_mut())?;
        let btn_font_id = FontId::BoldPixelScaled { pixel_size: 24 };
        self.quit_btn = Some(Self::quit_btn(btn_font_id, ui_id));
        self.start_btn = Some(Self::start_btn(btn_font_id, ui_id));

        font_library.ensure(FontId::BoldPixels, asset_loader.as_mut())?;
        font_library.ensure(btn_font_id, asset_loader.as_mut())?;

        if let Some(btn) = self.quit_btn.as_mut() {
            btn.init(game_context)?;
        }
        if let Some(btn) = self.start_btn.as_mut() {
            btn.init(game_context)?;
        }
        Ok(())
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) -> GameResult<()> {
        if let Some(mut fade) = self.fade.take() {
            fade.update(ticks, game_context)?;
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
        if let Some(btn) = self.quit_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked && game_context.next_scene.is_none() {
                if let Some(audio) = game_context.audio.as_mut()
                    && let Some(id) = self.ids.as_ref().map(|ids| ids.btn_sfx_id)
                {
                    let _ = audio.play_sfx(id);
                }
                game_context.shutdown();
                self.fade = Some(TransitionScene::new_fade_out());
            }
        }

        if let Some(btn) = self.start_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked && game_context.next_scene.is_none() {
                if let Some(audio) = game_context.audio.as_mut()
                    && let Some(id) = self.ids.as_ref().map(|ids| ids.btn_sfx_id)
                {
                    let _ = audio.play_sfx(id);
                }
                let Some(start_scene_id) = self.ids.as_ref().map(|ids| ids.start_scene_id) else {
                    return Err("cannot start new game if scene id is unknown".into());
                };
                game_context.queue_story_scene(start_scene_id);
                self.fade = Some(TransitionScene::new_fade_out());
            }
        }

        if let ReadyState::Ready = self.played_intro {
            self.played_intro = ReadyState::Cooldown {
                wait_for: u32::MAX,
                ticks_waited: 0,
            };
            if let Some(audio) = game_context.audio.as_mut()
                && let Some(id) = self.ids.as_ref().map(|ids| ids.meme_id)
            {
                let _ = audio.play_sfx(id);
            };
        }

        self.played_intro = advance_ready_state(self.played_intro, ticks);
        Ok(())
    }
    fn draw(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        let layout = TitleScene::layout(game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return Err("no renderer defined in game context, not drawing scene.".into());
        };
        let Some(ids) = self.ids.as_ref() else {
            return Err("cannot draw scene if asset ids are not initialized".into());
        };

        if let Some(src) = self.bg.as_ref().map(|s| s.get_rect()) {
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: ids.bg_id,
                source: src,
                destination: Rect::new(0, 0, layout.area.width, layout.area.height),
            });
        }
        if let Some(btn) = self.quit_btn.as_ref() {
            btn.draw(game_context, &layout);
        }
        if let Some(btn) = self.start_btn.as_ref() {
            btn.draw(game_context, &layout);
        }
        if let Some(fade) = self.fade.as_mut() {
            fade.draw(game_context)?;
        }
        Ok(())
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
            start_scene_id: SceneId(0),
        });

        scene.quit_btn = Some(TitleScene::quit_btn(FontId::BoldPixels, TextureId(1)));

        let mut game_context = GameContext::default();
        game_context.renderer = Some(Box::new(renderer));

        let layout = TitleScene::layout(&game_context);

        let _ = scene.init(&mut game_context);
        click_button_with_left_mouse(scene.quit_btn.as_ref().unwrap(), &mut game_context, &layout);
        let game_ticks = 60;
        for _ in 0..game_ticks {
            scene.update(1, &mut game_context).unwrap();
            // The first update should consume the click
            release_button_click_of_left_mouse(
                scene.quit_btn.as_ref().unwrap(),
                &mut game_context,
                &layout,
            );
            assert_eq!(false, game_context.mouse_context.left_clicked);
            scene.draw(&mut game_context).unwrap();
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
