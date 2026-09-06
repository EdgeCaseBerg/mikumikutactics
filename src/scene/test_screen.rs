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
    focus_sprite: Option<(SpriteInfo, TextureId)>,
    current_portrait: Option<(SpriteInfo, TextureId)>,
    current_portrait_bg: Option<(SpriteInfo, TextureId)>,
    btn_prototype: Option<Button>,
    map: Option<(MapInstance, MapState)>,
    context_menu: Option<ContextMenu>,
}

#[derive(Default)]
pub struct ContextMenu {
    move_btn: Option<(Button, FromHere, ToThere)>,
    attack_btn: Option<(Button, Attacker, Defender)>,
    cancel_btn: Option<Button>,
}

impl ContextMenu {
    // where the context menu's top left corner is, shifted to ensure it doesn't render
    // off the side of the screen by accident
    fn anchor(
        map_instance: &MapInstance,
        btn_size: (isize, isize),
        clicked_position: Coordinate,
        num_btns: isize,
    ) -> Rect {
        let (bw, bh) = btn_size;
        assert!(bw + 1 < map_instance.width);
        assert!(bh + 1 < map_instance.height);
        let w = map_instance.width;
        let h = map_instance.height;
        let (row, column) = clicked_position;
        let c = if column + 1 + bw < w {
            column + 1
        } else {
            column - bw - 1
        };
        let r = if row - 1 + bh * num_btns < h {
            row
        } else {
            row - bh
        };
        Rect::new(c, r, bw, bh)
    }
    fn new(
        map_instance: &MapInstance,
        anchor: Rect,
        btn_prototype: &Option<Button>,
        clicked_position: Coordinate,
        selected_unit_position: Coordinate,
        selected_unit_id: UnitId,
    ) -> ContextMenu {
        let (r, c) = clicked_position;
        let (selected_row, selected_column) = selected_unit_position;
        let mut y_offset = 0;
        let mut next_rect = || {
            let mut rect = anchor.clone();
            rect.y += y_offset;
            y_offset += 1;
            rect
        };

        let tile = map_instance.get((r, c));

        let move_btn = if tile.is_some_and(|tile| tile.occupant.is_none()) {
            new_btn(btn_prototype, "Move".to_owned(), next_rect())
                .map(|btn| (btn, selected_unit_position, clicked_position))
        } else {
            None
        };

        let attack_btn = tile.and_then(|tile| tile.occupant).and_then(|target_id| {
            new_btn(btn_prototype, "Attack".to_owned(), next_rect()).map(|btn| {
                let attacker = (selected_unit_id, selected_unit_position);
                let defender = (target_id, clicked_position);
                (btn, attacker, defender)
            })
        });

        let cancel_btn = new_btn(btn_prototype, "Cancel".to_owned(), next_rect());

        ContextMenu {
            move_btn,
            attack_btn,
            cancel_btn,
        }
    }
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
fn new_btn(btn_prototype: &Option<Button>, text: String, rect: Rect) -> Option<Button> {
    let mut btn = btn_prototype.clone()?;
    btn.text = text;
    btn.rect = rect;
    Some(btn)
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
                        range: 3,
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

        self.focus_sprite = Some(sprite_tuple("ui", Some(&["focus"]), asset_loader.as_mut())?);
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
        self.btn_prototype = Some(Button::new(
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
        self.btn_prototype.as_mut().unwrap().init(game_context)?;
        Ok(())
    }
    fn update(&mut self, ticks: u32, game_context: &mut GameContext) -> GameResult<()> {
        if let Some((sprite_info, _)) = self.focus_sprite.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some((sprite_info, _)) = self.current_portrait_bg.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some((sprite_info, _)) = self.current_portrait.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some(btn) = self.btn_prototype.as_mut() {
            let layout = TestScene::layout(game_context);
            btn.update(ticks, game_context, &layout);
        }
        if let Some((map_instance, map_state)) = self.map.as_mut() {
            'action: {
                match map_state.action {
                    None => {
                        if let Some((action, unit_data, reachable_positions)) =
                            check_if_unit_selected_action(game_context, map_instance, map_state)
                        {
                            self.current_portrait = selected_portrait_from(unit_data);
                            map_state.selectable_tiles = reachable_positions;
                            map_state.action = action;
                        };
                    }
                    Some(ActionState::UnitSelected(unit_id, (selected_row, selected_column))) => {
                        match valid_selected_tile_clicked(
                            game_context,
                            map_instance,
                            map_state,
                            (selected_row, selected_column),
                        ) {
                            None => {}
                            Some(None) => {
                                map_state.action = None;
                                map_state.selectable_tiles = HashSet::new();
                                self.current_portrait = None;
                            }
                            Some(Some((r, c))) => {
                                map_state.action =
                                    Some(ActionState::CommandPending(unit_id, (r, c)));
                                let anchor = ContextMenu::anchor(map_instance, (2, 1), (r, c), 2);
                                self.context_menu = Some(ContextMenu::new(
                                    map_instance,
                                    anchor,
                                    &self.btn_prototype,
                                    (r, c),
                                    (selected_row, selected_column),
                                    unit_id,
                                ));
                            }
                        }
                    }
                    Some(ActionState::CommandPending(_unit_id, (target_row, target_column))) => {
                        if !game_context.mouse_context.right_clicked {
                            break 'action;
                        }
                        // Check if each context menu btn is being pressed or not, update/advance them.
                        map_state.action = None;
                        map_state.selectable_tiles = HashSet::new();
                        self.context_menu = None;
                        self.current_portrait = None;
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
        draw_context_menu(self, &layout, game_context);
        if let Some(btn) = self.btn_prototype.as_ref() {
            btn.draw(game_context, &layout);
        }
        Ok(())
    }
}

fn check_if_unit_selected_action<'a>(
    game_context: &GameContext,
    map_instance: &MapInstance,
    map_state: &'a MapState,
) -> Option<(
    Option<ActionState>,
    Option<&'a UnitData>,
    HashSet<(isize, isize)>,
)> {
    if !game_context.mouse_context.left_clicked {
        return None;
    }
    let layout = TestScene::layout(game_context);
    let (r, c, _) = layout.cell_for_mouse(game_context.mouse_context.position)?;
    let tile = map_instance.get((r as isize, c as isize))?;
    if let Some(uid) = tile.occupant {
        if let Some(unit_data) = map_state.units.get(&uid) {
            let position = (r as isize, c as isize);
            let action = Some(ActionState::UnitSelected(uid, position));
            let reachable_positions = reachable_by_unit(unit_data, position, map_instance);
            return Some((action, Some(unit_data), reachable_positions));
        }
    } else {
        return Some((None, None, HashSet::new()));
    }
    None
}

fn selected_portrait_from(unit_data: Option<&UnitData>) -> Option<(SpriteInfo, TextureId)> {
    let unit_data = unit_data?;
    let UnitSpriteData { portrait, .. } = &unit_data.sprite_data;
    Some(portrait.clone())
}

fn valid_selected_tile_clicked(
    game_context: &GameContext,
    map_instance: &MapInstance,
    map_state: &MapState,
    selected_tile_position: (isize, isize),
) -> Option<Option<(isize, isize)>> {
    if !game_context.mouse_context.left_clicked {
        return None;
    }
    let layout = TestScene::layout(game_context);
    let (r, c, _) = layout.cell_for_mouse(game_context.mouse_context.position)?;
    let (selected_row, selected_column) = selected_tile_position;

    // Double clicking on the selected unit does nothing
    if selected_row == r as isize && selected_column == c as isize {
        return None;
    }
    // Clicking outside of the range of the unit should revert us to no action
    if !map_state
        .selectable_tiles
        .contains(&(r as isize, c as isize))
    {
        return Some(None);
    }

    // Ensure that the click within range is on the map
    let tile = map_instance.get((r as isize, c as isize))?;

    Some(Some((r as isize, c as isize)))
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
            if let Some((sprite_info, texture_id)) = scene.focus_sprite.as_ref()
                && map_state.selectable_tiles.contains(&(row, column))
            {
                let src = sprite_info.get_rect();
                let dst = layout.cell_rect(row as usize, column as usize);
                renderer.send_command(RenderCommand::DrawRect {
                    texture_id: *texture_id,
                    source: src,
                    destination: dst,
                });
            }
            if let Some(uid) = tile.occupant
                && let Some(unit_data) = map_state.units.get(&uid)
            {
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

fn draw_context_menu(scene: &TestScene, layout: &GridLayout, game_context: &mut GameContext) {
    let Some(menu) = scene.context_menu.as_ref() else {
        return;
    };
    if let Some((btn, ..)) = menu.move_btn.as_ref() {
        btn.draw(game_context, layout);
    }
    if let Some((btn, ..)) = menu.attack_btn.as_ref() {
        btn.draw(game_context, layout);
    }
    if let Some(btn) = menu.cancel_btn.as_ref() {
        btn.draw(game_context, layout);
    }
}
