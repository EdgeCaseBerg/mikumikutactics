use crate::GameResult;
use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::asset_loader::{AssetLoader, ensure_texture_by_name};
use crate::constants::TextureId;
use crate::constants::{FontId, TEXTURE_UI};
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::renderer::Renderer;
use crate::scene::button::Button;
use crate::tactics::units::*;
use std::collections::HashSet;

#[derive(Default)]
pub struct TestScene {
    test_sprite: Option<(SpriteInfo, TextureId)>,
    current_portrait: Option<(SpriteInfo, TextureId)>,
    current_portrait_bg: Option<(SpriteInfo, TextureId)>,
    test_btn: Option<Button>,
    map: Option<(MapInstance, MapState)>,
}

impl TestScene {
    fn layout(game_context: &GameContext) -> GridLayout {
        let (screen_width, screen_height) = game_context.screen_size;
        GridLayout {
            area: Rect::new(0, 0, screen_width as isize, screen_height as isize),
            rows: 18,
            columns: 32,
            cell_gap: 0,
        }
    }
}

fn sprite_tuple(
    name: &str,
    tags: Option<&[&str]>,
    asset_loader: &mut dyn AssetLoader,
) -> GameResult<(SpriteInfo, TextureId)> {
    let catalog = asset_loader.get_catalog();
    let sprite_info = match tags {
        Some(tags) => {
            let Ok(sprite_info) = catalog.get_sprite_with_tags(name, tags) else {
                return Err(
                    format!("cannot init scene if {name} {tags:?} texture not available").into(),
                );
            };
            sprite_info
        }
        None => catalog.get_sprite(name)?,
    };

    let id = ensure_texture_by_name(name, asset_loader)?;
    Ok((sprite_info, id))
}

fn make_map() -> MapInstance {
    let rows = 18;
    let columns = 32;
    MapInstance::empty_walkable(columns, rows)
}

impl Scene for TestScene {
    fn init(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        let Some(asset_loader) = game_context.asset_loader.as_mut() else {
            return Err("cannot init scene if asset_loader is not set".into());
        };
        let mut the_map = make_map();
        let map_state = MapState {
            action: None,
            selectable_tiles: HashSet::new(),
            units: (0..2)
                .map(|uid| {
                    let id = UnitId(uid);
                    let name = if uid % 2 == 0 { "miku" } else { "meiko" };
                    let p = sprite_tuple(
                        "portrait",
                        Some(&[name, "talking-idle"]),
                        asset_loader.as_mut(),
                    )
                    .expect("cannot load miku portrait");
                    let s = sprite_tuple("onmap", Some(&[name]), asset_loader.as_mut())
                        .expect("cannot load miku sprite");
                    let unit_data = UnitData {
                        name: name.to_owned(),
                        sprite_data: UnitSpriteData {
                            sprite: s,
                            portrait: p,
                        },
                    };
                    let tile = the_map
                        .get_mut((3, 3 + uid as isize))
                        .expect("couldnt get tile to place unit in");
                    tile.occupant = Some(id);
                    (id, unit_data)
                })
                .collect(),
        };
        self.map = Some((the_map, map_state));

        let ui_texture_id = ensure_texture_by_name(TEXTURE_UI, asset_loader.as_mut())?;

        self.test_sprite = Some(sprite_tuple("onmap", None, asset_loader.as_mut())?);
        self.current_portrait = None;
        self.current_portrait_bg = Some(sprite_tuple(
            "portrait",
            Some(&["background"]),
            asset_loader.as_mut(),
        )?);

        let font_id = FontId::BoldPixelScaled { pixel_size: 24 };
        let Some(font_library) = game_context.font_library.as_mut() else {
            return Err("cannot init scene if font_library is not set".into());
        };
        font_library.ensure(font_id, asset_loader.as_mut())?;
        self.test_btn = Some(Button::new(
            "Test".to_string(),
            Rect {
                x: 16,
                y: 9,
                width: 2,
                height: 2,
            },
            font_id,
            ui_texture_id,
        ));
        self.test_btn.as_mut().unwrap().init(game_context)?;
        Ok(())
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) -> GameResult<()> {
        if let Some((sprite_info, _)) = self.test_sprite.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some((sprite_info, _)) = self.current_portrait_bg.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some((sprite_info, _)) = self.current_portrait.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some(btn) = self.test_btn.as_mut() {
            let layout = TestScene::layout(game_context);
            btn.update(ticks, game_context, &layout);
        }
        if let Some((map_instance, map_state)) = self.map.as_mut() {
            'action: {
                match map_state.action {
                    None => {
                        let Some((action, unit_data)) = check_if_unit_selected_action(
                            &game_context,
                            &map_instance,
                            &map_state,
                        ) else {
                            // If the state function said there was no update, then bail.
                            break 'action;
                        };
                        // Otherwise, take the action (which might clear via None) and update the portrait
                        if let Some(unit_data) = unit_data {
                            let UnitSpriteData { portrait, .. } = &unit_data.sprite_data;
                            self.current_portrait = Some(portrait.clone());
                            // Thought: create hash set of selected tiles based on range of unit, computed once now
                            // and then used to validate if next click is within the range of the unit and should result
                            // in anything happening or not.
                            eprintln!("swap to selected");
                        } else {
                            self.current_portrait = None;
                        }
                        map_state.action = action;
                    }
                    Some(ActionState::UnitSelected(_unit_id, (selected_row, selected_column))) => {
                        if !game_context.mouse_context.left_clicked {
                            break 'action;
                        }
                        let layout = TestScene::layout(game_context);
                        let Some((r, c, _)) =
                            layout.cell_for_mouse(game_context.mouse_context.position)
                        else {
                            break 'action;
                        };
                        // Double clicking on the selected unit does nothing
                        if selected_row == r as isize && selected_column == c as isize {
                            break 'action;
                        }
                        // If we're clicking on something else though?
                        let Some(_tile) = map_instance.get((r as isize, c as isize)) else {
                            break 'action;
                        };

                        // for now clear state. but when ready, this would prompt open a menu and do range checks?
                        map_state.action = None;
                        eprintln!("swap to cleared");
                    }
                    _ => {}
                }
            }
        };

