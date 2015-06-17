use glium;
use cgmath;
use cgmath::FixedArray;
use clock_ticks;

use std::{io, fs, thread};

use glium::Surface;
use glium::glutin;
use glium::index::{PrimitiveType, NoIndices};

use datcontainer;
use datcontainer::DatContainer;

use spritecontainer::SpriteContainer;
use opentibia;
use opentibia::itemtypes;
use super::map;

use super::spriteatlas::SpriteAtlas;

enum Action {
    Stop,
    Continue,
}

pub struct RootWindow {
	display: glium::backend::glutin_backend::GlutinFacade,

	renderer: Renderer,
	ortho_matrix: cgmath::Matrix4<f32>,
	program: glium::Program,
	vertex_buffer: glium::VertexBuffer<Vertex>,
	vertex_buffer_len: usize
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
    tex_coord: [f32; 2],
}

pub struct Renderer {
    pub spr: SpriteContainer,
    pub spr_data: io::BufReader<fs::File>,
    pub dat: DatContainer,
    pub otb: itemtypes::Container,

    pub atlas: SpriteAtlas,
    pub map: map::Map,
	
	pub vertices: Vec<Vertex>,
}

fn get_sprite_id(obj: &datcontainer::Thing, layer: u8, pattern_x: u16, pattern_y: u16, x: u8, y: u8) -> usize {
	let animation_time = 0;

	((((((animation_time % 4095) * obj.pattern_height as u16
		+ pattern_y % obj.pattern_height as u16) * obj.pattern_width as u16
		+ pattern_x % obj.pattern_width as u16) * obj.layers as u16
		+ layer as u16) * obj.height as u16
		+ y as u16) * obj.width as u16
		+ x as u16) as usize % obj.sprite_ids.len()
}

impl Renderer {
    pub fn render(&mut self) -> &[Vertex] {
        let tmp = opentibia::Position { x: 95, y: 117, z: 7 };
        let sec = self.map.get(tmp).unwrap();

        let mut pos = 0usize;

        self.vertices.clear();

        // must iterate with y
        for ref tile in sec.tiles.iter() {
        	let tile_x = pos / 32;
        	let tile_y = pos % 32;

        	let mut elevation = 0;

        	for ref item in tile.iter() {
        		let otb_entry = &self.otb.items[item.id as usize];

        		if let Some(client_id) = otb_entry.client_id {
        			let obj = &self.dat.items[(client_id - 100) as usize];
        			//println!("dat: {:?}", obj);

        			let pattern_x = ((tmp.x & !31) + tile_x as u16) % obj.pattern_width as u16;
        			let pattern_y = ((tmp.y & !31) + tile_y as u16) % obj.pattern_height as u16;

        			for layer in 0..obj.layers {
        				for y in 0..obj.height {
        					for x in 0..obj.width {
        						let spr_idx = get_sprite_id(obj, layer, pattern_x, pattern_y, x, y);
        						let spr_id = obj.sprite_ids[spr_idx] as u32;

        						if spr_id != 0 {
									let mut tex_pos = self.atlas.get(spr_id);

									if tex_pos == [0., 0.] {
										let sprite = self.spr.get_sprite(&mut self.spr_data, spr_id).unwrap();
										tex_pos = self.atlas.add(spr_id, sprite);
									}

			        				let obj_x = tile_x as f32 - x as f32 - (obj.displacement.0 + elevation) as f32 / 32.;
			        				let obj_y = tile_y as f32 - y as f32 - (obj.displacement.1 + elevation) as f32 / 32.;

									self.vertices.push(Vertex { position: [obj_x, obj_y, 7.], color: [1.0, 1.0, 1.0, 1.0], tex_coord: tex_pos });
			        			}
        					}
        				}
        			}

        			elevation += obj.elevation;
        		}
			}

        	pos += 1;
        }

        &self.vertices[..]
	}
}

