use std::{io, fs};

use clock_ticks;

use datcontainer;
use datcontainer::DatContainer;

use spritecontainer::SpriteContainer;

use opentibia;
use opentibia::{itemtypes, Position};

use super::map;
use super::spriteatlas::SpriteAtlas;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, color, tex_coord);

pub struct Renderer {
    pub spr: SpriteContainer<io::BufReader<fs::File>>,
    pub dat: DatContainer,
    pub otb: itemtypes::Container,

    pub map: map::Map,

    pub atlas: SpriteAtlas,
    pub bounds: (u16, u16, u16, u16),
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

impl Renderer {
    pub fn resize(&mut self, ul: (i32, i32), size: (u16, u16), output: &mut Vec<Vertex>) {
        let (u, l) = ul;
        let (u, l) = (u as u16, l as u16);
        let (w, h) = size;

        let br = ((u + w), (l + h));

        let bnd = self.bounds;

        if u < bnd.0 || l < bnd.1 || br.0 > bnd.2 || br.1 > bnd.3 {
            println!("resize {:?} {:?} bnd {:?}", ul, size, bnd);
        } else {
            return;
        }

        // FIXME FIXME FIXME FIXME
        self.bounds = (u & !31,
                       l & !31,
                       ((u + 31) & !31) + (w + 31) & !31,
                       ((l + 31) & !31) + (h + 31) & !31);

        let start = clock_ticks::precise_time_ms();
        let mut sector_count = 0;

        for x in (0..w + 31).step_by(map::Sector::SIZE) {
            for y in (0..h + 31).step_by(map::Sector::SIZE) {
                let pos = Position {
                    x: u + x,
                    y: l + y,
                    z: 7,
                };

                if self.try_render_sector(&pos, output) {
                    sector_count += 1;
                }
            }
        }

        let end = clock_ticks::precise_time_ms();
        println!("Rendering {} sectors took {}ms", sector_count, end - start);
    }

    fn try_render_sector(&mut self, pos: &opentibia::Position, output: &mut Vec<Vertex>) -> bool {
        let sec = match self.map.get(pos) {
            Some(sec) => sec,
            None => return false,
        };

        let mut pos = 0u16;

        // Must iterate with y
        for tile in &sec.tiles {
            let tile_x = sec.origin.x + pos / 32;
            let tile_y = sec.origin.y + pos % 32;

            let mut elevation = 0;

            for item in tile {
                let otb_entry = &self.otb.items[item.id as usize];

                if let Some(client_id) = otb_entry.client_id {
                    let obj = &self.dat.items[(client_id - 100) as usize];

                    let pattern_x = tile_x as u16 % obj.pattern_width as u16;
                    let pattern_y = tile_y as u16 % obj.pattern_height as u16;

                    for layer in 0..obj.layers {
                        for y in 0..obj.height {
                            for x in 0..obj.width {
                                let spr_idx = get_sprite_id(obj, layer, pattern_x, pattern_y, x, y);
                                let spr_id = obj.sprite_ids[spr_idx] as u32;

                                if spr_id != 0 {
                                    let mut tex_pos = self.atlas.get(spr_id);

                                    if tex_pos == [0., 0.] {
                                        use glium;
                                        let mut sprite_data = vec![0; 32 * 32 * 4];

                                        self.spr
                                            .get_sprite(spr_id, &mut sprite_data, 32 * 4)
                                            .unwrap();

                                        let sprite = glium::texture::RawImage2d::from_raw_rgba_reversed(sprite_data, (32, 32));
                                        tex_pos = self.atlas.add(spr_id, sprite);
                                    }

                                    let obj_x = tile_x as f32 - x as f32 -
                                                (obj.displacement.0 + elevation) as f32 / 32.;
                                    let obj_y = tile_y as f32 - y as f32 -
                                                (obj.displacement.1 + elevation) as f32 / 32.;

                                    output.push(Vertex {
                                        position: [obj_x, obj_y, 7.],
                                        color: [1.0, 1.0, 1.0, 1.0],
                                        tex_coord: tex_pos,
                                    });
                                }
                            }
                        }
                    }

                    elevation += obj.elevation;
                }
            }

            pos += 1;
        }

        return true;
    }
}
