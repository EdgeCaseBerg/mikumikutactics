use crate::SpriteInfo;
use crate::asset_loader::AssetCatalog;
use crate::constants::{TextureId, WeaponId};
use crate::tactics::map::*;

use std::collections::HashSet;
use std::collections::VecDeque;
use std::ops::RangeInclusive;

#[derive(Debug)]
pub struct UnitSpriteData {
    pub sprite: (SpriteInfo, TextureId),
    pub portrait: (SpriteInfo, TextureId),
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Faction {
    Player,
    Computer,
}

#[derive(Debug)]
pub struct UnitData {
    pub name: String,
    // behavior: UnitBehavior, add this back when we actually need it
    pub sprite_data: UnitSpriteData, // load via name
    pub move_range: usize,
    pub health: usize,
    pub weapon: Option<WeaponId>,
    pub faction: Faction, // should we really put this here or would Faction { player: [usize], computer: [usize]} be better in maps state or something
}

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct UnitId(pub usize);

pub type FromHere = Coordinate;
pub type ToThere = Coordinate;
pub type Attacker = (UnitId, Coordinate);
pub type Defender = (UnitId, Coordinate);

#[derive(Debug)]
pub enum ActionState {
    UnitSelected(UnitId, Coordinate),   // 1. Click a unit
    CommandPending(UnitId, Coordinate), // 2. Click a tile around that unit, ask what to do
    Move(UnitId, FromHere, ToThere),    // 3. If tile empty -> check in range?
    Attack(Attacker, Defender),         // 4. If tile has empty -> check in range?
    ClearAction,
    DrainCommandQueue,         // Process commands not yet applied
    RewindCommandQueue(usize), // Rewind to idx in command log
}

pub fn reachable_by_unit(
    unit_data: &UnitData,
    at: Coordinate,
    map_instance: &MapInstance,
    asset_catalog: &AssetCatalog,
) -> HashSet<(isize, isize)> {
    let range = unit_data.move_range as isize;
    let attack_range = if let Some(weapon) = unit_data
        .weapon
        .and_then(|wid| asset_catalog.id_to_weapon(wid).ok())
    {
        weapon.range
    } else {
        // We could also potentially consider them "unarmed" and do a range of 1
        0..=0
    };
    let (start_row, start_column) = at;
    let mut queue = VecDeque::from([(at, 0)]); // FIFO
    let mut seen = HashSet::new();
    let mut reachable = HashSet::new();
    while let Some((location, distance)) = queue.pop_front() {
        if seen.contains(&(location, distance)) {
            continue;
        }
        seen.insert((location, distance));
        let Some(tile) = map_instance.get(location) else {
            continue;
        };
        if !tile.is_walkable {
            continue;
        }
        let manhatten_d = manhatten(at, location);
        if manhatten_d > range {
            continue;
        }
        if distance > range {
            continue;
        }
        let (tr, tc) = location;
        if manhatten_d <= range
            && (tile.occupant.is_none() || attack_range.contains(&(manhatten_d as usize)))
        {
            reachable.insert(location);
        }
        // You cannot stand on top of another unit to walk elsewhere, so don't consider the next frontier
        if tile.occupant.is_some() && location != at {
            continue;
        }
        queue.push_front(((tr + 1, tc), distance + 1));
        queue.push_front(((tr - 1, tc), distance + 1));
        queue.push_front(((tr, tc + 1), distance + 1));
        queue.push_front(((tr, tc - 1), distance + 1));
    }

    // In the case of a weapon having longer range than a unit's move
    // we also need to work backwards from those tiles.
    for attacker_position in reachable.clone().iter() {
        let locations =
            destination_can_be_attacked_from(&attack_range, *attacker_position, map_instance);
        for location in locations {
            let (row, column) = location;
            if start_row == row && start_column == column {
                continue;
            }
            reachable.insert(location);
        }
    }

    reachable
}

pub fn destination_can_be_attacked_from(
    attack_range: &RangeInclusive<usize>,
    attacker_position: Coordinate,
    map_instance: &MapInstance,
) -> HashSet<(isize, isize)> {
    let mut reachable = HashSet::new();
    let (dr, dc) = attacker_position;
    let max_attack_range = *attack_range.end() as isize;
    let min_attack_range = *attack_range.start() as isize;
    for row in dr - max_attack_range..=dr + max_attack_range {
        for column in dc - max_attack_range..=dc + max_attack_range {
            let location = (row, column);
            if dr == row && dc == column {
                continue;
            }
            let Some(tile) = map_instance.get(location) else {
                continue;
            };
            // Only an occupied tile can be attacked
            if tile.occupant.is_none() {
                continue;
            }
            let manhatten_d = manhatten(location, attacker_position);
            if manhatten_d <= max_attack_range && manhatten_d >= min_attack_range {
                reachable.insert(location);
            }
        }
    }
    reachable
}

pub fn defender_can_be_attacked_from(
    attack_range: &RangeInclusive<usize>,
    attacker_position: Coordinate,
    defender_position: Coordinate,
    map_instance: &MapInstance,
) -> HashSet<(isize, isize)> {
    let mut reachable = HashSet::new();
    let (dr, dc) = defender_position;
    let max_attack_range = *attack_range.end() as isize;
    let min_attack_range = *attack_range.start() as isize;
    for row in dr - max_attack_range..=dr + max_attack_range {
        for column in dc - max_attack_range..=dc + max_attack_range {
            let location = (row, column);
            if dr == row && dc == column {
                continue;
            }
            let Some(tile) = map_instance.get(location) else {
                continue;
            };
            let manhatten_d = manhatten(defender_position, location);
            if attacker_position == (row, column)
                && manhatten_d <= max_attack_range
                && manhatten_d >= min_attack_range
            {
                reachable.insert(attacker_position);
            }
            // Only an empty tile can be moved into for when we attack
            // unless it's the above case where we're already in position and therefore occupy the space
            if tile.occupant.is_some() {
                continue;
            }
            if manhatten_d <= max_attack_range && manhatten_d >= min_attack_range {
                reachable.insert(location);
            }
        }
    }

    reachable
}

pub fn choose_tile_to_attack_from(
    unit_data: &UnitData,
    at: Coordinate,
    target: Coordinate,
    map_instance: &MapInstance,
    asset_catalog: &AssetCatalog,
) -> Option<(isize, isize)> {
    let attack_range = if let Some(weapon) = unit_data
        .weapon
        .and_then(|wid| asset_catalog.id_to_weapon(wid).ok())
    {
        weapon.range
    } else {
        return None;
    };
    let mut engage_points = defender_can_be_attacked_from(&attack_range, at, target, map_instance);
    let mut reachable = reachable_by_unit(unit_data, at, map_instance, asset_catalog);
    // 'reachable' considers an attackable unit in range as a reachable tile, but you obviously
    // can't stand on top of a target when attacking.
    reachable.remove(&target);
    // we can only insert ourselves if the weapon range allows for it (minimum > 1 means stand away)
    if attack_range.contains(&(manhatten(at, target) as usize)) {
        reachable.insert(at);
        engage_points.insert(at);
    }
    let mut potential: Vec<(isize, isize)> =
        engage_points.intersection(&reachable).cloned().collect();

    // try to find the closet to our current position place we can attack from so a range
    // unit doesn't do something like attack from 2 spaces away when they can attack from 3 instead
    potential.sort_by_key(|p| manhatten(at, *p));
    potential.into_iter().next()
}

pub fn calculate_health_after_attack(
    attacker: &UnitData,
    defender: &UnitData,
    asset_catalog: &AssetCatalog,
) -> usize {
    // TODO: include armor or similar such things once we have
    let attacker_weapon_power = attacker
        .weapon
        .and_then(|wid| asset_catalog.id_to_weapon(wid).ok())
        .map(|weapon| weapon.attack)
        .unwrap_or(0);
    let health = defender.health;
    health.saturating_sub(attacker_weapon_power)
}

#[cfg(test)]
mod tactics_units {
    use super::*;

