use glium;
use cgmath::{self, Zero};
use clock_ticks;

use std::{cmp, f32, thread, io, fs};
use std::time::Duration;

use glium::Surface;
use glium::index::{PrimitiveType, NoIndices};

use spritecontainer::SpriteContainer;

use super::renderer::Renderer;
use super::spriteatlas::SpriteAtlas;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, color, tex_coord);
enum Action {
    Stop,
    Continue,
}

pub struct RootWindow {
    display: glium::backend::glutin_backend::GlutinFacade,

    renderer: Renderer,
    spr: SpriteContainer<io::BufReader<fs::File>>,
    spr_atlas: SpriteAtlas,

    ortho_matrix: cgmath::Matrix4<f32>,
    program: glium::Program,

    temp_buffer: Vec<Vertex>,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    vertex_buffer_len: usize,

    dimensions: (f32, f32),
    zoom_level: i32,
    scaling_factor: f32,
    ul_offset: (f32, f32),

    dragging: bool,
    dragging_position: Option<(i32, i32)>,
}

impl RootWindow {
    pub fn new(display: glium::backend::glutin_backend::GlutinFacade,
               renderer: Renderer,
               spr: SpriteContainer<io::BufReader<fs::File>>)
               -> RootWindow {
        let vertex_buffer = glium::VertexBuffer::empty(&display, 16384).expect("VBO creation failed");

        // {
        // use image;
        // use std;
        // use std::path::Path;
        //
        // let image: image::DynamicImage = renderer.atlas.texture.read();
        // let mut output = std::fs::File::create(&Path::new("atlas.png")).unwrap();
        // image.save(&mut output, image::ImageFormat::PNG).unwrap();
        // }

        // Compiling shaders and linking them together
        let program = program!(&display,
            330 => {
                vertex: include_str!("shaders/330.vert"),
                geometry: include_str!("shaders/330.geom"),
                fragment: include_str!("shaders/330.frag")
            },
        ).unwrap();

        RootWindow {            
            spr: spr,
            spr_atlas: SpriteAtlas::new(&display),
            renderer: renderer,

            display: display,
            program: program,

            temp_buffer: Vec::new(),
            vertex_buffer: vertex_buffer,
            vertex_buffer_len: 0,

            ortho_matrix: cgmath::Matrix4::zero(),

            dimensions: (0., 0.),
            zoom_level: 0,
            scaling_factor: 1.0,
            ul_offset: (80. * 32., 110. * 32.0),

            dragging: false,
            dragging_position: None,
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.dimensions = (w as f32, h as f32);
        self.calculate_projection();
    }

    fn calculate_projection(&mut self) {
        self.scaling_factor = 1.33_f32.powi(self.zoom_level);

        let (w, h) = self.dimensions;
        let (w, h) = (w * self.scaling_factor, h * self.scaling_factor);
        let ul = self.ul_offset;

        self.ortho_matrix = cgmath::ortho(ul.0, ul.0 + w, ul.1 + h, ul.1, -1.0, 1.0);

        // FIXME FIXME FIXME FIXME
        let (w, h) = (w / 32., h / 32.);
        let (w, h) = (w.ceil() as u16, h.ceil() as u16);

        let (u, l) = (ul.0 / 32., ul.1 / 32.);
        let (u, l) = (u as i32, l as i32);

        let spr = &mut self.spr;
        let atlas = &mut self.spr_atlas;
        let out = &mut self.temp_buffer;

        self.renderer.resize((u, l), (w, h), |(x, y), sprite_id| {
            let tex_pos = atlas.get_or_load(sprite_id, |buf, stride| {
                spr.get_sprite(sprite_id, buf, stride).expect("failed to load sprite {}", sprite_id)
            });

            out.push(Vertex {
                position: [x, y, 7.],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coord: tex_pos,
            });
        });
    }

    pub fn run(&mut self) {
        // the main loop
        start_loop(|| self.loop_callback());
    }

    fn loop_callback(&mut self) -> Action {
        // polling and handling the events received by the window
        while let Some(event) = self.display.poll_events().next() {
            use glium::glutin::Event::*;
            use glium::glutin::{MouseButton, MouseScrollDelta};
            // println!("ev: {:?}", event);

            match event {
                Closed => return Action::Stop,

                Resized(w, h) => self.resize(w, h),

                MouseMoved(x, y) => {
                    if self.dragging {
                        if let Some((prev_x, prev_y)) = self.dragging_position {
                            let offset = (prev_x - x, prev_y - y);

                            self.ul_offset.0 += offset.0 as f32 * self.scaling_factor;
                            self.ul_offset.1 += offset.1 as f32 * self.scaling_factor;

                            self.calculate_projection();
                        }

                        self.dragging_position = Some((x, y));
                    }
                }

                MouseInput(state, MouseButton::Middle) |
                MouseInput(state, MouseButton::Left) => {
                    use glium::glutin::ElementState::*;

                    match state {
                        Pressed => self.dragging = true,
                        Released => {
                            self.dragging = false;
                            self.dragging_position = None;
                        }
                    }
                }

                // FIXME: Support PixelDelta
                MouseWheel(MouseScrollDelta::LineDelta(h, v), _) => {
                    self.zoom_level = cmp::max(-4, self.zoom_level - v as i32);
                    self.calculate_projection();
                }

                Focused(false) => {
                    self.dragging = false;
                    self.dragging_position = None;
                }

                _ => (),
            }
        }

        if !self.temp_buffer.is_empty() {
            let data = &mut self.temp_buffer;

            let start = clock_ticks::precise_time_ms();
            self.vertex_buffer.slice(0..data.len()).unwrap().write(data);
            let end = clock_ticks::precise_time_ms();

            println!("VBO upload took {}ms", end - start);

            self.vertex_buffer_len = data.len();
            data.clear();
        }

        let ortho_matrix: &[[f32; 4]; 4] = self.ortho_matrix.as_ref();

        // building the uniforms
        let uniforms = uniform! {
            matrix: *ortho_matrix,
            tex: self.spr_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
        };

        let draw_params =
            glium::DrawParameters { blend: glium::Blend::alpha_blending(), ..Default::default() };

        // drawing a frame
        let mut target = self.display.draw();
        target.clear_color(0.5, 0.5, 0.5, 1.0);
        target.draw(self.vertex_buffer.slice(0..self.vertex_buffer_len).unwrap(),
                  NoIndices(PrimitiveType::Points),
                  &self.program,
                  &uniforms,
                  &draw_params)
            .unwrap();
        target.finish().unwrap();

        Action::Continue
    }
}

fn start_loop<F>(mut callback: F)
    where F: FnMut() -> Action
{
    let mut accumulator = 0;
    let mut previous_clock = clock_ticks::precise_time_ns();

    loop {
        match callback() {
            Action::Stop => break,
            Action::Continue => (),
        };

        let now = clock_ticks::precise_time_ns();

        accumulator += now - previous_clock;
        previous_clock = now;

        const FIXED_TIME_STAMP: u64 = 16666667;
        while accumulator >= FIXED_TIME_STAMP {
            accumulator -= FIXED_TIME_STAMP;

            // if you have a game, update the state here
        }

        thread::sleep(Duration::from_millis(((FIXED_TIME_STAMP - accumulator) / 1000000)));
    }
}
