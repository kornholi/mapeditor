use std::cmp;
use clock_ticks;

use lru_cache::LruCache;

use datcontainer;
use datcontainer::DatContainer;
use opentibia::{itemtypes, Position};

use super::map;

pub struct Renderer<V> {
    pub dat: DatContainer,
    pub otb: itemtypes::Container,
    pub map: map::Map,

    sector_cache: LruCache<Position, Vec<V>>,
}

fn get_sprite_id(obj: &datcontainer::Thing,
                 layer: u8,
                 pattern_x: u16,
                 pattern_y: u16,
                 x: u8,
                 y: u8)
                 -> usize {
    let animation_time = 0;

    ((((((animation_time % 4095) * obj.pattern_height as u16 +
         pattern_y % obj.pattern_height as u16) * obj.pattern_width as u16 +
        pattern_x % obj.pattern_width as u16) * obj.layers as u16 +
       layer as u16) * obj.height as u16 + y as u16) * obj.width as u16 + x as u16) as usize %
    obj.sprite_ids.len()
}

impl<V> Renderer<V> {
    pub fn new(dat: DatContainer, otb: itemtypes::Container, map: map::Map) -> Renderer<V> {
        Renderer {
            dat: dat,
            otb: otb,
            map: map,

            sector_cache: LruCache::new(512)
        }
    }

    pub fn get_visible_sectors(&self, ul: (i32, i32), size: (u16, u16)) -> Vec<Position> {
        let (w, h) = size;
        let (u, l) = (cmp::max(ul.0 - (w / 2) as i32, 0) as u16, cmp::max(ul.1 - (h / 2) as i32, 0) as u16);

        let w_ceil = w + map::Sector::SIZE - 1;
        let h_ceil = h + map::Sector::SIZE - 1;
        let num_sectors = w_ceil as usize * h_ceil as usize / map::Sector::SIZE as usize;

        let mut sectors = Vec::with_capacity(num_sectors);

        for x in (0..w_ceil).step_by(map::Sector::SIZE) {
            for y in (0..h_ceil).step_by(map::Sector::SIZE) {
                sectors.push(Position {
                    x: u + x,
                    y: l + y,
                    z: 7,
                });
            }
        }

        sectors
    }

    pub fn get_sector_vertices<'a, F>(&'a mut self, sector_pos: Position, mut sprite_callback: F) -> Option<&'a [V]>
        where F: FnMut((f32, f32), u32) -> V
    {
        if !self.sector_cache.contains_key(&sector_pos) {
            if let Some(s) = self.map.get(&sector_pos) {
                let mut vertices = Vec::new();
                self.render_sector(s, |(x, y), id| vertices.push(sprite_callback((x, y), id)));
                self.sector_cache.insert(sector_pos.clone(), vertices);
            } else {
                return None;
            }
        }

        Some(self.sector_cache.get_mut(&sector_pos).unwrap())
    }

    fn render_sector<F>(&self, sector: &map::Sector, mut sprite_callback: F)
        where F: FnMut((f32, f32), u32)
    {
        for (pos, tile) in sector {
            let mut elevation = 0;

            for item in tile {
                let otb_entry = &self.otb.items[item.id as usize];

                let client_id = match otb_entry.client_id {
                    Some(v) => v,
                    None => continue
                };

                let obj = &self.dat.items[(client_id - 100) as usize];

                let pattern_x = pos.x % obj.pattern_width as u16;
                let pattern_y = pos.y % obj.pattern_height as u16;

                for layer in 0..obj.layers {
                    for y in 0..obj.height {
                        for x in 0..obj.width {
                            let spr_idx = get_sprite_id(obj, layer, pattern_x, pattern_y, x, y);
                            let spr_id = obj.sprite_ids[spr_idx] as u32;

                            if spr_id == 0 {
                                continue;
                            }

                            let obj_x = pos.x as f32 - x as f32 -
                                        (obj.displacement.0 + elevation) as f32 / 32.;
                            let obj_y = pos.y as f32 - y as f32 -
                                        (obj.displacement.1 + elevation) as f32 / 32.;

                            sprite_callback((obj_x, obj_y), spr_id);
                        }
                    }
                }

                elevation += obj.elevation;
            }
        }
    }
}
