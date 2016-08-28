use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::hash_map::Entry;

use glium;
use glium::backend::Facade;
use glium::texture::SrgbTexture2d;

pub struct SpriteAtlas {
    pub texture: SrgbTexture2d,
    sprites: HashMap<u32, [f32; 2]>,
    loading_buffer: Vec<u8>,
}

static EMPTY_SPRITE: &'static [u8] = &[0; 32 * 32 * 4];

impl SpriteAtlas {
    pub fn new<F: Facade>(display: &F) -> SpriteAtlas {
        let texture = SrgbTexture2d::empty(display, 2048, 2048).expect("texture creation failed");

        SpriteAtlas {
            texture: texture,
            sprites: HashMap::new(),
            loading_buffer: vec![0; 32 * 32 * 4],
        }
    }

    pub fn get_or_load<F>(&mut self, id: u32, mut loader: F) -> [f32; 2]
        where F: FnMut(&mut [u8], usize)
    {
        assert!(self.sprites.len() < (2048 * 2048) / (34 * 34));
        let end_idx = self.sprites.len() + 1;

        match self.sprites.entry(id) {
            Entry::Occupied(tex) => tex.get().clone(),
            Entry::Vacant(tex) => {
                let width_in_sprites = 2048 / 34;
                let (l, b) = (end_idx % width_in_sprites, end_idx / width_in_sprites);

                self.loading_buffer[..].copy_from_slice(EMPTY_SPRITE);

                loader(&mut self.loading_buffer, 32 * 4);

                let sprite = glium::texture::RawImage2d {
                    data: Cow::Borrowed(&self.loading_buffer),
                    width: 32,
                    height: 32,
                    format: glium::texture::ClientFormat::U8U8U8U8,
                };

                self.texture.write(glium::Rect {
                                       left: l as u32 * 34,
                                       bottom: b as u32 * 34,
                                       width: 32,
                                       height: 32,
                                   },
                                   sprite);

                tex.insert([l as f32 * 34. / 2048., b as f32 * 34. / 2048.]).clone()
            }
        }
    }
}