    fn fake_unit(faction: Faction) -> UnitData {
        UnitData {
            name: "".to_owned(),
            sprite_data: fake_sprite_data(),
            health: 10,
            move_range: 2,
            weapon: None,
            faction,
        }
    }

    fn fake_catalog() -> AssetCatalog {
        let catalog = AssetCatalog::from_manifest_str(
            r#"{
            "spritesheets": [],
            "sounds": [],
            "music": [],
            "scenes": [],
            "weapons": [{
                "id": 0,
                "name": "Sword",
                "attack": 1,
                "min_range": 1,
                "max_range": 1
            }, {
                "id": 1,
                "name": "Bow",
                "attack": 1,
                "min_range": 2,
                "max_range": 2
            }, {
                "id": 2,
                "name": "Long Bow",
                "attack": 1,
                "min_range": 2,
                "max_range": 3
            }]
        }"#,
        );
        let Ok(catalog) = catalog else {
            panic!("expected to parse manifest but got {:?}", catalog);
        };
        catalog
    }

    fn fake_sprite_data() -> UnitSpriteData {
        let s = SpriteInfo {
            current_frame: 0,
            delta: 0,
            framerate_per_second: 0,
            frames: 0,
            height: 0,
            start_x: 0,
            start_y: 0,
            width: 1,
        };
        UnitSpriteData {
            portrait: (s.clone(), TextureId(0)),
            sprite: (s.clone(), TextureId(0)),
        }
    }

