use std::io;
use helpers::ReadExt;

use num::FromPrimitive;

pub struct DatContainer {
    pub signature: u32,

    // index 0 = item id 100
    pub items: Vec<Thing>
}

enum_from_primitive! {
#[derive(Debug, PartialEq)]
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

    pub sprite_ids: Vec<u32>
}

impl Thing {
    pub fn deserialize(r: &mut io::Read) -> io::Result<Thing> {
        let mut displacement = (0, 0);
        let mut elevation = 0;

        loop {
            // TODO: return custom error
            let raw_attr = try!(r.read_byte());
            let attr = Attribute::from_u8(raw_attr).expect(&format!("unknown attribute {}", raw_attr));

            {
                use self::Attribute::*;

                match attr {
                    End => break,

                    Ground | Writable | WritableOnce => {
                        let _speed = try!(r.read_u16());
                    },

                    Light => {
                        let _intensity = try!(r.read_u16());
                        let _color = try!(r.read_u16());
                    }

                    Displacement => {
                        let x = try!(r.read_u16());
                        let y = try!(r.read_u16());

                        displacement = (x, y);
                    }

                    Elevation => {
                        elevation = try!(r.read_u16());
                    }

                    DefaultAction | MinimapColor | Cloth | LensHelp => {
                        try!(r.read_u16());
                    }

                    Market => {
                        let _category = try!(r.read_u16());
                        let _trade_id = try!(r.read_u16());
                        let _show_id = try!(r.read_u16());
                        let _name = try!(r.read_string());

                        let _voc = try!(r.read_u16());
                        let _level = try!(r.read_u16());
                    }

                    _ => {}
                }
            }
        }

        let width = try!(r.read_byte());
        let height = try!(r.read_byte());

        if width > 1 || height > 1 {
            try!(r.read_byte());
        }

        let layers = try!(r.read_byte());
        let pattern_width = try!(r.read_byte());
        let pattern_height = try!(r.read_byte());
        let pattern_depth = try!(r.read_byte());

        let animation_length = try!(r.read_byte());

        if animation_length > 1 {
            let _async = try!(r.read_byte()) == 0;
            let _loop_count = try!(r.read_i32());
            let _start_phase = try!(r.read_byte());

            for _ in 0..animation_length {
                let _min = try!(r.read_u32());
                let _max = try!(r.read_u32());
            }
        }

        let sprite_count = width as u16 * height as u16 *
            pattern_width as u16 * pattern_height as u16 * pattern_depth as u16 *
            layers as u16 * animation_length as u16;

        let mut sprite_ids = Vec::with_capacity(sprite_count as usize);

        for _ in 0..sprite_count {
            let id = try!(r.read_u32());
            sprite_ids.push(id);
        }

        Ok(Thing {
            width: width,
            height: height,
            layers: layers,

            pattern_width: pattern_width,
            pattern_height: pattern_height,
            pattern_depth: pattern_depth,

            displacement: displacement,
            elevation: elevation,

            sprite_ids: sprite_ids
        })
    }
}

impl DatContainer {
    pub fn new(r: &mut io::Read) -> io::Result<DatContainer> {
        let signature = try!(r.read_u32());

        let num_items = try!(r.read_u16());
        let _num_creatures = try!(r.read_u16());
        let _num_magic_effects = try!(r.read_u16());
        let _num_distance_effects = try!(r.read_u16());

        let mut client_id = 100;
        let mut items = Vec::with_capacity((num_items - client_id) as usize);

        while client_id <= num_items {
            let thing = try!(Thing::deserialize(r));
            items.push(thing);

            client_id += 1;
        }

        Ok(DatContainer {
            signature: signature,
            items: items
        })
    }
}
