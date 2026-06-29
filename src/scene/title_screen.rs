use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::constants::{
    SFX_ID_BLIP, SFX_ID_MEME, TEXTURE_ID_LEEKSHEET, TEXTURE_ID_TITLE_BG, sprite_info_title,
};
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::scene::button::Button;
use crate::scene::transitions::{Transition, TransitionScene};
use crate::{ReadyState, advance_ready_state};

pub struct TitleScene {
    bg: SpriteInfo,
    quit_btn: Button,
    played_intro: ReadyState,
    fade: Option<TransitionScene>,
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
                "Quit".to_string(),
                Rect {
                    x: 32,
                    y: 18,
                    width: 10,
                    height: 2,
                },
            ),
            played_intro: ReadyState::Ready,
            fade: None,
        }
    }
}

impl Scene for TitleScene {
    fn init(&mut self, game_context: &mut GameContext) {
        self.fade = TransitionScene::FadeIn().into();

        let Some(ref mut asset_loader) = game_context.asset_loader else {
            return;
        };

        asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_TITLE_BG);
        asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_LEEKSHEET);

        let Some(ref mut audio) = game_context.audio else {
            return;
        };

        let _ = audio.load_sfx(SFX_ID_MEME);
        let _ = audio.load_sfx(SFX_ID_BLIP);
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) {
        let remove_fade = self.fade.as_mut().map_or(false, |fade| {
            fade.update(ticks, game_context);
            match (fade.is_done(), &fade.transition) {
                (true, Transition::FadeOut { .. }) => {
                    // TitleScene for now because I don't have anything else to use
                    game_context.next_scene = Some(Box::new(TitleScene::default()));
                    true
                }
                (true, _) => true,
                _ => false,
            }
        });
        if remove_fade {
            self.fade = None;
        }

        let layout = TitleScene::layout(&game_context);
        self.quit_btn.update(ticks, game_context, &layout);

        match self.played_intro {
            ReadyState::Ready => {
                self.played_intro = ReadyState::Cooldown {
                    wait_for: u32::MAX,
                    ticks_waited: 0,
                };
                game_context.audio.as_mut().map(|audio| {
                    let _ = audio.play_sfx(SFX_ID_MEME);
                });
            }
            _ => {}
        }

        self.played_intro = advance_ready_state(self.played_intro, ticks);

        if self.quit_btn.clicked && game_context.next_scene.is_none() {
            game_context.audio.as_mut().map(|audio| {
                let _ = audio.play_sfx(SFX_ID_BLIP);
            });
            // TODO restore shut down once we have another scene to swap to
            // game_context.shutdown();
            self.quit_btn.clicked = false;
            game_context.mouse_context.consume_left_click();
            self.fade = Some(TransitionScene::FadeOut());
        }
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        let layout = TitleScene::layout(&game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };

        let src = self.bg.get_rect();
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: TEXTURE_ID_TITLE_BG,
            source: src,
            destination: Rect::new(0, 0, layout.area.width, layout.area.height),
        });

        self.quit_btn.draw(game_context, &layout);
        self.fade.as_mut().map(|f| f.draw(game_context));
    }
}
