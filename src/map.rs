use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::opentibia::Position;
use crate::opentibia::map::Item;

#[derive(Debug, Default)]
pub struct Map {
    sectors: HashMap<Position, Sector>,
}

#[derive(Debug)]
pub struct Sector {
    pub origin: Position,
    pub tiles: Vec<Vec<Item>>,
}

impl Map {
    pub fn new() -> Map {
        Map { ..Default::default() }
    }

    pub fn get(&self, pos: &Position) -> Option<&Sector> {
        let sector_pos = Sector::get_sector_pos(pos);
        self.sectors.get(&sector_pos)
    }

    pub fn get_mut(&mut self, pos: &Position) -> Option<&mut Sector> {
        let sector_pos = Sector::get_sector_pos(pos);
        self.sectors.get_mut(&sector_pos)
    }

    pub fn get_or_create(&mut self, pos: &Position) -> &mut Sector {
        let sector_pos = Sector::get_sector_pos(pos);

        match self.sectors.entry(sector_pos) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(Sector::new(sector_pos))
        }
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

        Sector {
            origin: origin,
            tiles: tiles,
        }
    }

    fn get_sector_pos(pos: &Position) -> Position {
        Position {
            x: pos.x & !(Sector::SIZE - 1),
            y: pos.y & !(Sector::SIZE - 1),
            z: pos.z,
        }
    }

    pub fn get_tile(&mut self, pos: &Position) -> &mut Vec<Item> {
        &mut self.tiles[((pos.x % Sector::SIZE) * Sector::SIZE + (pos.y % Sector::SIZE)) as usize]
    }

    pub fn iter(&self) -> SectorTileIterator {
        SectorTileIterator {
            sector: self,
            index: 0
        }
    }
}

impl<'a> IntoIterator for &'a Sector {
    type IntoIter = SectorTileIterator<'a>;
    type Item = (Position, &'a [Item]);

    fn into_iter(self) -> Self::IntoIter {
        SectorTileIterator {
            sector: self,
            index: 0
        }
    }
}

pub struct SectorTileIterator<'a> {
    sector: &'a Sector,
    index: u16,
}

impl<'a> Iterator for SectorTileIterator<'a> {
    type Item = (Position, &'a [Item]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < Sector::NUM_TILES as u16 {
            let tile_x = self.sector.origin.x + self.index / Sector::SIZE;
            let tile_y = self.sector.origin.y + self.index % Sector::SIZE;

            let pos = Position {
                x: tile_x,
                y: tile_y,
                z: self.sector.origin.z
            };

            let tile = &self.sector.tiles[self.index as usize];
            self.index += 1;

            Some((pos, tile))
        } else {
            None
        }
    }
}
