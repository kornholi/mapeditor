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

	pub fn get(&mut self, pos: Position) -> Option<&mut Sector> {
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
		//return &mut self.sectors[&sector_pos];
	}
}

impl Sector {
	fn new(origin: Position) -> Sector {
		let mut tiles = Vec::with_capacity(32*32);

		for _ in 0..32*32 {
			tiles.push(Vec::new());
		}

		Sector { origin: origin, tiles: tiles }
	}

	pub fn get_tile(&mut self, pos: Position) -> &mut Vec<Item> {
		&mut self.tiles[((pos.x % 32) * 32 + (pos.y % 32)) as usize]
	}
}