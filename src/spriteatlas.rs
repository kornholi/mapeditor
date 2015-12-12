use std::collections::HashMap;

use glium;
use image;

use glium::backend::Facade;
use glium::texture::Texture2d;

pub struct SpriteAtlas {
    pub texture: Texture2d,

    sprites: HashMap<u32, [f32; 2]>
}

impl SpriteAtlas {
    pub fn new<F: Facade>(display: &F) -> SpriteAtlas {
        let texture = Texture2d::empty(display, 2048, 2048).expect("texture creation failed");

        SpriteAtlas {
            texture: texture,
            sprites: HashMap::new()
        }
    }

    pub fn get(&self, id: u32) -> [f32; 2]
    {
        match self.sprites.get(&id) {
            Some(pos) => *pos,
            None => [0., 0.]
        }
    }

    pub fn add(&mut self, id: u32, sprite: image::ImageBuffer<image::Rgba<u8>, Vec<u8>>) -> [f32; 2]
    {
        assert!(self.sprites.len() < (2048*2048)/(32*32));

        let data = glium::texture::RawImage2d::from_raw_rgba_reversed(sprite.into_raw(), (32, 32));

        let end_idx = self.sprites.len() + 1;
        
        let (l, b) = (end_idx % 64, end_idx / 64);
        let pos = [l as f32 * 32. / 2048., b as f32 * 32. / 2048.];

        self.texture.write(glium::Rect { left: l as u32 * 32, bottom: b as u32 * 32, width: 32, height: 32}, data);
        self.sprites.insert(id, pos);

        pos
    }
}
