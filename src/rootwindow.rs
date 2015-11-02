use glium;
use cgmath;
use cgmath::Matrix;
use clock_ticks;

use std::thread;

use glium::Surface;
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

    dimensions: (f32, f32),
    ul_offset: (f32, f32),

    dragging: bool,
    dragging_position: Option<(i32, i32)>
}

impl RootWindow {
    pub fn new(display: glium::backend::glutin_backend::GlutinFacade, renderer: Renderer) -> RootWindow {
        let vertex_buffer = {
            implement_vertex!(Vertex, position, color, tex_coord);

            glium::VertexBuffer::empty(&display, 16384).expect("VBO creation failed")
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
            vertex_buffer_len: 0,
            
            ortho_matrix: cgmath::Matrix4::zero(),

            dimensions: (0., 0.),
            ul_offset: (80.*32., 110.*32.0),

            dragging: false,
            dragging_position: None
            //..Default::default()
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.dimensions = (w as f32, h as f32);
        self.calculate_projection();        
    }

    fn calculate_projection(&mut self) {
        let (w, h) = self.dimensions;
        let ul = self.ul_offset;

        self.ortho_matrix = cgmath::ortho(ul.0, ul.0 + w, ul.1 + h, ul.1, -1.0, 1.0);

        // FIXME FIXME FIXME FIXME

        let (w, h) = (w / 32., h / 32.);
        let (w, h) = (w.ceil() as u16, h.ceil() as u16);

        let (u, l) = (ul.0 / 32., ul.1 / 32.);
        let (u, l) = (u as i32, l as i32);

        self.renderer.resize((u, l), (w, h));
    }

    pub fn run(&mut self) {
        // the main loop
        start_loop(|| { self.loop_callback() });
    }

    fn loop_callback(&mut self) -> Action {
        // polling and handling the events received by the window
        while let Some(event) = self.display.poll_events().next() {
            use glium::glutin::Event::*;
            use glium::glutin::MouseButton;
            //println!("ev: {:?}", event);

            match event {
                Closed => return Action::Stop,

                Resized(w, h) => {
                    self.resize(w, h)
                }

                MouseMoved((x, y)) => {
                    if self.dragging {
                        if let Some((prev_x, prev_y)) = self.dragging_position {
                            let offset = (prev_x - x, prev_y - y);

                            self.ul_offset.0 += offset.0 as f32;
                            self.ul_offset.1 += offset.1 as f32;

                            self.calculate_projection();
                        }

                        self.dragging_position = Some((x, y));
                    }
                }

                MouseInput(state, MouseButton::Middle) | MouseInput(state, MouseButton::Left) => {
                    use glium::glutin::ElementState::*;
                    
                    match state {
                        Pressed => self.dragging = true,
                        Released => {
                            self.dragging = false;
                            self.dragging_position = None;
                        }
                    }
                }

                Focused(false) => {
                    self.dragging = false;
                    self.dragging_position = None;
                }

                _ => ()
            }
        }

        if self.renderer.new_data {
            let data = &self.renderer.vertices[..];

            let start = clock_ticks::precise_time_ms();
            self.vertex_buffer.slice(0..data.len()).unwrap().write(data);
            let end = clock_ticks::precise_time_ms();

            println!("vbo upload took {}ms", end-start);

            self.vertex_buffer_len = data.len();
            self.renderer.new_data = false;
        }

        let ortho_matrix: &[[f32; 4]; 4] = self.ortho_matrix.as_ref();

        // building the uniforms
        let uniforms = uniform! {
            matrix: *ortho_matrix,
            tex: self.renderer.atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        let draw_params = glium::DrawParameters {
            blend: glium::Blend::alpha_blending(),
            .. Default::default()
        };

        // drawing a frame
        let mut target = self.display.draw();
        target.clear_color(0.5, 0.5, 0.5, 1.0);
        target.draw(self.vertex_buffer.slice(0..self.vertex_buffer_len).unwrap(), NoIndices(PrimitiveType::Points), &self.program, &uniforms, &draw_params).unwrap();
        target.finish().unwrap();
        
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