        Ok(())
    }
    fn draw(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        let layout = TestScene::layout(game_context);
        let Some(renderer) = game_context.renderer.as_mut() else {
            return Err("no renderer defined in game context, not drawing scene.".into());
        };
        draw_map(self, &layout, renderer.as_mut());
        draw_portrait(self, &layout, renderer.as_mut());
        if let Some(btn) = self.test_btn.as_ref() {
            btn.draw(game_context, &layout);
        }
        Ok(())
    }
}

fn check_if_unit_selected_action<'a>(
    game_context: &GameContext,
    map_instance: &MapInstance,
    map_state: &'a MapState,
) -> Option<(Option<ActionState>, Option<&'a UnitData>)> {
    if !game_context.mouse_context.left_clicked {
        return None;
    }
    let layout = TestScene::layout(game_context);
    let Some((r, c, _)) = layout.cell_for_mouse(game_context.mouse_context.position) else {
        return None;
    };
    let Some(tile) = map_instance.get((r as isize, c as isize)) else {
        return None;
    };
    if let Some(uid) = tile.occupant {
        if let Some(unit_data) = map_state.units.get(&uid) {
            let action = Some(ActionState::UnitSelected(uid, (r as isize, c as isize)));
            return Some((action, Some(unit_data)));
        }
    } else {
        return Some((None, None));
    }
    return None;
}

fn draw_portrait(scene: &TestScene, layout: &GridLayout, renderer: &mut dyn Renderer) {
    let row = 8;
    if let Some((sprite_info, texture_id)) = scene.current_portrait.as_ref() {
        if let Some((sprite_info, texture_id)) = scene.current_portrait_bg.as_ref() {
            let src = sprite_info.get_rect();
            let tl = layout.cell_rect(row, 0);
            let (cw, ch) = layout.cell_size();
            let dst = Rect::new(tl.x, tl.y, cw * 4, ch * 4);
            renderer.send_command(RenderCommand::DrawRect {
                texture_id: *texture_id,
                source: src,
                destination: dst,
            });
        }
        let src = sprite_info.get_rect();
        let tl = layout.cell_rect(row, 0);
        let (cw, ch) = layout.cell_size();
        let dst = Rect::new(tl.x, tl.y, cw * 4, ch * 4);
        renderer.send_command(RenderCommand::DrawRect {
            texture_id: *texture_id,
            source: src,
            destination: dst,
        });
    }
}

fn draw_map(scene: &TestScene, layout: &GridLayout, renderer: &mut dyn Renderer) {
    let Some((map_instance, map_state)) = scene.map.as_ref() else {
        return;
    };
    for row in 0..map_instance.height {
        for column in 0..map_instance.width {
            let Some(tile) = map_instance.get((row, column)) else {
                continue;
            };
            if let Some(uid) = tile.occupant {
                if let Some(unit_data) = map_state.units.get(&uid) {
                    let UnitSpriteData { sprite, .. } = &unit_data.sprite_data;
                    let (sprite_info, texture_id) = sprite;
                    let src = sprite_info.get_rect();
                    let dst = layout.cell_rect(row as usize, column as usize);
                    renderer.send_command(RenderCommand::DrawRect {
                        texture_id: *texture_id,
                        source: src,
                        destination: dst,
                    });
                }
            }
        }
    }
}
