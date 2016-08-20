use std::{io, ptr};
use glium;

use helpers::ReadExt;

pub struct SpriteContainer {
    pub signature: u32,
    pub num_sprites: u32,
    pub offsets: Vec<u32>,
}

pub type SpriteImage<'a> = glium::texture::RawImage2d<'a, u8>;

// TODO:
//  support for u16 num sprites
//  autodetect u16/u32 based on signature

impl SpriteContainer {
    pub fn new(r: &mut io::Read) -> io::Result<SpriteContainer> {
        let signature = try!(r.read_u32());
        let num_sprites = try!(r.read_u32());

        let mut offsets = Vec::with_capacity(num_sprites as usize);

        for _ in 0..num_sprites {
            offsets.push(try!(r.read_u32()));
        }

        Ok(SpriteContainer {
            signature: signature,
            num_sprites: num_sprites,
            offsets: offsets,
        })
    }

    pub fn get_sprite<T: io::Read + io::Seek>(&self,
                                              f: &mut T,
                                              idx: u32)
                                              -> io::Result<SpriteImage> {
        try!(f.seek(io::SeekFrom::Start(self.offsets[idx as usize - 1] as u64)));

        let mut raw_data = Vec::with_capacity(32 * 32 * 4);

        unsafe {
            ptr::write_bytes(raw_data.as_mut_ptr(), 0, 32 * 32 * 4);
            raw_data.set_len(32 * 32 * 4);
        }

        // RGB color key (typically magenta)
        try!(f.read_byte());
        try!(f.read_byte());
        try!(f.read_byte());

        let mut size = try!(f.read_u16());
        let mut p = 0;

        while size > 0 {
            let transparent_pixels = try!(f.read_u16());
            let pixels = try!(f.read_u16());

            p += 4 * transparent_pixels as usize;

            for _ in 0..pixels {
                try!(f.read(&mut raw_data[p..p + 3]));

                // Set alpha channel
                raw_data[p + 3] = 255; 
                p += 4;
            }

            size -= 2 + 2 + pixels * 3;
        }

        Ok(glium::texture::RawImage2d::from_raw_rgba_reversed(raw_data, (32, 32)))
    }
}
