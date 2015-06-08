use std::io;
use std::fmt;
use helpers::ReadExt;

pub mod binaryfile;
pub mod map;
pub mod itemtypes;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: u16,
    pub y: u16,
    pub z: u8
}

impl Position {
	pub fn deserialize(r: &mut io::Read) -> io::Result<Position> {
		Ok(Position {
			x: try!(r.read_u16()),
			y: try!(r.read_u16()),
			z: try!(r.read_byte())
		})
	}
}

impl fmt::Display for Position	{
	fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		fmt.write_fmt(format_args!("{},{},{}", self.x, self.y, self.z))
	}
}
