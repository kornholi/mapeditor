use std::io;
use glium;

use helpers::ReadExt;

pub struct SpriteContainer<R> {
    pub f: R,
    pub signature: u32,
    pub num_sprites: u32,
    pub offsets: Vec<u32>,
}

pub type SpriteImage<'a> = glium::texture::RawImage2d<'a, u8>;

// TODO:
//  support for u16 num sprites
//  autodetect u16/u32 based on signature

impl<R> SpriteContainer<R>
    where R: io::Read + io::Seek
{
    pub fn new(mut r: R) -> io::Result<SpriteContainer<R>> {
        let signature = try!(r.read_u32());
        let num_sprites = try!(r.read_u32());

        let mut offsets = Vec::with_capacity(num_sprites as usize);

        for _ in 0..num_sprites {
            offsets.push(try!(r.read_u32()));
        }

        Ok(SpriteContainer {
            f: r,
            signature: signature,
            num_sprites: num_sprites,
            offsets: offsets,
        })
    }

    pub fn get_sprite(&mut self,
                      idx: u32,
                      output: &mut [u8],
                      output_stride: usize)
                      -> io::Result<()> {
        let offset = self.offsets[idx as usize - 1];
        try!(self.f.seek(io::SeekFrom::Start(offset as u64)));

        // RGB color key (typically magenta)
        try!(self.f.read_byte());
        try!(self.f.read_byte());
        try!(self.f.read_byte());

        let mut size = try!(self.f.read_u16());
        let mut p = 0;

        while size > 0 {
            let transparent_pixels = try!(self.f.read_u16());
            let pixels = try!(self.f.read_u16());

            p += 4 * transparent_pixels as usize;

            for _ in 0..pixels {
                try!(self.f.read(&mut output[p..p + 3]));

                // Set alpha channel
                output[p + 3] = 255;
                p += 4;
            }

            size -= 2 + 2 + pixels * 3;
        }

        Ok(())
    }
}
