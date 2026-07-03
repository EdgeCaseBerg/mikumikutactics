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

pub struct StoryScene {
    bg: SpriteInfo,
    fade: Option<TransitionScene>,
}

impl StoryScene {
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

impl Default for StoryScene {
    fn default() -> StoryScene {
        StoryScene {
            bg: sprite_info_title(),
            fade: None,
        }
    }
}

impl Scene for StoryScene {
    fn init(&mut self, game_context: &mut GameContext) {
        self.fade = TransitionScene::new_fade_in().into();

        if let Some(ref mut asset_loader) = game_context.asset_loader {
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_TITLE_BG);
            asset_loader.ensure_texture_spritesheet_loaded(TEXTURE_ID_LEEKSHEET);
        } else {
            return;
        };

        if let Some(ref mut audio) = game_context.audio {
            let _ = audio.load_sfx(SFX_ID_BLIP);
        } else {
            return;
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

        let layout = StoryScene::layout(&game_context);
    }
    fn draw(&mut self, game_context: &mut GameContext) {
        let layout = StoryScene::layout(&game_context);
        let Some(ref mut renderer) = game_context.renderer else {
            return;
        };

        let src = self.bg.get_rect();
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: TEXTURE_ID_TITLE_BG,
            source: src,
            destination: Rect::new(0, 0, layout.area.width, layout.area.height),
        });

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
