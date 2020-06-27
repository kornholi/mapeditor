use std::borrow::Cow;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use glium::backend::Facade;
use glium::texture::SrgbTexture2d;

pub struct SpriteAtlas {
    pub texture: SrgbTexture2d,
    sprites: HashMap<u32, [f32; 2]>,
    loading_buffer: Vec<u8>,
}

static EMPTY_SPRITE: &[u8] = &[0; 34 * 34 * 4];

#[inline(always)]
fn copy_pixel(buf: &mut [u8], tx: usize, ty: usize, fx: usize, fy: usize) {
    for i in 0..4 {
        buf[ty * 34 * 4 + tx * 4 + i] = buf[fy * 34 * 4 + fx * 4 + i];
    }
}

fn copy_borders(buf: &mut [u8]) {
    // Copy top and bottom borders
    {
        let (top, rest) = buf.split_at_mut(34 * 4);
        let (body, bottom) = rest.split_at_mut(32 * 34 * 4);

        top.copy_from_slice(&body[..34 * 4]);
        bottom.copy_from_slice(&body[31 * 34 * 4..]);
    }

    // Copy left and right borders
    for i in 0..34 {
        copy_pixel(buf, 0, i, 1, i);
        copy_pixel(buf, 33, i, 32, i);
    }
}

impl SpriteAtlas {
    pub fn new<F: Facade>(display: &F) -> SpriteAtlas {
        let texture = SrgbTexture2d::empty(display, 2048, 2048).expect("texture creation failed");

        SpriteAtlas {
            texture,
            sprites: HashMap::new(),
            loading_buffer: vec![0; 34 * 34 * 4],
        }
    }

    pub fn get_or_load<F>(&mut self, id: u32, mut loader: F) -> [f32; 2]
    where
        F: FnMut(&mut [u8], usize),
    {
        assert!(self.sprites.len() < (2048 * 2048) / (34 * 34));
        let end_idx = self.sprites.len() + 1;

        match self.sprites.entry(id) {
            Entry::Occupied(tex) => *tex.get(),
            Entry::Vacant(tex) => {
                let width_in_sprites = 2048 / 34;
                let (l, b) = (end_idx % width_in_sprites, end_idx / width_in_sprites);

                self.loading_buffer[..].copy_from_slice(EMPTY_SPRITE);

                // Load sprite at (1,1)
                loader(&mut self.loading_buffer[35 * 4..], 34 * 4);

                // Store 1px border around the sprite to eliminate bilinear
                // resampling errors
                copy_borders(&mut self.loading_buffer);

                let sprite = glium::texture::RawImage2d {
                    data: Cow::Borrowed(&self.loading_buffer),
                    width: 34,
                    height: 34,
                    format: glium::texture::ClientFormat::U8U8U8U8,
                };

                self.texture.write(
                    glium::Rect {
                        left: l as u32 * 34,
                        bottom: b as u32 * 34,
                        width: 34,
                        height: 34,
                    },
                    sprite,
                );

                *tex.insert([(l as f32 * 34. + 1.) / 2048., (b as f32 * 34. + 1.) / 2048.])
            }
        }
    }
}
