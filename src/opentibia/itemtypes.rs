use vec_map::VecMap;
use std::io;
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
    Attr07,
    Attr08,
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
    pub items: VecMap<Item>,
}

#[derive(Debug, Default)]
pub struct Item {
    pub server_id: u16,
    pub client_id: Option<u16>,
}

impl Container {
    pub fn new<R>(mut r: R) -> io::Result<Container>
        where R: io::Read
    {
        let root_node = try!(binaryfile::Node::deserialize(&mut r, false));
        let mut data = &root_node.data[..];

        let mut container = Container { ..Default::default() };

        // currently not being used
        container.flags = try!(data.read_u32());

        let attr = try!(data.read_byte());
        if attr == 1 {
            let datalen = try!(data.read_u16());
            assert!(datalen == 140);

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

        for item_node in &root_node.children {
            let mut item = Item { ..Default::default() };

            let mut data = &item_node.data[..];
            let _flags = try!(data.read_u32());

            while !data.is_empty() {
                use self::AttributeKind::*;

                let kind = AttributeKind::from_u8(try!(data.read_byte()))
                    .expect("unknown map node");
                let len = try!(data.read_u16());

                match kind {
                    ServerId => item.server_id = try!(data.read_u16()),
                    ClientId => item.client_id = Some(try!(data.read_u16())),

                    _ => data = &data[len as usize..],
                }
            }

            container.items.insert(item.server_id as usize, item);
        }

        Ok(container)
    }
}
