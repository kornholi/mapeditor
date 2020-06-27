use num::FromPrimitive;
use num_derive::FromPrimitive;
use std::io;

use crate::helpers::ReadExt;

use super::binaryfile;
use super::Position;

#[derive(Debug, FromPrimitive, PartialEq)]
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
    WayPoint = 16,
}

#[derive(Debug, FromPrimitive, PartialEq)]
enum NodeAttributeKind {
    MapDescription = 1,
    TileFlags = 3,
    ItemActionId = 4,
    ItemUniqueId = 5,
    ItemText = 6,
    ItemDescription = 7,
    Teleport = 8,
    Item = 9,
    DepotId = 10,
    HouseFile = 11,
    RuneCharges = 12,
    SpawnFile = 13,
    HouseDoorId = 14,
    ItemCount = 15,
    ItemDuration = 16,
    ItemDecayingState = 17,
    ItemWrittenDate = 18,
    ItemWrittenBy = 19,
    SleeperGuid = 20,
    SleepStart = 21,
    ItemCharges = 22,
    ContainerItems = 23,
    ItemName = 24,
    ItemArticle = 25,
    ItemPluralName = 26,
    ItemWeight = 27,
    ItemAttack = 28,
    ItemDefense = 29,
    ItemExtraDefense = 30,
    ItemArmor = 31,
    ItemHitChance = 32,
    ItemShootRange = 33,

    AttributeMap = 128,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: u16,
    pub attributes: Vec<ItemAttribute>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ItemAttribute {
    Count(u8),
    ActionId(u16),
    UniqueId(u16),
    Text(String),
    Description(String),
    Teleport(Position),
    DepotId(u16),
    HouseDoorId(u8),
    Duration(i32),
    DecayingState(u8),
    WrittenDate(u32),
    WrittenBy(String),
    SleeperGuid(u32),
    SleepStart(u32),
    Charges(u16),
    Name(String),
    Article(String),
    PluralName(String),
    Weight(u32),
    Attack(i32),
    Defense(i32),
    ExtraDefense(i32),
    Armor(i32),
    HitChance(u8),
    ShootRange(u8),
}

#[derive(Debug, Default)]
pub struct Loader {
    pub version: u32,
    pub width: u16,
    pub height: u16,
    pub items_version: (u32, u32),

    pub description: Vec<String>,
    pub house_file: Vec<String>,
    pub spawn_file: Vec<String>,

    pub towns: Vec<Town>,
    pub waypoints: Vec<Waypoint>,

    current_tile_origin: Option<Position>,
    current_tile: Option<Position>,
    current_tile_items: Vec<Item>,
}

impl Loader {
    pub fn open<R>(r: R) -> io::Result<Loader>
    where
        R: io::Read,
    {
        let mut loader = Loader {
            ..Default::default()
        };

        binaryfile::streaming_parser(r, false, |kind, data| {
            loader.load_headers_callback(kind, data)
        })?;

        Ok(loader)
    }

    pub fn load<F>(&mut self, r: &mut dyn io::Read, mut tile_callback: F) -> io::Result<()>
    where
        F: FnMut(Position, &[Item]),
    {
        binaryfile::streaming_parser(r, true, |kind, data| {
            self.load_callback(kind, data, &mut tile_callback)
        })
    }

    fn load_headers_callback(&mut self, kind: u8, mut data: &[u8]) -> io::Result<bool> {
        let kind = NodeKind::from_u8(kind).expect("unknown map node kind");

        match kind {
            NodeKind::Root => {
                self.version = data.read_u32()?;
                self.width = data.read_u16()?;
                self.height = data.read_u16()?;
                self.items_version = (data.read_u32()?, data.read_u32()?);

                Ok(true)
            }

            NodeKind::MapData => {
                while !data.is_empty() {
                    use self::NodeAttributeKind::*;
                    let raw_attr = data.read_byte()?;
                    let attribute =
                        NodeAttributeKind::from_u8(raw_attr).expect("unknown attribute");

                    match attribute {
                        MapDescription => self.description.push(data.read_string()?),
                        HouseFile => self.house_file.push(data.read_string()?),
                        SpawnFile => self.spawn_file.push(data.read_string()?),
                        _ => panic!("Unknown map attribute"),
                    }
                }

                Ok(false)
            }

            _ => unreachable!(),
        }
    }

