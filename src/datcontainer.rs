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

pub struct Thing {
	pub sprite_ids: Vec<u32>
}

impl Thing {
	pub fn deserialize(r: &mut io::Read) -> io::Result<Thing> {
		loop {
			// TODO: return custom error
			let raw_attr = try!(r.read_byte());
			let attr = Attribute::from_u8(raw_attr).expect(&format!("unknown attribute {}", raw_attr));

			{
				use self::Attribute::*;

				match attr {
					End => break,

					Ground | Writable | WritableOnce => {
						let speed = try!(r.read_u16());
					},

					Light => {
						let intensity = try!(r.read_u16());
						let color = try!(r.read_u16());
					}

					Displacement => {
						let x = try!(r.read_u16());
						let y = try!(r.read_u16());
					}

					DefaultAction | Elevation | MinimapColor | Cloth | LensHelp => {
						try!(r.read_u16());
					}

					Market => {
						let category = try!(r.read_u16());
						let trade_id = try!(r.read_u16());
						let show_id = try!(r.read_u16());
						let name = try!(r.read_string());

						let voc = try!(r.read_u16());
						let level = try!(r.read_u16());
					}

					a @ _ => {} //println!("got attr {:?}", a)
				}
			}
		}

		let width = try!(r.read_byte());
		let height = try!(r.read_byte());

		if width > 1 || height > 1 {
			try!(r.read_byte());
		}

		let frames = try!(r.read_byte());
		let pattern_width = try!(r.read_byte());
		let pattern_height = try!(r.read_byte());
		let pattern_depth = try!(r.read_byte());

		let animation_length = try!(r.read_byte());

		if animation_length > 1 {
			let async = try!(r.read_byte()) == 0;
			let loop_count = try!(r.read_i32());
			let start_phase = try!(r.read_byte());

			for _ in (0..animation_length) {
				let min = try!(r.read_u32());
				let max = try!(r.read_u32());
			}
		}

		let sprite_count = width as u16 * height as u16 *
			pattern_width as u16 * pattern_height as u16 * pattern_depth as u16 *
			frames as u16 * animation_length as u16;

		let mut sprite_ids = Vec::with_capacity(sprite_count as usize);

		for i in (0..sprite_count) {
			let id = try!(r.read_u32());
			sprite_ids.push(id);
		}

		Ok(Thing { sprite_ids: sprite_ids })
	}
}

impl DatContainer {
    pub fn new(r: &mut io::Read) -> io::Result<DatContainer> {
    	let signature = try!(r.read_u32());

    	let num_items = try!(r.read_u16());
    	let num_creatures = try!(r.read_u16());
    	let num_magic_effects = try!(r.read_u16());
    	let num_distance_effects = try!(r.read_u16());

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