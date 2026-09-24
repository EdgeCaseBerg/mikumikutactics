use crate::GameResult;
use crate::tactics::units::*;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;

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
    pub fn path_between(
        &self,
        unit_id: UnitId,
        from: Coordinate,
        to: Coordinate,
    ) -> GameResult<Vec<Coordinate>> {
        // djikstra basically but also https://en.wikipedia.org/wiki/A*_search_algorithm
        let mut open_set = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = HashMap::new();
        let mut f_score = HashMap::new();
        const INFINITY: isize = isize::MAX;
        open_set.push(from);
        g_score.insert(from, 0);
        f_score.insert(from, 0);

        while let Some(current) = open_set.pop() {
            if current == to {
                // construct the path
                break;
            }

            let (r, c) = current;
            eprintln!("look at {:?}", current);
            for neighbor in [(r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)] {
                let Some(node) = self.get(neighbor) else {
                    continue;
                };
                let cost = if node.is_walkable { 0 } else { INFINITY };
                // TODO: Check for occupied nodes we have to walk around
                // since other faction units can't be traversed.

                let h = manhatten(current, neighbor);
                let g = g_score.get(&current).unwrap_or(&INFINITY);
                let g_neighbor = g_score.get(&neighbor).unwrap_or(&INFINITY);
                let tentative_g_score = g.saturating_add(h).saturating_add(cost);
                eprintln!("{:?}", (tentative_g_score, g));
                if tentative_g_score < *g_neighbor {
                    came_from
                        .entry(neighbor)
                        .and_modify(|node| *node = current)
                        .or_insert(current);
                    g_score
                        .entry(neighbor)
                        .and_modify(|v| *v = tentative_g_score)
                        .or_insert(tentative_g_score);
                    if open_set.iter().find(|v| **v == neighbor).is_none() {
                        open_set.push(neighbor);
                    }
                }
            }
        }
        let mut path: VecDeque<Coordinate> = VecDeque::new();
        path.push_front(to);
        let mut current = to;
        while let Some(prev) = came_from.get(&current) {
            path.push_front(*prev);
            current = *prev;
        }
        Ok(path.into_iter().collect())
    }
}

pub fn manhatten(p1: (isize, isize), p2: (isize, isize)) -> isize {
    (p1.0 - p2.0).abs() + (p1.1 - p2.1).abs()
}

#[cfg(test)]
mod map_unit_tests {
    use super::*;

    #[test]
    fn map_search_finds_path() {
        let m = MapInstance::empty_walkable(10, 10);
        let path = m.path_between(UnitId(0), (0, 0), (9, 9));
        let expected_path = vec![
            (0, 0),
            (1, 0),
            (2, 0),
            (3, 0),
            (4, 0),
            (5, 0),
            (6, 0),
            (7, 0),
            (8, 0),
            (9, 0),
            (9, 1),
            (9, 2),
            (9, 3),
            (9, 4),
            (9, 5),
            (9, 6),
            (9, 7),
            (9, 8),
            (9, 9),
        ];
        if let Ok(path) = path {
            assert_eq!(expected_path, path);
        } else {
            assert!(false, "Expected an Ok, got a {:?}", path);
        }
    }

    #[test]
    fn takes_the_optimal_path() {
        let m = MapInstance::empty_walkable(10, 10);
        let path = m.path_between(UnitId(0), (3, 3), (3, 8));
        let expected_path = vec![(3, 3), (3, 4), (3, 5), (3, 6), (3, 7), (3, 8)];
        if let Ok(path) = path {
            assert_eq!(expected_path, path);
        } else {
            assert!(false, "Expected an Ok, got a {:?}", path);
        }
    }
}
