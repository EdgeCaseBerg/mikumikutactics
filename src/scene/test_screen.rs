use crate::GameResult;
use crate::Rect;
use crate::Scene;
use crate::SpriteInfo;
use crate::asset_loader::{AssetCatalog, AssetLoader, ensure_texture_by_name};
use crate::constants::{FontId, TEXTURE_UI};
use crate::constants::{TextureId, WeaponId};
use crate::game::GameContext;
use crate::grid_layout::GridLayout;
use crate::manifest::TerrainMap;
use crate::renderer::RenderCommand;
use crate::renderer::Renderer;
use crate::scene::button::{Button, new_btn};
use crate::tactics::commands::*;
use crate::tactics::map::*;
use crate::tactics::units::*;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default)]
pub struct TestScene {
    focus_sprite: Option<(SpriteInfo, TextureId)>,
    walkable_sprite: Option<(SpriteInfo, TextureId)>,
    attackable_sprite: Option<(SpriteInfo, TextureId)>,
    danger_sprite: Option<(SpriteInfo, TextureId)>,
    current_portrait: Option<(SpriteInfo, TextureId)>,
    current_portrait_bg: Option<(SpriteInfo, TextureId)>,
    terrain: Option<(TerrainMap, HashMap<String, (SpriteInfo, TextureId)>)>,
    btn_prototype: Option<Button>,
    map: Option<BattleData>,
    context_menu: Option<ContextMenu>,
    command_log: CommandLog,
    undo_buttons: Vec<(Button, usize)>,
    quit_btn: Option<Button>,
    continue_btn: Option<Button>,
    retry_btn: Option<Button>,
}

