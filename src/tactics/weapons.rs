use crate::GameResult;
use std::collections::HashMap;
use std::ops::RangeInclusive;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WeaponId(pub usize);

#[derive(Debug, Clone)]
pub struct Weapon {
    pub name: String,
    pub range: RangeInclusive<usize>,
    pub attack: usize,
    // TODO once initial game is good and we want to build more on: other properties as we build, cost, crit rate, durability, damage types, weight, status effects
}

pub fn dummy_sword() -> (WeaponId, Weapon) {
    (
        WeaponId(0),
        Weapon {
            name: "Sword".to_owned(),
            range: 1..=1,
            attack: 1,
        },
    )
}

pub fn dummy_bow() -> (WeaponId, Weapon) {
    (
        WeaponId(1),
        Weapon {
            name: "Bow".to_owned(),
            range: 2..=3,
            attack: 1,
        },
    )
}

pub trait WeaponCatalog {
    fn name_to_weapon(&self, name: &str) -> GameResult<Weapon>;
    fn id_to_weapon(&self, id: WeaponId) -> GameResult<Weapon>;
}

pub struct DummyWeaponCatalog {
    pub weapons: Vec<(WeaponId, Weapon)>,
}

impl Default for DummyWeaponCatalog {
    fn default() -> DummyWeaponCatalog {
        DummyWeaponCatalog {
            weapons: vec![dummy_sword(), dummy_bow()],
        }
    }
}

impl WeaponCatalog for DummyWeaponCatalog {
    fn name_to_weapon(&self, name: &str) -> GameResult<Weapon> {
        if let Some((_, weapon)) = self.weapons.iter().find(|(id, w)| w.name == name) {
            Ok(weapon.clone())
        } else {
            Err(format!("could not find weapon with name {}", name).into())
        }
    }
    fn id_to_weapon(&self, id: WeaponId) -> GameResult<Weapon> {
        if let Some((_, weapon)) = self.weapons.iter().find(|(wid, w)| *wid == id) {
            Ok(weapon.clone())
        } else {
            Err(format!("could not find weapon with name {}", id.0).into())
        }
    }
}
