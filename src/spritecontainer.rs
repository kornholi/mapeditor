use std::io;
use crate::helpers::ReadExt;

pub struct SpriteContainer<R> {
    pub f: R,
    pub signature: u32,
    pub num_sprites: u32,
    pub offsets: Vec<u32>,
}

// TODO:
//  Support for u16 num_sprites
//  Autodetect u16/u32 based on file signature

impl<R> SpriteContainer<R>
    where R: io::Read + io::Seek
{
    pub fn new(mut r: R) -> io::Result<SpriteContainer<R>> {
        let signature = r.read_u32()?;
        let num_sprites = r.read_u32()?;

        let mut offsets = Vec::with_capacity(num_sprites as usize);

        for _ in 0..num_sprites {
            offsets.push(r.read_u32()?);
        }

        Ok(SpriteContainer {
            f: r,
            signature,
            num_sprites,
            offsets,
        })
    }

    pub fn get_sprite(&mut self,
                      idx: u32,
                      output: &mut [u8],
                      output_stride: usize)
                      -> io::Result<()> {
        let offset = self.offsets[idx as usize - 1];
        self.f.seek(io::SeekFrom::Start(offset as u64))?;

        // RGB color key (typically magenta)
        self.f.read_byte()?;
        self.f.read_byte()?;
        self.f.read_byte()?;

        let mut size = self.f.read_u16()?;
        let (mut p, mut i) = (0, 0);

        let bytes_to_next_row = output_stride - 32 * 4;

        while size > 0 {
            let transparent_pixels = self.f.read_u16()? as usize;
            let pixels = self.f.read_u16()?;

            let rows_skipped = (i + transparent_pixels) / 32 - i / 32;

            i += transparent_pixels;
            p += transparent_pixels * 4 + bytes_to_next_row * rows_skipped;

            for _ in 0..pixels {
                self.f.read_exact(&mut output[p..p + 3])?;

                // Set alpha channel
                output[p + 3] = 255;
                p += 4;
                i += 1;

                if i % 32 == 0 {
                    p += bytes_to_next_row;
                }
            }

            size -= 2 + 2 + pixels * 3;
        }

        Ok(())
    }
}
