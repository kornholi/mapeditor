use std::io;
use std::iter;

use num::FromPrimitive;

use helpers::ReadExt;

use super::binaryfile;

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum AttributeKind {
	ServerId = 0x10,
	ClientId,
	Name,
	Description,
	Speed,
	Slot,
	MaxItems,
	Weight,
	Weapon,
	Ammunition,
	Armor,
	MagicLevel,
	MagicFieldType,
	Writeable,
	RotateTo,
	Decay,
	SpriteHash,
	MinimapColor,
	ATTR_07,
	ATTR_08,
	Light,

	Decay2, //deprecated
	Weapon2, //deprecated
	Ammunition2, //deprecated
	Armor2, //deprecated
	Writeable2, //deprecated
	Light2, 
	TopOrder,
	Writeable3, //deprecated

	WareId
}
}

#[derive(Debug, Default)]
pub struct Container {
	pub flags: u32,
	pub version: (u32, u32, u32),
	pub description: String,
}

impl Container {
	pub fn new(r: &mut io::Read) -> io::Result<Container> {
		let root_node = try!(binaryfile::Node::deserialize(r, false));
		let mut data = &root_node.data[..];

		let mut container = Container { ..Default::default() };

		// currently not being used
		container.flags = try!(data.read_u32());

		let attr = try!(data.read_byte());
		if attr == 1 {
			let datalen = try!(data.read_u16());

			let major_version = try!(data.read_u32());
			let minor_version = try!(data.read_u32());
			let build = try!(data.read_u32());

			container.version = (major_version, minor_version, build);
			
			let mut desc = try!(data.read_fixed_string(128));

			if let Some(end) = desc.find('\0') {
				desc.truncate(end);
			}

			container.description = desc;
		}

		let mut total = 0;
		let mut highest = 0;

		for ref item_node in root_node.children {
			let mut data = &item_node.data[..];

			let flags = try!(data.read_u32());

			let mut has_sid = false;
			let mut has_cid = false;

			while !data.is_empty() {
				use self::AttributeKind::*;

				let kind = AttributeKind::from_u8(try!(data.read_byte())).expect("unknown map node kind");
				let len = try!(data.read_u16());

				match kind {
					ServerId => {
						let server_id = try!(data.read_u16());
						has_sid = true;

						if server_id > highest {
							highest = server_id;
						}
					}

					ClientId => has_cid = true,

					_ => {}
				}


				//println!("{:?} {}", kind, len);
				data = &data[len as usize..];
			}

			if !has_cid || !has_sid {
				println!("WOW!!!! {} {}", has_sid, has_cid);
			}

			total+=1;
		}

		println!("{} max {}", total, highest);

		Ok(container)
	}
}