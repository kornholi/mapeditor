use std::{io, fs};

use datcontainer;
use datcontainer::DatContainer;

use spritecontainer::SpriteContainer;
use opentibia;
use opentibia::itemtypes;

use super::map;
use super::spriteatlas::SpriteAtlas;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

pub struct Renderer {
    pub spr: SpriteContainer,
    pub spr_data: io::BufReader<fs::File>,
    pub dat: DatContainer,
    pub otb: itemtypes::Container,

    pub atlas: SpriteAtlas,
    pub map: map::Map,
    
    pub vertices: Vec<Vertex>,
}

fn get_sprite_id(obj: &datcontainer::Thing, layer: u8, pattern_x: u16, pattern_y: u16, x: u8, y: u8) -> usize {
    let animation_time = 0;

    ((((((animation_time % 4095) * obj.pattern_height as u16
        + pattern_y % obj.pattern_height as u16) * obj.pattern_width as u16
        + pattern_x % obj.pattern_width as u16) * obj.layers as u16
        + layer as u16) * obj.height as u16
        + y as u16) * obj.width as u16
        + x as u16) as usize % obj.sprite_ids.len()
}

impl Renderer {
    pub fn render(&mut self) -> &[Vertex] {
        self.vertices.clear();

        self.render_sector(opentibia::Position { x: 95, y: 117, z: 7 });
        self.render_sector(opentibia::Position { x: 95+32, y: 117, z: 7 });
        self.render_sector(opentibia::Position { x: 95, y: 117+32, z: 7 });
        self.render_sector(opentibia::Position { x: 95+32, y: 117+32, z: 7 });

        &self.vertices[..]
    }

    fn render_sector(&mut self, pos: opentibia::Position) {
        let sec = self.map.get(pos).unwrap();

        let mut pos = 0u16;

        // must iterate with y
        for ref tile in sec.tiles.iter() {
            let tile_x = sec.origin.x + pos / 32;
            let tile_y = sec.origin.y + pos % 32;

            let mut elevation = 0;

            for ref item in tile.iter() {
                let otb_entry = &self.otb.items[item.id as usize];

                if let Some(client_id) = otb_entry.client_id {
                    let obj = &self.dat.items[(client_id - 100) as usize];
                    //println!("dat: {:?}", obj);

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
                                        let sprite = self.spr.get_sprite(&mut self.spr_data, spr_id).unwrap();
                                        tex_pos = self.atlas.add(spr_id, sprite);
                                    }

                                    let obj_x = tile_x as f32 - x as f32 - (obj.displacement.0 + elevation) as f32 / 32.;
                                    let obj_y = tile_y as f32 - y as f32 - (obj.displacement.1 + elevation) as f32 / 32.;

                                    self.vertices.push(Vertex { position: [obj_x, obj_y, 7.], color: [1.0, 1.0, 1.0, 1.0], tex_coord: tex_pos });
                                }
                            }
                        }
                    }

                    elevation += obj.elevation;
                }
            }

            pos += 1;
        }
    }
}
