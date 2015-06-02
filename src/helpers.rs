use std::io;
use byteorder::{ReadBytesExt, LittleEndian, Result};

pub trait ReadExt: io::Read {
    fn read_byte(&mut self) -> Result<u8> {
        ReadBytesExt::read_u8(self)
    }

    fn read_u16(&mut self) -> Result<u16> {
        ReadBytesExt::read_u16::<LittleEndian>(self)
    }

    fn read_i16(&mut self) -> Result<i16> {
        ReadBytesExt::read_i16::<LittleEndian>(self)
    }

    fn read_u32(&mut self) -> Result<u32> {
        ReadBytesExt::read_u32::<LittleEndian>(self)
    }

    fn read_i32(&mut self) -> Result<i32> {
        ReadBytesExt::read_i32::<LittleEndian>(self)
    }

    fn read_f32(&mut self) -> Result<f32> {
        ReadBytesExt::read_f32::<LittleEndian>(self)
    }
}

impl<R: io::Read + ?Sized> ReadExt for R {}