    #[rustfmt::skip]
    fn center_star_2() -> HashSet<(isize, isize)> {
        HashSet::from(
            [
                           (5,3),
                    (4,4), (5,4), (6,4),
            (3,5),  (4,5), (5,5), (6,5), (7,5),
                    (4,6), (5,6), (6,6),
                           (5,7),
            ],
        )
    }

    #[rustfmt::skip]
    fn center_star_3() -> HashSet<(isize, isize)> {
        HashSet::from(
            [
                                  (5,2),
                           (4,3), (5,3), (6,3),
                    (3,4), (4,4), (5,4), (6,4), (7,4),
            (2, 5), (3,5), (4,5), (5,5), (6,5), (7,5), (8,5),
                    (3,6), (4,6), (5,6), (6,6), (7,6),
                           (4,7), (5,7), (6,7),
                                  (5,8)
            ],
        )
    }

    #[rustfmt::skip]
    fn bl_corner() -> HashSet<(isize, isize)> {
        HashSet::from(
            [
                (0,7),
                (0,8), (1,8),
                (0,9), (1, 9), (2, 9),
            ],
        )
    }

    #[rustfmt::skip]
    fn tl_corner() -> HashSet<(isize, isize)> {
        HashSet::from(
            [
                (0,0), (0,1), (0,2),
                (1,0), (1,1),
                (2,0),
            ],
        )
    }

    #[test]
    fn reachable_by_unit_should_work_properly_on_fully_walkable_map() {
        let m = MapInstance::empty_walkable(10, 10);
        let u = fake_unit(Faction::Player);
        let c = fake_catalog();

        eprintln!("in center 2");
        let reachable = reachable_by_unit(&u, (5, 5), &m, &c);
        let expected = center_star_2();
        assert_eq!(expected, reachable);

        eprintln!("bottom corner left");
        let reachable = reachable_by_unit(&u, (0, 9), &m, &c);
        let expected = bl_corner();
        assert_eq!(expected, reachable);

        let mut u = fake_unit(Faction::Player);
        u.move_range = 3;
        let reachable = reachable_by_unit(&u, (5, 5), &m, &c);
        let expected = center_star_3();
        eprintln!("in center 3 {:?}", {
            (
                reachable.difference(&expected).collect::<Vec<_>>(),
                expected.difference(&reachable).collect::<Vec<_>>(),
            )
        });
        assert_eq!(expected, reachable);
    }

    #[test]
    fn reachable_by_unit_should_work_properly_on_partially_walkable_map() {
        let mut m = MapInstance::empty_walkable(10, 10);
        m.get_mut((4, 4)).unwrap().is_walkable = false;
        m.get_mut((0, 8)).unwrap().is_walkable = false;

        let u = fake_unit(Faction::Player);
        let c = fake_catalog();

        eprintln!("in center");
        let reachable = reachable_by_unit(&u, (5, 5), &m, &c);
        let mut expected = center_star_2();
        expected.remove(&(4, 4));
        assert_eq!(expected, reachable);

        eprintln!("bottom corner left");
        let reachable = reachable_by_unit(&u, (0, 9), &m, &c);
        let mut expected = bl_corner();
        expected.remove(&(0, 8));
        expected.remove(&(0, 7)); // can't walk around and still be in move range
        assert_eq!(expected, reachable);
    }

