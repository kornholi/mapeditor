use std::io;
use crate::helpers::ReadExt;

use num_derive::FromPrimitive;
use num::FromPrimitive;

pub struct DatContainer {
    pub signature: u32,

    // index 0 = item id 100
    pub items: Vec<Thing>,
}

#[derive(Debug, FromPrimitive, PartialEq)]
pub enum Attribute {
    Ground = 0,
    GroundBorder,
    OnBottom,
    OnTop,
    Container,
    Stackable,
    ForceUse,
    MultiUse,
    Writable,
    WritableOnce,
    FluidContainer = 10,
    Splash,
    NotWalkable,
    NotMovable,
    BlockProjectile,
    NotPathable,
    NoMoveAnimation,
    Pickupable,
    Hangable,
    HookSouth,
    HookEast = 20,
    Rotateable,
    Light,
    DontHide,
    Translucent,
    Displacement,
    Elevation,
    LyingCorpse,
    AnimateAlways,
    MinimapColor,
    LensHelp = 30,
    FullGround,
    LookThrough,
    Cloth,
    Market,
    DefaultAction,

    Usable = 0xFE,
    End = 0xFF
}

#[derive(Debug)]
pub struct Thing {
    pub width: u8,
    pub height: u8,
    pub layers: u8,

    pub pattern_width: u8,
    pub pattern_height: u8,
    pub pattern_depth: u8,

    pub displacement: (u16, u16),
    pub elevation: u16,

    pub sprite_ids: Vec<u32>,
}

impl Thing {
    pub fn deserialize(r: &mut dyn io::Read) -> io::Result<Thing> {
        let mut displacement = (0, 0);
        let mut elevation = 0;

        loop {
            // TODO: return custom error
            let raw_attr = r.read_byte()?;
            let attr = Attribute::from_u8(raw_attr)
                .expect(&format!("unknown attribute {}", raw_attr));

            {
                use self::Attribute::*;

                match attr {
                    End => break,

                    Ground | Writable | WritableOnce => {
                        let _speed = r.read_u16()?;
                    }

                    Light => {
                        let _intensity = r.read_u16()?;
                        let _color = r.read_u16()?;
                    }

                    Displacement => {
                        let x = r.read_u16()?;
                        let y = r.read_u16()?;

                        displacement = (x, y);
                    }

                    Elevation => {
                        elevation = r.read_u16()?;
                    }

                    DefaultAction | MinimapColor | Cloth | LensHelp => {
                        r.read_u16()?;
                    }

                    Market => {
                        let _category = r.read_u16()?;
                        let _trade_id = r.read_u16()?;
                        let _show_id = r.read_u16()?;
                        let _name = r.read_string()?;

                        let _voc = r.read_u16()?;
                        let _level = r.read_u16()?;
                    }

                    _ => {}
                }
            }
        }

        let width = r.read_byte()?;
        let height = r.read_byte()?;

        if width > 1 || height > 1 {
            r.read_byte()?;
        }

        let layers = r.read_byte()?;
        let pattern_width = r.read_byte()?;
        let pattern_height = r.read_byte()?;
        let pattern_depth = r.read_byte()?;

        let animation_length = r.read_byte()?;

        if animation_length > 1 {
            let _async = r.read_byte()? == 0;
            let _loop_count = r.read_i32()?;
            let _start_phase = r.read_byte()?;

            for _ in 0..animation_length {
                let _min = r.read_u32()?;
                let _max = r.read_u32()?;
            }
        }

        let sprite_count =
            width as u16 * height as u16 * pattern_width as u16 * pattern_height as u16 *
            pattern_depth as u16 * layers as u16 * animation_length as u16;

        let mut sprite_ids = Vec::with_capacity(sprite_count as usize);

        for _ in 0..sprite_count {
            sprite_ids.push(r.read_u32()?);
        }

        Ok(Thing {
            width,
            height,
            layers,

            pattern_width,
            pattern_height,
            pattern_depth,

            displacement,
            elevation,

            sprite_ids,
        })
    }
}

impl DatContainer {
    pub fn new(r: &mut dyn io::Read) -> io::Result<DatContainer> {
        let signature = r.read_u32()?;

        let num_items = r.read_u16()?;
        let _num_creatures = r.read_u16()?;
        let _num_magic_effects = r.read_u16()?;
        let _num_distance_effects = r.read_u16()?;

        let mut client_id = 100;
        let mut items = Vec::with_capacity((num_items - client_id) as usize);

        while client_id <= num_items {
            items.push(Thing::deserialize(r)?);
            client_id += 1;
        }

        Ok(DatContainer {
            signature,
            items,
        })
    }
}
