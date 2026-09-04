use crate::SpriteInfo;
use crate::constants::TextureId;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub enum UnitState {
    Idle,
    Selected,
}

#[derive(Debug, Clone, Copy)]
pub enum Actions {
    Select,
    Deselect,
}

// #[derive(Debug)]
// pub struct UnitBehavior {
//     pub state: UnitState,
// }

// impl UnitBehavior {
//     fn act(&mut self, input: Actions) {
//         match (self.state, input) {
//             (UnitState::Idle, Actions::Select) => {
//                 self.state = UnitState::Selected;
//             }
//             (UnitState::Selected, Actions::Deselect) => {
//                 self.state = UnitState::Idle;
//             }
//             _ => {}
//         }
//     }
//     fn current_state(&self) -> UnitState {
//         self.state
//     }
// }

#[derive(Debug)]
pub struct UnitSpriteData {
    pub sprite: (SpriteInfo, TextureId),
    pub portrait: (SpriteInfo, TextureId),
}

#[derive(Debug)]
pub struct UnitData {
    pub name: String,
    // behavior: UnitBehavior, add this back when we actually need it
    pub sprite_data: UnitSpriteData, // load via name
}

#[derive(PartialEq, Copy, Debug, Clone, Hash, Eq)]
pub struct UnitId(pub usize);

#[derive(Debug)]
pub struct Tile {
    pub is_walkable: bool,
    pub occupant: Option<UnitId>,
}

#[derive(Debug)]
pub struct MapInstance {
    pub width: isize,
    pub height: isize,
    pub tiles: Vec<Tile>,
}

impl MapInstance {
    pub fn empty_walkable(width: isize, height: isize) -> Self {
        let tiles = (0..height)
            .flat_map(|_row| {
                (0..width).map(|_column| Tile {
                    is_walkable: true,
                    occupant: None,
                })
            })
            .collect();
        MapInstance {
            width,
            height,
            tiles,
        }
    }
    pub fn get(&self, coordinate: Coordinate) -> Option<&Tile> {
        let (row, column) = coordinate;
        if row >= self.height || row < 0 || column < 0 || column >= self.width {
            return None;
        }
        let idx = row * self.width + column;
        self.tiles.get(idx as usize)
    }
    pub fn get_mut(&mut self, coordinate: Coordinate) -> Option<&mut Tile> {
        let (row, column) = coordinate;
        if row >= self.height || row < 0 || column < 0 || column >= self.width {
            return None;
        }
        let idx = row * self.width + column;
        self.tiles.get_mut(idx as usize)
    }
}

pub type Coordinate = (isize, isize);
pub type FromHere = Coordinate;
pub type ToThere = Coordinate;
pub type Attacker = (UnitId, Coordinate);
pub type Defender = (UnitId, Coordinate);

pub enum ActionState {
    UnitSelected(UnitId, Coordinate),   // 1. Click a unit
    CommandPending(UnitId, Coordinate), // 2. Click a tile around that unit, ask what to do
    Move(UnitId, FromHere, ToThere),    // 3. If tile empty -> check in range?
    Attack(Attacker, Defender),         // 4. If tile has empty -> check in range?
}

pub struct MapState {
    pub units: HashMap<UnitId, UnitData>,
    pub selectable_tiles: HashSet<(isize, isize)>,
    pub action: Option<ActionState>,
}
