use std::thread;

use cgmath::FixedArray;

use glium;
use cgmath;
use clock_ticks;

use glium::Surface;
use glium::glutin;
use glium::index::PrimitiveType;

enum Action {
    Stop,
    Continue,
}

pub struct RootWindow {
	display: glium::backend::glutin_backend::GlutinFacade,
	texture: glium::texture::SrgbTexture2d,

	ortho_matrix: cgmath::Matrix4<f32>,

	program: glium::Program,
	vertex_buffer: glium::VertexBuffer<Vertex>,
	index_buffer: glium::IndexBuffer<u16>,
}

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 4],
    tex_coord: [f32; 2],
}

impl RootWindow {
	pub fn new(display: glium::backend::glutin_backend::GlutinFacade, texture: glium::texture::SrgbTexture2d) -> RootWindow {
		// building the vertex buffer, which contains all the vertices that we will draw
	    let vertex_buffer = {
	        implement_vertex!(Vertex, position, color, tex_coord);

	        glium::VertexBuffer::new(&display, 
	            vec![
	            	Vertex { position: [1., 1., 7.], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [0.0, 0.0] },
	                Vertex { position: [2., 2., 7.], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [0.0, 0.0] },
	                Vertex { position: [4., 3., 7.], color: [1.0, 0.0, 1.0, 1.0], tex_coord: [0.0, 0.0] },
	            ]
	        )
	    };

	    // building the index buffer
	    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::Points,
	                                               vec![0 as u16, 1, 2]); //1 as u16, 2, 0, 3]);

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
			texture: texture,

			program: program,

			vertex_buffer: vertex_buffer,
			index_buffer: index_buffer,
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
	            tex: self.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
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
	        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &draw_params).unwrap();
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