// TODO: Refactor to use list of buttons and tuple of (button, command) instead since Err for cancel can be nixed for clear
// although commands might not work because of the whole selectable thing unless we push some of the scene state down into the
// command itself or something weird. Tricky, but we should figure it out since that'll make it easier to make menus and we can
// move the rewind menu into a context menu or something similar.
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
    fn compute_undo_buttons(&self, anchor: Rect) -> Vec<(Button, usize)> {
        let actions = self.command_log.get_unit_actions();
        let strings = actions.map(|unit_action| {
            let name = self.map.as_ref().and_then(|battle_data| {
                battle_data
                    .map_state
                    .units
                    .get(&unit_action.unit_id)
                    .map(|unit| unit.name.clone())
            });
            let action = self
                .command_log
                .get_command_at(unit_action.end_offset)
                .map(|cmd| cmd.description());
            eprintln!("DEBUG {:?}", (&name, &action, unit_action.end_offset));
            (
                format!(
                    "{} {}",
                    name.expect("name to exist"),
                    action.expect("action to exist")
                ),
                unit_action.start_offset,
            )
        });
        let mut y_offset = 0;
        let mut next_rect = || {
            let mut rect = anchor;
            rect.y += y_offset;
            y_offset += 1;
            rect
        };
        eprintln!("compute_undo_buttons! {:?}", (&strings));
        strings
            .filter_map(|(btn_text, cmd_offset)| {
                new_btn(&self.btn_prototype, btn_text, next_rect()).map(|btn| (btn, cmd_offset))
            })
            .collect()
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

fn make_map(asset_loader: &dyn AssetLoader, terrain_map: &str) -> GameResult<MapInstance> {
    let catalog = asset_loader.get_catalog();
    let map = catalog.name_to_terrain_map(terrain_map)?;
    MapInstance::from_terrain(asset_loader, &map)
}

fn make_terrain(
    asset_loader: &mut dyn AssetLoader,
    terrain_map: &str,
) -> GameResult<(TerrainMap, HashMap<String, (SpriteInfo, TextureId)>)> {
    let catalog = asset_loader.get_catalog();
    let map = catalog.name_to_terrain_map(terrain_map)?;
    let mut hashmap = HashMap::new();
    for (k, v) in map.tile_definitions.iter() {
        let tuple = sprite_tuple(&map.sheet, Some(&[&v.tag_name]), asset_loader)?;
        hashmap.insert(k.clone(), tuple);
    }
    Ok((map, hashmap))
}

impl Scene for TestScene {
    fn init(&mut self, game_context: &mut GameContext) -> GameResult<()> {
        let Some(asset_loader) = game_context.asset_loader.as_mut() else {
            return Err("cannot init scene if asset_loader is not set".into());
        };

        let terrain_map_name = "grassland";
        let mut the_map = make_map(asset_loader.as_ref(), "grassland")?;
        self.terrain = Some(make_terrain(asset_loader.as_mut(), "grassland")?);

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
                        health: 3,
                        sprite_data: UnitSpriteData {
                            sprite: s,
                            portrait: p,
                        },
                        move_range: 3,
                        weapon: Some(WeaponId(uid % 2)),
                        faction: if uid % 2 == 0 {
                            Faction::Player
                        } else {
                            Faction::Computer
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
        self.map = Some(BattleData {
            untapped_units: units_of(&map_state, Faction::Player),
            map_instance: the_map,
            map_state,
            active_faction: Faction::Player,
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
            if btn.clicked
                && let Some(battle_data) = self.map.as_mut()
            {
                battle_data.untapped_units = units_of(&battle_data.map_state, Faction::Player);
            }
        }
        for (btn, rewind_to) in &mut self.undo_buttons {
            let layout = TestScene::layout(game_context);
            btn.update(ticks, game_context, &layout);
            if btn.clicked {
                // trigger rewind
                eprintln!("rewind to offset...");
                if let Some(battle_data) = self.map.as_mut() {
                    battle_data.map_state.action =
                        Some(ActionState::RewindCommandQueue(*rewind_to));
                }
            }
        }
        if let Some(btn) = self.continue_btn.as_mut() {
            let layout = TestScene::layout(game_context);
            btn.update(ticks, game_context, &layout);
            if btn.clicked
                && let Some(battle_data) = self.map.as_mut()
            {
                eprintln!("continue...");
            }
        }
        if let Some(btn) = self.continue_btn.as_mut() {
            let layout = TestScene::layout(game_context);
            btn.update(ticks, game_context, &layout);
            if btn.clicked
                && let Some(battle_data) = self.map.as_mut()
            {
                eprintln!("continue...");
            }
        }
        if let Some(btn) = self.retry_btn.as_mut() {
            let layout = TestScene::layout(game_context);
            btn.update(ticks, game_context, &layout);
            if btn.clicked
                && let Some(battle_data) = self.map.as_mut()
            {
                eprintln!("retry...");
            }
        }
        if let Some(btn) = self.quit_btn.as_mut() {
            let layout = TestScene::layout(game_context);
            btn.update(ticks, game_context, &layout);
            if btn.clicked
                && let Some(battle_data) = self.map.as_mut()
            {
                eprintln!("quit!");
            }
        }

        if let Some(battle_data) = self.map.as_mut() {
            let BattleData {
                map_instance,
                map_state,
                untapped_units,
                active_faction,
            } = battle_data;
            match map_state.action {
                None => {
                    if let Some((action, unit_data, reachable_positions)) =
                        check_if_unit_selected_action(
                            game_context,
                            map_instance,
                            map_state,
                            untapped_units,
                        )
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
                    let path = map_instance.path_between(unit_id, from, to)?;
                    let mut pairs = path.windows(2);
                    let mut commands: Vec<Box<dyn Command>> = vec![];
                    while let Some(&[from, to]) = pairs.next() {
                        let command = MoveCommand { unit_id, from, to };
                        eprintln!("add move command");
                        commands.push(Box::new(command));
                    }
                    self.command_log.add_unit_actions(unit_id, commands);
                    // queue up turn change
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

                    match determine_move_to_and_damage(
                        attacker,
                        defender,
                        map_instance,
                        map_state,
                        catalog.as_ref(),
                    ) {
                        None => { /* Weird state. The attacker or defender dont exist */ }
                        Some((Some(required_move), damage)) => {
                            /* Lazy player, requested attack that requires a unit to move first */
                            let path = map_instance.path_between(
                                attacker_uid,
                                attacker_position,
                                required_move,
                            )?;
                            let mut pairs = path.windows(2);
                            let mut commands: Vec<Box<dyn Command>> = vec![];
                            while let Some(&[from, to]) = pairs.next() {
                                let command = MoveCommand {
                                    unit_id: attacker_uid,
                                    from,
                                    to,
                                };
                                eprintln!("add move command");
                                commands.push(Box::new(command));
                            }

                            let command = AttackCommand {
                                defender,
                                damage,
                                attacker_uid,
                            };
                            commands.push(Box::new(command));
                            self.command_log.add_unit_actions(attacker_uid, commands);
                            map_state.action = Some(ActionState::DrainCommandQueue);
                        }
                        Some((None, damage)) => {
                            let command = AttackCommand {
                                defender,
                                damage,
                                attacker_uid,
                            };
                            self.command_log
                                .add_unit_actions(attacker_uid, vec![Box::new(command)]);
                            map_state.action = Some(ActionState::DrainCommandQueue);
                        }
                    }
                }
                Some(ActionState::ClearAction) => {
                    map_state.action = None;
                    map_state.selectable_tiles = HashSet::new();
                    self.context_menu = None;
                    self.current_portrait = None;
                    let anchor = Rect::new(10, 2, 4, 1);
                    self.undo_buttons = self.compute_undo_buttons(anchor);
                }
                Some(ActionState::DrainCommandQueue) => {
                    match self.command_log.update(ticks, battle_data) {
                        Ok(CommandLogState::Exhausted) => {
                            if battle_data.untapped_units.is_empty() {
                                battle_data.map_state.action = Some(ActionState::SwapFaction);
                            } else {
                                battle_data.map_state.action = Some(ActionState::ClearAction);
                            }
                            if let Some(losing_faction) = loser(&battle_data.map_state) {
                                battle_data.map_state.action =
                                    Some(ActionState::GameOver(losing_faction))
                            }
                        }
                        other => {
                            other?;
                        }
                    }
                }
                Some(ActionState::SwapFaction) => {
                    self.command_log
                        .add_command(Box::new(active_faction.swap_command()));
                    map_state.action = Some(ActionState::DrainCommandQueue);
                    // TODO: show text saying we're swapping turns, clear on clear action
                }
                Some(ActionState::RewindCommandQueue(to)) => {
                    map_state.action = Some(ActionState::ClearAction);
                    self.command_log.rewind(to, battle_data)?;
                }
                Some(ActionState::GameOver(losing_faction)) => {
                    let mut y_offset = 0;
                    let anchor =
                        Rect::new(map_instance.width / 2 - 1, map_instance.height / 2, 3, 1);
                    let mut next_rect = || {
                        let mut rect = anchor;
                        rect.y += y_offset;
                        y_offset += 1;
                        rect
                    };
                    match losing_faction {
                        Faction::Computer => {
                            self.continue_btn =
                                new_btn(&self.btn_prototype, "Continue".to_owned(), next_rect());
                        }
                        Faction::Player => {
                            self.retry_btn =
                                new_btn(&self.btn_prototype, "Retry".to_owned(), next_rect());
                        }
                    }
                    self.quit_btn = new_btn(&self.btn_prototype, "Quit".to_owned(), next_rect());
                    map_state.action = Some(ActionState::ClearAction);
                }
            }
        };

        if let Some((_, terrain_sprites)) = self.terrain.as_mut() {
            for (_, (sprite_info, _)) in terrain_sprites.into_iter() {
                sprite_info.advance(ticks);
            }
        }
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
        for (btn, _) in &self.undo_buttons {
            btn.draw(game_context, &layout);
        }
        if let Some(btn) = self.continue_btn.as_ref() {
            btn.draw(game_context, &layout);
        }
        if let Some(btn) = self.retry_btn.as_ref() {
            btn.draw(game_context, &layout);
        }
        if let Some(btn) = self.quit_btn.as_ref() {
            btn.draw(game_context, &layout);
        }
        draw_faction_status(self, game_context)?;
        Ok(())
    }
}

fn determine_move_to_and_damage(
    attacker: Attacker,
    defender: Defender,
    map_instance: &MapInstance,
    map_state: &MapState,
    asset_catalog: &AssetCatalog,
) -> Option<(Option<Coordinate>, usize)> {
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

    let damage = map_state
        .units
        .get(&defender_uid)
        .map(|defender| {
            let new_health = calculate_health_after_attack(attacker, defender, asset_catalog);
            defender.health.saturating_sub(new_health)
        })
        .unwrap_or(0);

    Some((move_to, damage))
}

fn check_if_unit_selected_action<'a>(
    game_context: &GameContext,
    map_instance: &MapInstance,
    map_state: &'a MapState,
    untapped_units: &HashSet<UnitId>,
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
        // Clicking on a spent unit does nothing.
        if tile
            .occupant
            .is_some_and(|uid| !untapped_units.contains(&uid))
        {
            return None;
        }
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
        ..
    }) = scene.map.as_ref()
    else {
        return;
    };

    let Some((terrain_map, terrain_sprite_map)) = scene.terrain.as_ref() else {
        return;
    };
    let mut tile_key = terrain_map.grid.chars().filter(|c| !c.is_whitespace());
    let mut key_buf = [0; 4];
    for row in 0..map_instance.height {
        for column in 0..map_instance.width {
            let Some(tile) = map_instance.get((row, column)) else {
                continue;
            };
            let tile_key = tile_key
                .next()
                .expect("loaded terrain does not align to map");
            // Background texture
            let key = tile_key.encode_utf8(&mut key_buf);
            if let Some((sprite_info, texture_id)) = terrain_sprite_map.get(key) {
                let src = sprite_info.get_rect();
                let dst = layout.cell_rect(row as usize, column as usize);
                renderer.send_command(RenderCommand::DrawRect {
                    texture_id: *texture_id,
                    source: src,
                    destination: dst,
                });
            }

            // Occupant
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

fn draw_faction_status(scene: &mut TestScene, game_context: &mut GameContext) -> GameResult<()> {
    let layout = TestScene::layout(game_context);
    let Some(renderer) = game_context.renderer.as_mut() else {
        return Err("no renderer defined in game context, not faction status.".into());
    };
    let Some(font_id) = scene.btn_prototype.as_ref().map(|b| b.font_id) else {
        return Err("no font id to use for displaying faction status.".into());
    };
    let Some(BattleData {
        map_instance,
        untapped_units,
        active_faction,
        ..
    }) = scene.map.as_ref()
    else {
        return Err("no battle data defined to display faction status.".into());
    };
    let anchor = layout.cell_rect(1, map_instance.width as usize - 5);
    let units_left = untapped_units.len();
    let faction = active_faction.to_string();

    if let Some(font_library) = game_context.font_library.as_mut() {
        if let Some(font) = font_library.get(font_id) {
            crate::font::draw_text_anchored_with_bg(
                &format!("{}: {} left", faction, units_left),
                &anchor,
                font,
                renderer.as_mut(),
                crate::font::FontAlign::Left,
                crate::renderer::Color::white(),
            );
        }
    }
    Ok(())
}
