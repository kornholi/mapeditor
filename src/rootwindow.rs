use glium;
use cgmath;
use cgmath::FixedArray;
use clock_ticks;

use std::thread;

use glium::Surface;
use glium::glutin;
use glium::index::{PrimitiveType, NoIndices};


use super::renderer::{Renderer, Vertex};

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
	vertex_buffer_len: usize,


}

impl RootWindow {
	pub fn new(display: glium::backend::glutin_backend::GlutinFacade, mut renderer: Renderer) -> RootWindow {
	    let vertex_buffer = {
	        implement_vertex!(Vertex, position, color, tex_coord);

			glium::VertexBuffer::empty(&display, 16384)
	    };

	    let vertex_buffer_len = {
			let start = clock_ticks::precise_time_ms();
			let data = renderer.render();
			let end = clock_ticks::precise_time_ms();

			println!("writing {} vertices (took {}ms to build)", data.len(), end-start);

			let start = clock_ticks::precise_time_ms();
			vertex_buffer.slice(0..data.len()).unwrap().write(data);
			let end = clock_ticks::precise_time_ms();

			println!("vbo upload took {}ms", end-start);

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