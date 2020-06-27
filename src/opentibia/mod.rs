use crate::helpers::ReadExt;
use std::fmt;
use std::io;

pub mod binaryfile;
pub mod itemtypes;
pub mod map;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub z: u8,
}

impl Position {
    pub fn deserialize<R>(mut r: R) -> io::Result<Position>
    where
        R: io::Read,
    {
        Ok(Position {
            x: r.read_u16()?,
            y: r.read_u16()?,
            z: r.read_byte()?,
        })
    }
}

impl fmt::Display for Position {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.write_fmt(format_args!("{},{},{}", self.x, self.y, self.z))
    }
}