impl RootWindow {
	pub fn new(display: glium::backend::glutin_backend::GlutinFacade, mut renderer: Renderer) -> RootWindow {
		// building the vertex buffer, which contains all the vertices that we will draw
	    let vertex_buffer = {
	        implement_vertex!(Vertex, position, color, tex_coord);

			glium::VertexBuffer::empty(&display, 16384)
	    };

	    let vertex_buffer_len = {
	    	let start = clock_ticks::precise_time_ms();
			let data = renderer.render();
			let end = clock_ticks::precise_time_ms();

			println!("writing {} vertices (took {}ms to build)", data.len(), end-start);

			vertex_buffer.slice(0..data.len()).unwrap().write(data);	
			data.len()
		};

		/*{
			use image;
	    	use std;
	    	use std::path::Path;

			let image: image::DynamicImage = renderer.atlas.texture.read();
			let mut output = std::fs::File::create(&Path::new("atlas.png")).unwrap();
    		image.save(&mut output, image::ImageFormat::PNG).unwrap();
		}*/

	    // compiling shaders and linking them together
	    let program = program!(&display,
	        330 => {
	            vertex: include_str!("shaders/330.vert"),
	            geometry: include_str!("shaders/330.geom"),
	            fragment: include_str!("shaders/330.frag")
	        },
	    ).unwrap();

		RootWindow {
			display: display,
			renderer: renderer,

			program: program,

			vertex_buffer: vertex_buffer,
			vertex_buffer_len: vertex_buffer_len,
			//..Default::default()
			ortho_matrix: cgmath::Matrix4::zero() 
		}
	}

	pub fn resize(&mut self, w: u32, h: u32) {
		self.ortho_matrix = cgmath::ortho(0.0, w as f32, h as f32, 0.0, -1.0, 1.0);
	}

	pub fn run(&mut self) {
		// the main loop
	    start_loop(|| { self.loop_callback() });
	}

	fn loop_callback(&mut self) -> Action {
		{
			// building the uniforms
	        let uniforms = uniform! {
	            matrix: *self.ortho_matrix.as_fixed(),
	            tex: self.renderer.atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
	        };

	        let draw_params = glium::DrawParameters {
	            blending_function:
	                Some(glium::BlendingFunction::Addition { 
	                                        source: glium::LinearBlendingFactor::SourceAlpha,
	                                        destination: glium::LinearBlendingFactor::OneMinusSourceAlpha }),
	            .. Default::default()
	        };

	        // drawing a frame
	        let mut target = self.display.draw();
	        target.clear_color(0.5, 0.5, 0.5, 1.0);
	        target.draw(self.vertex_buffer.slice(0..self.vertex_buffer_len).unwrap(), NoIndices(PrimitiveType::Points), &self.program, &uniforms, &draw_params).unwrap();
	        target.finish();
		}
	       
	    // polling and handling the events received by the window
        while let Some(event) = self.display.poll_events().next() {
            //println!("ev: {:?}", event);

            match event {
                glutin::Event::Closed => return Action::Stop,

                glutin::Event::Resized(w, h) => {
                	self.resize(w, h)
                }

                _ => ()
            }
        }

        Action::Continue
	}
}

fn start_loop<F>(mut callback: F) where F: FnMut() -> Action {
    let mut accumulator = 0;
    let mut previous_clock = clock_ticks::precise_time_ns();

    loop {
        match callback() {
            Action::Stop => break,
            Action::Continue => ()
        };

        let now = clock_ticks::precise_time_ns();

        accumulator += now - previous_clock;
        previous_clock = now;

        const FIXED_TIME_STAMP: u64 = 16666667;
        while accumulator >= FIXED_TIME_STAMP {
            accumulator -= FIXED_TIME_STAMP;

            // if you have a game, update the state here
        }

        thread::sleep_ms(((FIXED_TIME_STAMP - accumulator) / 1000000) as u32);
    }
}