    #[test]
    fn reachable_by_unit_should_not_let_one_walk_through_enemies() {
        // this will probably be a pain when we do factions...
        let mut m = MapInstance::empty_walkable(10, 10);
        m.get_mut((0, 0)).unwrap().occupant = Some(UnitId(0));
        m.get_mut((0, 1)).unwrap().occupant = Some(UnitId(1));

        let c = fake_catalog();
        let mut u = fake_unit(Faction::Player);
        u.weapon = Some(WeaponId(0)); // sword with range 1
        u.move_range = 2;

        eprintln!("cant walk by enemy");
        let reachable = reachable_by_unit(&u, (0, 0), &m, &c);
        let mut expected = tl_corner();
        expected.remove(&(0, 0)); // we are standing at 0,0
        expected.remove(&(0, 2)); // enemy is at 0,1 can't walk through
        assert_eq!(expected, reachable);
    }

    #[test]
    fn able_to_attack_unit_within_range_of_weapon() {
        let mut m = MapInstance::empty_walkable(10, 10);
        m.get_mut((4, 4)).unwrap().occupant = Some(UnitId(1));

        let mut u = fake_unit(Faction::Player);
        u.weapon = Some(WeaponId(1)); // Bow with range 2..2

        let c = fake_catalog();

        eprintln!("should be able to hit");
        let reachable = reachable_by_unit(&u, (5, 5), &m, &c);
        let expected = center_star_2();
        assert_eq!(expected, reachable);

        eprintln!("should not be able to hit");
        m.get_mut((4, 4)).unwrap().occupant = None;
        m.get_mut((4, 5)).unwrap().occupant = Some(UnitId(1));
        for walled in &[(4, 4), (5, 4), (6, 4), (6, 5), (4, 6), (5, 6), (6, 6)] {
            m.get_mut(*walled).unwrap().is_walkable = false;
        }
        // and shouldn't be able to hit the enemy standing 1 away from us with a bow of range 2..2
        // if we can't walk around them at all to get a position, we also cant move past the enemy
        // so really the only option we have is to wait and get hit
        let expected = HashSet::from([(5, 5)]);
        let reachable = reachable_by_unit(&u, (5, 5), &m, &c);
        eprintln!("in center 2 {:?}", {
            (
                reachable.difference(&expected).collect::<Vec<_>>(),
                expected.difference(&reachable).collect::<Vec<_>>(),
            )
        });
        assert_eq!(expected, reachable);
    }

    #[test]
    fn able_to_attack_unit_within_range_of_weapon_outside_of_move_range() {
        let mut m = MapInstance::empty_walkable(10, 10);
        m.get_mut((8, 5)).unwrap().occupant = Some(UnitId(1));

        let mut u = fake_unit(Faction::Player);
        u.weapon = Some(WeaponId(2)); // Long Bow with range 2..3

        let c = fake_catalog();

        eprintln!("should be able to hit vertically");
        let reachable = reachable_by_unit(&u, (5, 5), &m, &c);
        let mut expected = center_star_2();
        expected.insert((8, 5));
        eprintln!("in center 2 {:?}", {
            (
                reachable.difference(&expected).collect::<Vec<_>>(),
                expected.difference(&reachable).collect::<Vec<_>>(),
            )
        });
        assert_eq!(expected, reachable);

        eprintln!("should be able to hit horizontal");
        // Just outside the move range
        m.get_mut((5, 2)).unwrap().occupant = Some(UnitId(1));
        m.get_mut((2, 5)).unwrap().occupant = Some(UnitId(2));
        m.get_mut((8, 5)).unwrap().occupant = Some(UnitId(3));
        m.get_mut((5, 8)).unwrap().occupant = Some(UnitId(4));
        // Only attackable via move then attack:
        m.get_mut((7, 8)).unwrap().occupant = Some(UnitId(4));
        let reachable = reachable_by_unit(&u, (5, 5), &m, &c);
        let mut expected = center_star_2();

        expected.insert((5, 2));
        expected.insert((2, 5));
        expected.insert((8, 5));
        expected.insert((5, 8));
        expected.insert((7, 8));
        eprintln!("in center 2 {:?}", {
            (
                reachable.difference(&expected).collect::<Vec<_>>(),
                expected.difference(&reachable).collect::<Vec<_>>(),
            )
        });
        assert_eq!(expected, reachable);
    }

