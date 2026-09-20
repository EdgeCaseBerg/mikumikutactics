use crate::GameResult;
use crate::tactics::units::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub type Coordinate = (isize, isize);

#[derive(Debug)]
pub struct MapInstance {
    pub width: isize,
    pub height: isize,
    pub tiles: Vec<Tile>,
}

#[derive(Debug)]
pub struct Tile {
    pub is_walkable: bool,
    pub occupant: Option<UnitId>,
}

#[derive(Debug)]
pub struct MapState {
    pub units: HashMap<UnitId, UnitData>,
    pub selectable_tiles: HashSet<(isize, isize)>,
    pub action: Option<ActionState>,
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
    pub fn move_unit_to(
        &mut self,
        unit_id: UnitId,
        from: Coordinate,
        to: Coordinate,
    ) -> GameResult<()> {
        let unit_moving = self.get_mut(from).and_then(|tile| tile.occupant.take());
        if unit_moving.is_none_or(|occupant| occupant != unit_id) {
            return Err(
                format!("unexpected map state, occupant to be moved {:?} did not match target of move command {:?}", unit_moving, unit_id).into()
            );
        }
        if self.get(to).is_none_or(|tile| tile.occupant.is_some()) {
            return Err(format!(
                "destination {:?} is already occupied by {:?}, should have been empty",
                to,
                self.get(to)
            )
            .into());
        }
        if let Some(tile) = self.get_mut(to) {
            tile.occupant = unit_moving;
        } else {
            return Err(format!("destination {:?} was not a valid tile on the map", to).into());
        }
        Ok(())
    }
}
