use crate::GameResult;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct WeaponId(pub usize);

#[derive(Debug, Clone)]
pub struct Weapon {
    pub name: String,
    pub range: usize,
    pub attack: usize,
    // TODO once initial game is good and we want to build more on: other properties as we build, cost, crit rate, durability, damage types, weight, status effects
}

pub fn dummy_sword() -> (WeaponId, Weapon) {
    (
        WeaponId(0),
        Weapon {
            name: "Sword".to_owned(),
            range: 1,
            attack: 1,
        },
    )
}

// What's interesting to think of, is that in FE there is min/max range since most Bows require at least a space between units to use
pub fn dummy_bow() -> (WeaponId, Weapon) {
    (
        WeaponId(1),
        Weapon {
            name: "Bow".to_owned(),
            range: 2,
            attack: 1,
        },
    )
}

trait WeaponCatalog {
    fn name_to_weapon(&self, name: &str) -> GameResult<Weapon>;
    fn id_to_weapon(&self, id: WeaponId) -> GameResult<Weapon>;
}

struct DummyWeaponCatalog {
    pub weapons: [(WeaponId, Weapon)],
}

// impl WeaponCatalog for DummyWeaponCatalog {
//     fn name_to_weapon(name: &str) -> GameResult<Weapon> {
//         self.weapons.find(|(id, w)| w.name == name).ok()
//     }
//     fn id_to_weapon(id: WeaponId) -> GameResult<Weapon> {
//         self.weapons.find(|(id, w)| id == id).ok()
//     }
// }
