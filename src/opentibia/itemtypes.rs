use num::FromPrimitive;
use num_derive::FromPrimitive;
use std::io;
use vec_map::VecMap;

use crate::helpers::ReadExt;

use super::binaryfile;

#[derive(Debug, FromPrimitive, PartialEq)]
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

    Decay2,      //deprecated
    Weapon2,     //deprecated
    Ammunition2, //deprecated
    Armor2,      //deprecated
    Writeable2,  //deprecated
    Light2,
    TopOrder,
    Writeable3, //deprecated

    WareId,
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
    where
        R: io::Read,
    {
        let root_node = binaryfile::Node::deserialize(&mut r, false)?;
        let mut data = &root_node.data[..];

        let mut container = Container {
            ..Default::default()
        };

        // currently not being used
        container.flags = data.read_u32()?;

        let attr = data.read_byte()?;
        if attr == 1 {
            let data_len = data.read_u16()?;
            assert_eq!(data_len, 140);

            let major_version = data.read_u32()?;
            let minor_version = data.read_u32()?;
            let build = data.read_u32()?;

            container.version = (major_version, minor_version, build);

            let mut desc = data.read_fixed_string(128)?;

            if let Some(end) = desc.find('\0') {
                desc.truncate(end);
            }

            container.description = desc;
        }

        for item_node in &root_node.children {
            let mut item = Item {
                ..Default::default()
            };

            let mut data = &item_node.data[..];
            let _flags = data.read_u32()?;

            while !data.is_empty() {
                use self::AttributeKind::*;

                let kind = AttributeKind::from_u8(data.read_byte()?).expect("unknown map node");
                let len = data.read_u16()?;

                match kind {
                    ServerId => item.server_id = data.read_u16()?,
                    ClientId => item.client_id = Some(data.read_u16()?),

                    _ => data = &data[len as usize..],
                }
            }

            container.items.insert(item.server_id as usize, item);
        }

        Ok(container)
    }
}
