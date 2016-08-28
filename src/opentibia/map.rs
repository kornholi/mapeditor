use std::io;
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

enum_from_primitive! {
#[derive(Debug, PartialEq)]
enum NodeAttributeKind {
    MapDescription = 1,
    TileFlags = 3,
    ItemActionId = 4,
    ItemUniqueId = 5,
    ItemText = 6,
    ItemDescription = 7,
    TeleportDestination = 8,
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

    AttributeMap = 128
}
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
    TeleportDestination(Position),
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
        where R: io::Read
    {
        let mut loader = Loader { ..Default::default() };

        try!(binaryfile::streaming_parser(r,
                                          false,
                                          |kind, data| loader.load_headers_callback(kind, data)));

        Ok(loader)
    }

    pub fn load<F>(&mut self, r: &mut io::Read, mut tile_callback: F) -> io::Result<()>
        where F: FnMut(Position, &[Item])
    {
        binaryfile::streaming_parser(r, true, |kind, data| {
            self.load_callback(kind, data, |pos, items| tile_callback(pos, items))
        })
    }

    fn load_headers_callback(&mut self, kind: u8, mut data: &[u8]) -> io::Result<bool> {
        let kind = NodeKind::from_u8(kind).expect("unknown map node kind");

        match kind {
            NodeKind::Root => {
                self.version = try!(data.read_u32());
                self.width = try!(data.read_u16());
                self.height = try!(data.read_u16());
                self.items_version = (try!(data.read_u32()), try!(data.read_u32()));

                Ok(true)
            }

            NodeKind::MapData => {
                while !data.is_empty() {
                    use self::NodeAttributeKind::*;
                    let raw_attr = try!(data.read_byte());
                    let attribute = NodeAttributeKind::from_u8(raw_attr)
                        .expect("unknown attribute");

                    match attribute {
                        MapDescription => self.description.push(try!(data.read_string())),
                        HouseFile => self.house_file.push(try!(data.read_string())),
                        SpawnFile => self.spawn_file.push(try!(data.read_string())),
                        _ => panic!("Unknown map attribute"),
                    }
                }

                Ok(false)
            }

            _ => unreachable!(),
        }
    }

    fn load_callback<F>(&mut self,
                        kind: u8,
                        mut data: &[u8],
                        mut tile_callback: F)
                        -> io::Result<bool>
        where F: FnMut(Position, &[Item])
    {
        let kind = NodeKind::from_u8(kind).expect("unknown map node kind");

        match kind {
            NodeKind::TileArea => {
                let origin = try!(Position::deserialize(&mut data));
                self.current_tile_origin = Some(origin);
            }

            NodeKind::Tile | NodeKind::HouseTile => {
                let x_offset = try!(data.read_byte()) as u16;
                let y_offset = try!(data.read_byte()) as u16;

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
                    let _house_id = try!(data.read_u32());
                }

                while !data.is_empty() {
                    use self::NodeAttributeKind::*;
                    let raw_attr = try!(data.read_byte());
                    let attr = NodeAttributeKind::from_u8(raw_attr).expect("unknown attribute");

                    match attr {
                        TileFlags => {
                            let _flags = try!(data.read_u32());
                        }

                        Item => {
                            let item_id = try!(data.read_u16());
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

                let item_id = try!(data.read_u16());

                let mut item = Item {
                    id: item_id,
                    attributes: Vec::new(),
                };

                while !data.is_empty() {
                    use self::NodeAttributeKind::*;

                    let raw_attr = try!(data.read_byte());
                    let attribute = NodeAttributeKind::from_u8(raw_attr)
                        .expect("unknown attribute");

                    match attribute {
                        ItemCount | RuneCharges => {
                            let count = try!(data.read_byte());
                            item.attributes.push(ItemAttribute::Count(count));
                        }

                        ItemCharges => {
                            let charges = try!(data.read_u16());
                            item.attributes.push(ItemAttribute::Charges(charges));
                        }

                        ItemText => {
                            let text = try!(data.read_string());
                            item.attributes.push(ItemAttribute::Text(text));
                        }

                        ItemActionId => {
                            let action_id = try!(data.read_u16());
                            item.attributes.push(ItemAttribute::ActionId(action_id));
                        }

                        ItemUniqueId => {
                            let unique_id = try!(data.read_u16());
                            item.attributes.push(ItemAttribute::UniqueId(unique_id));
                        }

                        ItemWrittenDate => {
                            let date = try!(data.read_u32());
                            item.attributes.push(ItemAttribute::WrittenDate(date));
                        }

                        ItemWrittenBy => {
                            let author = try!(data.read_string());
                            item.attributes.push(ItemAttribute::WrittenBy(author));
                        }

                        ItemDescription => {
                            let description = try!(data.read_string());
                            item.attributes.push(ItemAttribute::Description(description));
                        }

                        ItemDuration => {
                            let duration = try!(data.read_i32());
                            item.attributes.push(ItemAttribute::Duration(duration));
                        }

                        ItemDecayingState => {
                            let state = try!(data.read_byte());
                            item.attributes.push(ItemAttribute::DecayingState(state));
                        }

                        ItemName => {
                            let name = try!(data.read_string());
                            item.attributes.push(ItemAttribute::Name(name));
                        }

                        ItemArticle => {
                            let article = try!(data.read_string());
                            item.attributes.push(ItemAttribute::Article(article));
                        }

                        ItemPluralName => {
                            let plural_name = try!(data.read_string());
                            item.attributes.push(ItemAttribute::PluralName(plural_name));
                        }

                        ItemWeight => {
                            let weight = try!(data.read_u32());
                            item.attributes.push(ItemAttribute::Weight(weight));
                        }

                        ItemAttack => {
                            let attack = try!(data.read_i32());
                            item.attributes.push(ItemAttribute::Attack(attack));
                        }                                

                        ItemDefense => {
                            let defense = try!(data.read_i32());
                            item.attributes.push(ItemAttribute::Defense(defense));
                        }

                        ItemExtraDefense => {
                            let defense = try!(data.read_i32());
                            item.attributes.push(ItemAttribute::ExtraDefense(defense));
                        }

                        ItemArmor => {
                            let armor = try!(data.read_i32());
                            item.attributes.push(ItemAttribute::Armor(armor));
                        }

                        ItemHitChance => {
                            let hit_chance = try!(data.read_byte());
                            item.attributes.push(ItemAttribute::HitChance(hit_chance));
                        }

                        ItemShootRange => {
                            let shoot_range = try!(data.read_byte());
                            item.attributes.push(ItemAttribute::ShootRange(shoot_range));
                        }

                        TeleportDestination => {
                            let destination = try!(Position::deserialize(&mut data));
                            item.attributes.push(ItemAttribute::TeleportDestination(destination));
                        }

                        HouseDoorId => {
                            let door_id = try!(data.read_byte());
                            item.attributes.push(ItemAttribute::HouseDoorId(door_id));
                        }

                        DepotId => {
                            let depot_id = try!(data.read_u16());
                            item.attributes.push(ItemAttribute::DepotId(depot_id));
                        }

                        _ => panic!("Unknown item attribute"),
                    }
                }

                self.current_tile_items.push(item);
            }

            NodeKind::Town => {
                let town = try!(Town::deserialize(&mut data));
                self.towns.push(town);
            }

            NodeKind::WayPoint => {
                let wp = try!(Waypoint::deserialize(&mut data));
                self.waypoints.push(wp);
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
        where R: io::Read
    {
        let id = try!(r.read_u32());
        let name = try!(r.read_string());
        let pos = try!(Position::deserialize(r));

        Ok(Town {
            id: id,
            name: name,
            temple_position: pos,
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
        where R: io::Read
    {
        let name = try!(r.read_string());
        let pos = try!(Position::deserialize(r));
        Ok(Waypoint {
            name: name,
            position: pos,
        })
    }
}