    fn load_callback<F>(
        &mut self,
        kind: u8,
        mut data: &[u8],
        mut tile_callback: F,
    ) -> io::Result<bool>
    where
        F: FnMut(Position, &[Item]),
    {
        let kind = NodeKind::from_u8(kind).expect("unknown map node kind");

        match kind {
            NodeKind::TileArea => {
                let origin = data.read_position()?;
                self.current_tile_origin = Some(origin);
            }

            NodeKind::Tile | NodeKind::HouseTile => {
                let x_offset = data.read_byte()? as u16;
                let y_offset = data.read_byte()? as u16;

                if let Some(origin) = self.current_tile_origin {
                    if let Some(old_pos) = self.current_tile {
                        tile_callback(old_pos, &self.current_tile_items);
                        self.current_tile_items.clear();
                    }

                    self.current_tile = Some(Position {
                        x: origin.x + x_offset,
                        y: origin.y + y_offset,
                        z: origin.z,
                    });
                } else {
                    panic!("Encountered Tile outside of a TileArea");
                }

                if kind == NodeKind::HouseTile {
                    let _house_id = data.read_u32()?;
                }

                while !data.is_empty() {
                    use self::NodeAttributeKind::*;
                    let raw_attr = data.read_byte()?;
                    let attr = NodeAttributeKind::from_u8(raw_attr).expect("unknown attribute");

                    match attr {
                        TileFlags => {
                            let _flags = data.read_u32()?;
                        }

                        Item => {
                            let item_id = data.read_u16()?;
                            self.current_tile_items.push(self::Item {
                                id: item_id,
                                attributes: Vec::new(),
                            });
                        }

                        _ => panic!("unexpected tile attribute {:?}", attr),
                    }
                }
            }

            NodeKind::Item => {
                if self.current_tile.is_none() {
                    panic!("Encountered Item outside of a Tile");
                }

                let item_id = data.read_u16()?;

                let mut item = Item {
                    id: item_id,
                    attributes: Vec::new(),
                };

                while !data.is_empty() {
                    use self::NodeAttributeKind::*;

                    let raw_attr = data.read_byte()?;
                    let attribute_kind =
                        NodeAttributeKind::from_u8(raw_attr).expect("unknown attribute");

                    let attribute = match attribute_kind {
                        ItemCount | RuneCharges => ItemAttribute::Count(data.read_byte()?),
                        ItemCharges => ItemAttribute::Charges(data.read_u16()?),
                        ItemText => ItemAttribute::Text(data.read_string()?),
                        ItemActionId => ItemAttribute::ActionId(data.read_u16()?),
                        ItemUniqueId => ItemAttribute::UniqueId(data.read_u16()?),
                        ItemWrittenDate => ItemAttribute::WrittenDate(data.read_u32()?),
                        ItemWrittenBy => ItemAttribute::WrittenBy(data.read_string()?),
                        ItemDescription => ItemAttribute::Description(data.read_string()?),
                        ItemDuration => ItemAttribute::Duration(data.read_i32()?),
                        ItemDecayingState => ItemAttribute::DecayingState(data.read_byte()?),
                        ItemName => ItemAttribute::Name(data.read_string()?),
                        ItemArticle => ItemAttribute::Article(data.read_string()?),
                        ItemPluralName => ItemAttribute::PluralName(data.read_string()?),
                        ItemWeight => ItemAttribute::Weight(data.read_u32()?),
                        ItemAttack => ItemAttribute::Attack(data.read_i32()?),
                        ItemDefense => ItemAttribute::Defense(data.read_i32()?),
                        ItemExtraDefense => ItemAttribute::ExtraDefense(data.read_i32()?),
                        ItemArmor => ItemAttribute::Armor(data.read_i32()?),
                        ItemHitChance => ItemAttribute::HitChance(data.read_byte()?),
                        ItemShootRange => ItemAttribute::ShootRange(data.read_byte()?),
                        Teleport => ItemAttribute::Teleport(data.read_position()?),
                        HouseDoorId => ItemAttribute::HouseDoorId(data.read_byte()?),
                        DepotId => ItemAttribute::DepotId(data.read_u16()?),

                        _ => panic!("Unknown item attribute"),
                    };

                    item.attributes.push(attribute);
                }

                self.current_tile_items.push(item);
            }

            NodeKind::Town => {
                self.towns.push(Town::deserialize(&mut data)?);
            }

            NodeKind::WayPoint => {
                self.waypoints.push(Waypoint::deserialize(&mut data)?);
            }

            _ => println!("ignoring node {:?}", kind),
        }

        Ok(true)
    }
}

#[derive(Debug)]
pub struct Town {
    pub id: u32,
    pub name: String,
    pub temple_position: Position,
}

impl Town {
    pub fn deserialize<R>(mut r: R) -> io::Result<Town>
    where
        R: io::Read,
    {
        Ok(Town {
            id: r.read_u32()?,
            name: r.read_string()?,
            temple_position: r.read_position()?,
        })
    }
}

#[derive(Debug)]
pub struct Waypoint {
    pub name: String,
    pub position: Position,
}

impl Waypoint {
    pub fn deserialize<R>(mut r: R) -> io::Result<Waypoint>
    where
        R: io::Read,
    {
        Ok(Waypoint {
            name: r.read_string()?,
            position: r.read_position()?,
        })
    }
}
