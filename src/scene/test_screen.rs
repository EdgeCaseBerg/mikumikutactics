use crate::GameResult;
use crate::ReadyState;
use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::asset_loader::{AssetCatalog, AssetLoader, ensure_texture_by_name};
use crate::constants::{FontId, TEXTURE_UI};
use crate::constants::{TextureId, WeaponId};
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::renderer::RenderCommand;
use crate::renderer::Renderer;
use crate::scene::button::Button;
use crate::tactics::commands::*;
use crate::tactics::map::*;
use crate::tactics::units::*;
use std::collections::HashSet;

#[derive(Default)]
pub struct TestScene {
    focus_sprite: Option<(SpriteInfo, TextureId)>,
    walkable_sprite: Option<(SpriteInfo, TextureId)>,
    attackable_sprite: Option<(SpriteInfo, TextureId)>,
    danger_sprite: Option<(SpriteInfo, TextureId)>,
    current_portrait: Option<(SpriteInfo, TextureId)>,
    current_portrait_bg: Option<(SpriteInfo, TextureId)>,
    btn_prototype: Option<Button>,
    map: Option<BattleData>,
    context_menu: Option<ContextMenu>,
    command_log: Vec<Box<dyn Command>>,
    command_tick: Option<ReadyState>,
    command_idx: usize,
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
        let mut y_offset = 0;
        let mut next_rect = || {
            let mut rect = anchor;
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
    fn update(
        &mut self,
        ticks: u32,
        game_context: &mut GameContext,
        unit_id: UnitId,
    ) -> GameResult<Option<ActionState>> {
        let layout = TestScene::layout(game_context);
        if let Some((btn, from, to)) = self.move_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked {
                return Ok(Some(ActionState::Move(unit_id, *from, *to)));
            }
        }
        if let Some((btn, attacker, defender)) = self.attack_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked {
                return Ok(Some(ActionState::Attack(*attacker, *defender)));
            }
        }
        if let Some(btn) = self.cancel_btn.as_mut() {
            btn.update(ticks, game_context, &layout);
            if btn.clicked {
                return Err("cancel clicked".into());
            }
        }
        Ok(None)
    }
    fn draw(&self, layout: &GridLayout, game_context: &mut GameContext) {
        if let Some((btn, ..)) = self.move_btn.as_ref() {
            btn.draw(game_context, layout);
        }
        if let Some((btn, ..)) = self.attack_btn.as_ref() {
            btn.draw(game_context, layout);
        }
        if let Some(btn) = self.cancel_btn.as_ref() {
            btn.draw(game_context, layout);
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
                        health: 10,
                        sprite_data: UnitSpriteData {
                            sprite: s,
                            portrait: p,
                        },
                        move_range: 3,
                        weapon: Some(WeaponId(uid % 2)),
                    };
                    let tile = the_map
                        .get_mut((3, 3 + uid as isize))
                        .expect("couldnt get tile to place unit in");
                    tile.occupant = Some(id);
                    (id, unit_data)
                })
                .collect(),
        };
        self.map = Some(BattleData {
            map_instance: the_map,
            map_state,
        });

        let ui_texture_id = ensure_texture_by_name(TEXTURE_UI, asset_loader.as_mut())?;

        self.focus_sprite = Some(sprite_tuple("ui", Some(&["focus"]), asset_loader.as_mut())?);
        self.walkable_sprite = Some(sprite_tuple(
            "ui",
            Some(&["walkable"]),
            asset_loader.as_mut(),
        )?);
        self.attackable_sprite = Some(sprite_tuple(
            "ui",
            Some(&["attackable"]),
            asset_loader.as_mut(),
        )?);
        self.danger_sprite = Some(sprite_tuple(
            "ui",
            Some(&["danger"]),
            asset_loader.as_mut(),
        )?);
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
        if let Some((sprite_info, _)) = self.walkable_sprite.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some((sprite_info, _)) = self.attackable_sprite.as_mut() {
            sprite_info.advance(ticks);
        }
        if let Some((sprite_info, _)) = self.danger_sprite.as_mut() {
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
            if btn.clicked {
                if let Some(battle_data) = self.map.as_mut() {
                    eprintln!("{:?}", self.command_log.len());
                    battle_data.map_state.action = Some(ActionState::RewindCommandQueue(0));
                }
            }
        }
        if let Some(battle_data) = self.map.as_mut() {
            let BattleData {
                map_instance,
                map_state,
            } = battle_data;
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
                            map_state.action = Some(ActionState::CommandPending(unit_id, (r, c)));
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
                Some(ActionState::CommandPending(unit_id, _)) => {
                    if let Some(menu) = self.context_menu.as_mut() {
                        match menu.update(ticks, game_context, unit_id) {
                            Ok(None) => {}
                            Ok(new_action) => {
                                self.context_menu = None;
                                map_state.action = new_action;
                                map_state.selectable_tiles = HashSet::new();
                                // If we were doing something like making the unit speak,
                                // then perhaps play it here like "I'm on my way!" And if
                                // we want to do animation of the character moving, then we
                                // could compute all of that here and then replay it back?
                                // or have it happen while processing that command perhaps
                            }
                            Err(_) => {
                                // Error is the cancel button signal
                                map_state.action = Some(ActionState::ClearAction);
                            }
                        }
                    } else {
                        assert!(
                            self.context_menu.is_some(),
                            "entered command pending state with no context menu options",
                        );
                    }
                }
                Some(ActionState::Move(unit_id, from, to)) => {
                    // For now, just move instantly.
                    // TODO: advance the unit along a computed path
                    // and when they reach the <to> then trigger move_unit_to
                    // or something similar.
                    // map_instance.move_unit_to(unit_id, from, to)?;
                    let command = MoveCommand { unit_id, from, to };
                    self.command_log.push(Box::new(command));
                    // TODO: suppose we also need to mark the unit not-selectable for the rest of their turn?
                    // that'd probably be a separate action from ClearAction
                    map_state.action = Some(ActionState::DrainCommandQueue);
                }
                Some(ActionState::Attack(attacker, defender)) => {
                    eprintln!("{:?} wants to attack {:?}", attacker, defender);
                    let (attacker_uid, attacker_position) = attacker;
                    let catalog = game_context
                        .asset_loader
                        .as_ref()
                        .map(|a| a.get_catalog())
                        .expect("no catalog?");

                    match determine_move_to_and_new_health(
                        attacker,
                        defender,
                        map_instance,
                        map_state,
                        catalog.as_ref(),
                    ) {
                        None => { /* Weird state. The attacker or defender dont exist */ }
                        Some((Some(required_move), (new_health, old_health))) => {
                            let command = MoveCommand {
                                unit_id: attacker_uid,
                                from: attacker_position,
                                to: required_move,
                            };
                            self.command_log.push(Box::new(command));

                            /* Lazy player, requested attack that requires a unit to move first */
                            let command = AttackCommand {
                                defender,
                                old_health,
                                new_health,
                            };
                            self.command_log.push(Box::new(command));
                            map_state.action = Some(ActionState::DrainCommandQueue);
                        }
                        Some((None, (new_health, old_health))) => {
                            let command = AttackCommand {
                                defender,
                                old_health,
                                new_health,
                            };
                            self.command_log.push(Box::new(command));
                            map_state.action = Some(ActionState::DrainCommandQueue);
                        }
                    }
                }
                Some(ActionState::ClearAction) => {
                    map_state.action = None;
                    map_state.selectable_tiles = HashSet::new();
                    self.context_menu = None;
                    self.current_portrait = None;
                }
                Some(ActionState::DrainCommandQueue) => {
                    match self.command_log.get(self.command_idx) {
                        None => {
                            // There is no ticking command, so reset ui state
                            eprintln!("reset ui state");
                            map_state.action = Some(ActionState::ClearAction);
                        }
                        Some(command) => {
                            // Have we started this command yet?
                            match self.command_tick.as_mut() {
                                None => {
                                    // Nope. Set it up.
                                    self.command_tick = Some(ReadyState::Cooldown {
                                        wait_for: command.as_ref().tick_duration(),
                                        ticks_waited: 0,
                                    });
                                    eprintln!("{:?} tick tick ", self.command_tick);
                                }
                                Some(ready_state) => {
                                    *ready_state = ready_state.advance(ticks);
                                    if matches!(ready_state, ReadyState::Ready) {
                                        eprintln!("{:?} tick tock ", ready_state);
                                        let _ = command.apply(battle_data)?;
                                        self.command_tick = None;
                                        self.command_idx += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                Some(ActionState::RewindCommandQueue(to)) => {
                    map_state.action = Some(ActionState::ClearAction);
                    assert!(to <= self.command_idx);
                    while self.command_idx >= to {
                        self.command_idx = self.command_idx.saturating_sub(1);
                        if let Some(command) = self.command_log.pop() {
                            let _ = command.undo(battle_data)?;
                        }

                        if self.command_idx == 0 || self.command_idx < to {
                            break;
                        }
                    }
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
        if let Some(menu) = self.context_menu.as_ref() {
            menu.draw(&layout, game_context);
        }
        if let Some(btn) = self.btn_prototype.as_ref() {
            btn.draw(game_context, &layout);
        }
        Ok(())
    }
}

fn determine_move_to_and_new_health(
    attacker: Attacker,
    defender: Defender,
    map_instance: &MapInstance,
    map_state: &MapState,
    asset_catalog: &AssetCatalog,
) -> Option<(Option<Coordinate>, (usize, usize))> {
    eprintln!("{:?} wants to attack {:?}", attacker, defender);
    let (attacker_uid, attacker_position) = attacker;
    let (defender_uid, defender_position) = defender;

    let attacker = map_state.units.get(&attacker_uid)?;
    let move_to = choose_tile_to_attack_from(
        attacker,
        attacker_position,
        defender_position,
        map_instance,
        asset_catalog,
    );

    let new_health = map_state
        .units
        .get(&defender_uid)
        .map(|defender| {
            (
                calculate_health_after_attack(attacker, defender, asset_catalog),
                defender.health,
            )
        })
        .unwrap_or((0, 0));

    Some((move_to, new_health))
}

fn update_defender_health(
    defender: Defender,
    new_health: usize,
    map_instance: &mut MapInstance,
    map_state: &mut MapState,
) -> GameResult<()> {
    let (defender_uid, defender_position) = defender;
    let Some(defender) = map_state.units.get_mut(&defender_uid) else {
        return Err(format!("defending unit does not exist in units list {defender_uid:?}").into());
    };
    defender.health = new_health;
    eprintln!("defender now has health {:?}", defender.health);
    if defender.health == 0 {
        map_instance
            .get_mut(defender_position)
            .and_then(|tile| tile.occupant.take());
        eprintln!("unit {:?} defeated", defender_uid);
    }
    Ok(())
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
    // If there is no catalog to use for weapon checks, then we can't determine actions and something is quite wrong with the world.
    let catalog = game_context
        .asset_loader
        .as_ref()
        .map(|a| a.get_catalog())?;
    let layout = TestScene::layout(game_context);
    let (r, c, _) = layout.cell_for_mouse(game_context.mouse_context.position)?;
    let tile = map_instance.get((r as isize, c as isize))?;
    if let Some(uid) = tile.occupant {
        if let Some(unit_data) = map_state.units.get(&uid) {
            let position = (r as isize, c as isize);
            let action = Some(ActionState::UnitSelected(uid, position));
            let reachable_positions =
                reachable_by_unit(unit_data, position, map_instance, catalog.as_ref());
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
    let _tile = map_instance.get((r as isize, c as isize))?;

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
    let Some(BattleData {
        map_instance,
        map_state,
    }) = scene.map.as_ref()
    else {
        return;
    };
    for row in 0..map_instance.height {
        for column in 0..map_instance.width {
            let Some(tile) = map_instance.get((row, column)) else {
                continue;
            };
            let sprite_info = if tile.occupant.is_none() {
                scene.walkable_sprite.as_ref()
            } else {
                scene.attackable_sprite.as_ref()
            };
            if let Some((sprite_info, texture_id)) = sprite_info
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
