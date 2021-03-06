use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Result};

use encoding::all::WINDOWS_1252;
use encoding::{DecoderTrap, Encoding};

use crate::opentibia::Position;

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

    fn read_string(&mut self) -> Result<String> {
        let length = self.read_u16()? as usize;
        self.read_fixed_string(length)
    }

    fn read_fixed_string(&mut self, length: usize) -> Result<String> {
        let mut data = vec![0; length];
        self.read_exact(&mut data)?;

        // FIXME: error handling
        Ok(WINDOWS_1252.decode(&data, DecoderTrap::Strict).unwrap())
    }

    fn read_position(&mut self) -> Result<Position> {
        Position::deserialize(self)
    }
}

impl<R: io::Read + ?Sized> ReadExt for R {}
