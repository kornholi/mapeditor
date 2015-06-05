use std::io;
use std::iter;

use num::FromPrimitive;

use helpers::ReadExt;

use super::binaryfile;
use super::Position;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum NodeKind {
	Root = 0,
    MapData = 2,
    ItemDef = 3,
    TileArea = 4,
    Tile = 5,
    Item = 6,
    TileSquare = 7,
    TileRef = 8,
    Spawns = 9,
    SpawnArea = 10,
    Monster = 11,
    Towns = 12,
    Town = 13,
    HouseTile = 14,
    WayPoints = 15,
    WayPoint = 16
}
}

#[derive(Debug)]
pub struct Container {
	pub version: u32,
	pub width: u16,
	pub height: u16,
	pub items_version: (u32, u32)
}

impl Container {
	pub fn load(root_node: &binaryfile::Node) -> io::Result<Container> {
		let kind = NodeKind::from_u8(root_node.kind).expect("unknown map node");
		if kind != NodeKind::Root { 
			return Err(io::Error::new(io::ErrorKind::Other, "invalid map node"))
		}

		let mut r = &root_node.data[..];

		Ok(Container {
			version: try!(r.read_u32()),
			width: try!(r.read_u16()),
			height: try!(r.read_u16()),
			items_version: (try!(r.read_u32()), try!(r.read_u32()))
		})
	}
}

pub fn streaming_debug(kind: u8, mut data: &[u8]) -> io::Result<bool> {
	let nn = NodeKind::from_u8(kind).expect("unknown map node");

	match nn {
		NodeKind::Root => {
			//let root = try!(Node::deserialize_root(&mut data));
			//println!("root {:?}", root);
			//let description = 
		}

		NodeKind::MapData => {
			while !data.is_empty() {
				match try!(data.read_byte()) {
					1 => {
						println!("desc {}", try!(data.read_string()));
					}

					11 => {
						println!("house {}", try!(data.read_string()));
					}

					13 => {
						println!("spawn {}", try!(data.read_string()));
					}

					_ => {}
				}
			}

			return Ok(false)
		}

		NodeKind::TileArea => {
			let origin = try!(Position::deserialize(&mut data));
		}

		NodeKind::Town => {
			let town = try!(Town::deserialize(&mut data));
			println!("town: {:?}", town);
		}

		NodeKind::WayPoint => {
			let wp = try!(Waypoint::deserialize(&mut data));
			println!("waypoint {:?}", wp)
		}

		_ => {}
	}

	Ok(true)
}

pub fn debug(node: &binaryfile::Node, level: usize) -> io::Result<()> {
	let nn = NodeKind::from_u8(node.kind).expect("unknown map node");

	if nn != NodeKind::TileArea && nn != NodeKind::Tile && nn != NodeKind::Item && nn != NodeKind::HouseTile {
		let lvl = String::from_utf8(iter::repeat('-' as u8).take(level).collect()).unwrap();
		println!("{} {:?} {}b of data {} children", lvl, nn, node.data.len(), node.children.len());
	}

	let mut data = &node.data[..];

	match nn {
		NodeKind::Root => {
			//let root = try!(Node::deserialize_root(&mut data));
			//println!("root {:?}", root);
			//let description = 
		}

		NodeKind::MapData => {
			while !data.is_empty() {
				match try!(data.read_byte()) {
					1 => {
						println!("desc {}", try!(data.read_string()));
					}

					11 => {
						println!("house {}", try!(data.read_string()));
					}

					13 => {
						println!("spawn {}", try!(data.read_string()));
					}

					_ => {}
				}
			}
		}

		NodeKind::TileArea => {
			let origin = try!(Position::deserialize(&mut data));

			for ref c in &node.children[..] {
				let c_kind = NodeKind::from_u8(c.kind).expect("unknown map node");
				if c_kind != NodeKind::Tile && c_kind != NodeKind::HouseTile {
					continue
				}

				let mut data = &c.data[..];

				let x_offset = try!(data.read_byte());
				let y_offset = try!(data.read_byte());

				if c_kind == NodeKind::HouseTile {
					let house_id = try!(data.read_u32());
				}

				while !data.is_empty() {
					match try!(data.read_byte()) {
						3 => {
							let flags = try!(data.read_u32());
						}

						9 => {
							let item_id = try!(data.read_u16());
						}

						_ => unreachable!()
					}
				}
			}
		}

		NodeKind::Town => {
			let town = try!(Town::deserialize(&mut data));
			println!("town: {:?}", town);
		}

		NodeKind::WayPoint => {
			let wp = try!(Waypoint::deserialize(&mut data));
			println!("waypoint {:?}", wp)
		}

		_ => {}
	}

	for ref c in &node.children[..] {
		debug(c, level + 1);
	}

	Ok(())
}

#[derive(Debug)]
pub struct Town {
	pub id: u32,
	pub name: String,
	pub temple_position: Position
}

impl Town {
	pub fn deserialize(r: &mut io::Read) -> io::Result<Town> {
		let id = try!(r.read_u32());
		let name = try!(r.read_string());
		let pos = try!(Position::deserialize(r));

	 	Ok(Town { id: id, name: name, temple_position: pos })
	}
}

#[derive(Debug)]
pub struct Waypoint {
	pub name: String,
	pub position: Position
}

impl Waypoint {
	pub fn deserialize(r: &mut io::Read) -> io::Result<Waypoint> {
		let name = try!(r.read_string());
		let pos = try!(Position::deserialize(r));
		Ok(Waypoint { name: name, position: pos })
	}
}