    #[test]
    fn attackable_from_verify_min_range() {
        let m = MapInstance::empty_walkable(10, 10);

        let attackable = defender_can_be_attacked_from(&(0..=0), (5, 6), (5, 5), &m);
        assert_eq!(attackable.into_iter().count(), 0);

        let attackable = defender_can_be_attacked_from(&(1..=1), (5, 6), (5, 5), &m);
        let expected = HashSet::from([(5, 4), (5, 6), (6, 5), (4, 5)]);
        assert_eq!(attackable, expected, "{:?}", {
            (
                attackable.difference(&expected).collect::<Vec<_>>(),
                expected.difference(&attackable).collect::<Vec<_>>(),
            )
        });

        let attackable = defender_can_be_attacked_from(&(2..=2), (5, 7), (5, 5), &m);
        let expected = HashSet::from([
            (5, 3),
            (4, 4),
            (6, 4),
            (3, 5),
            (7, 5),
            (4, 6),
            (6, 6),
            (5, 7),
        ]);
        assert_eq!(attackable, expected);

        let attackable = defender_can_be_attacked_from(&(1..=3), (5, 6), (5, 5), &m);
        let mut expected = center_star_3();
        expected.remove(&(5, 5));
        assert_eq!(attackable, expected, "{:?}", {
            (
                attackable.difference(&expected).collect::<Vec<_>>(),
                expected.difference(&attackable).collect::<Vec<_>>(),
            )
        });
    }

    #[test]
    fn move_into_range_to_attack_moves_to_close_tile() {
        let mut m = MapInstance::empty_walkable(10, 10);
        let target = (8, 5);
        let unit = (5, 5);
        m.get_mut(target).unwrap().occupant = Some(UnitId(1));
        m.get_mut(unit).unwrap().occupant = Some(UnitId(0));

        let mut u = fake_unit(Faction::Player);
        u.weapon = Some(WeaponId(0)); // Sword with range 1..=1

        let c = fake_catalog();
        let choice = choose_tile_to_attack_from(&u, unit, target, &m, &c);
        eprintln!("should choose a walkable target with 1 range");
        assert_eq!(Some((7, 5)), choice);

        let mut u = fake_unit(Faction::Player);
        u.weapon = Some(WeaponId(1)); // Bow with range 2..=2
        let choice = choose_tile_to_attack_from(&u, unit, target, &m, &c);
        eprintln!("should choose a walkable target with 2 range");
        assert_eq!(Some((6, 5)), choice);

        let mut u = fake_unit(Faction::Player);
        u.weapon = Some(WeaponId(2)); // Bow with range 2..=3
        let choice = choose_tile_to_attack_from(&u, unit, target, &m, &c);
        eprintln!("should not bother moving when its already in range");
        assert_eq!(Some((5, 5)), choice);

        let mut u = fake_unit(Faction::Player);
        u.weapon = Some(WeaponId(1)); // Bow with range 2..=2
        let mut m = MapInstance::empty_walkable(10, 10);
        let target = (8, 5);
        let unit = (7, 5);
        m.get_mut(target).unwrap().occupant = Some(UnitId(1));
        m.get_mut(unit).unwrap().occupant = Some(UnitId(0));
        let choice = choose_tile_to_attack_from(&u, unit, target, &m, &c);
        eprintln!("should move if minimum range not satisfied");
        let valid_moves = HashSet::from([(7, 4), (6, 5), (7, 6)]);
        assert!(choice.is_some());
        assert!(valid_moves.contains(&choice.unwrap()));
    }
}
