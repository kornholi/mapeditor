use std::io;
use helpers::ReadExt;

const NODE_ESCAPE: u8 = 0xFD;
const NODE_START: u8 = 0xFE;
const NODE_END: u8 = 0xFF;

#[derive(Debug)]
pub struct Node {
    pub kind: u8,
    pub data: Vec<u8>,
    pub children: Vec<Node>
}

impl Node {
	pub fn deserialize(r: &mut io::Read, skip_start: bool) -> io::Result<Node> {
		if !skip_start {
			let data = try!(r.read_byte());

			if data != NODE_START {
				let invalid_data_error: io::Error = io::Error::new(io::ErrorKind::Other, "unexpected data");
				return Err(invalid_data_error)
			}
		}

		let kind = try!(r.read_byte());

		let mut data = Vec::new();
		let mut children = Vec::new();

		loop {
			let b = try!(r.read_byte());

			match b {
				NODE_START => children.push(try!(Node::deserialize(r, true))),
				NODE_END => break,
				NODE_ESCAPE	=> data.push(try!(r.read_byte())),
				_ => data.push(b)
			}
		}

		data.shrink_to_fit();
		children.shrink_to_fit();

		Ok(Node {
			kind: kind,
			data: data,
			children: children
		})
	}
}