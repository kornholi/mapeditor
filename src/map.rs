use std::collections::HashMap;

use opentibia::Position;
use opentibia::map::Item;

#[derive(Debug, Default)]
pub struct Map {
    sectors: HashMap<Position, Sector>
}

#[derive(Debug)]
pub struct Sector {
    pub origin: Position,
    pub tiles: Vec<Vec<Item>>
}

impl Map {
    pub fn new() -> Map {
        Map { ..Default::default() }
    }

    pub fn get(&self, pos: Position) -> Option<&Sector> {
        let sector_pos = Position { x: pos.x & !31, y: pos.y & !31, z: pos.z};

        self.sectors.get(&sector_pos)
    }

    pub fn get_mut(&mut self, pos: Position) -> Option<&mut Sector> {
        let sector_pos = Position { x: pos.x & !31, y: pos.y & !31, z: pos.z};

        self.sectors.get_mut(&sector_pos)
    }

    pub fn get_or_create(&mut self, pos: Position) -> &mut Sector {
        let sector_pos = Position { x: pos.x & !31, y: pos.y & !31, z: pos.z};

        if !self.sectors.contains_key(&sector_pos) {
            let sec = Sector::new(sector_pos);
            self.sectors.insert(sector_pos, sec);
        }

        self.sectors.get_mut(&sector_pos).expect("impossible")
    }
}

impl Sector {
    pub const SIZE: u16 = 32;
    const NUM_TILES: usize = (Sector::SIZE * Sector::SIZE) as usize;

    fn new(origin: Position) -> Sector {
        let mut tiles = Vec::with_capacity(Sector::NUM_TILES);

        for _ in 0..Sector::NUM_TILES {
            tiles.push(Vec::new());
        }

        Sector { origin: origin, tiles: tiles }
    }

    pub fn get_tile(&mut self, pos: Position) -> &mut Vec<Item> {
        &mut self.tiles[((pos.x % Sector::SIZE) * Sector::SIZE + (pos.y % Sector::SIZE)) as usize]
    }
}