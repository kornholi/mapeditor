use std::time::Instant;

use cgmath::{self, Zero};

use std::{cmp, f32, fs, io};

use glium::glutin;
use glium::glutin::dpi::PhysicalPosition;
use glium::index::{NoIndices, PrimitiveType};
use glium::Surface;

use crate::spritecontainer::SpriteContainer;

use super::renderer::Renderer;
use super::spriteatlas::SpriteAtlas;

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coord: [f32; 2],
}

implement_vertex!(Vertex, position, color, tex_coord);

pub struct RootWindow {
    display: glium::backend::glutin::Display,

    renderer: Renderer<Vertex>,
    spr: SpriteContainer<io::BufReader<fs::File>>,
    spr_atlas: SpriteAtlas,

    ortho_matrix: cgmath::Matrix4<f32>,
    program: glium::Program,

    vertex_buffer: glium::VertexBuffer<Vertex>,
    vertex_buffer_len: usize,

    dimensions: (f32, f32),
    zoom_level: i32,
    scaling_factor: f32,
    ul_offset: (f32, f32),

    last_mouse_position: Option<PhysicalPosition<f64>>,
    dragging: bool,
}

impl RootWindow {
    pub fn new(
        display: glium::backend::glutin::Display,
        renderer: Renderer<Vertex>,
        spr: SpriteContainer<io::BufReader<fs::File>>,
    ) -> RootWindow {
        let vertex_buffer =
            glium::VertexBuffer::empty_persistent(&display, 1 << 24).expect("VBO creation failed");

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
        )
        .unwrap();

        RootWindow {
            spr,
            spr_atlas: SpriteAtlas::new(&display),
            renderer,

            display,
            program,

            vertex_buffer,
            vertex_buffer_len: 0,

            ortho_matrix: cgmath::Matrix4::zero(),

            dimensions: (0., 0.),
            zoom_level: 0,
            scaling_factor: 1.0,
            ul_offset: (80. * 32., 110. * 32.0),

            last_mouse_position: None,
            dragging: false,
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        self.dimensions = (w as f32, h as f32);
        self.calculate_projection();
    }

    fn get_zooming_factor(&self) -> f32 {
        1.33_f32.powi(self.zoom_level)
    }

    fn calculate_projection(&mut self) {
        self.scaling_factor = self.get_zooming_factor();

        let (w, h) = self.dimensions;
        let (w, h) = (w * self.scaling_factor, h * self.scaling_factor);
        let ul = self.ul_offset;

        self.ortho_matrix = cgmath::ortho(ul.0, ul.0 + w, ul.1 + h, ul.1, -1.0, 1.0);

        if !self.dragging {
            self.upload_vertices();
        }
    }

    fn upload_vertices(&mut self) {
        let (w, h) = self.dimensions;
        let (w, h) = (w * self.scaling_factor, h * self.scaling_factor);
        let ul = self.ul_offset;

        // FIXME FIXME FIXME FIXME
        let (w, h) = (w / 32., h / 32.);
        let (w, h) = (w.ceil() as u16, h.ceil() as u16);

        let (u, l) = (ul.0 / 32., ul.1 / 32.);
        let (u, l) = (u as i32, l as i32);

        let spr = &mut self.spr;
        let atlas = &mut self.spr_atlas;

        let vis = self.renderer.get_visible_sectors((u, l), (w, h));

        let mut sprite_callback = |(x, y), sprite_id| {
            let tex_pos = atlas.get_or_load(sprite_id, |buf, stride| {
                spr.get_sprite(sprite_id, buf, stride)
                    .expect("failed to load sprite")
            });

            Vertex {
                position: [x, y, 7.],
                color: [1.0, 1.0, 1.0, 1.0],
                tex_coord: tex_pos,
            }
        };

        let start = Instant::now();
        let mut vbo_offset = 0;

        self.vertex_buffer.invalidate();

        for sector_pos in &vis {
            let vertices = self
                .renderer
                .get_sector_vertices(*sector_pos, &mut sprite_callback);

            if let Some(vertices) = vertices {
                if vertices.is_empty() {
                    continue;
                }

                if vbo_offset + vertices.len() > self.vertex_buffer.len() {
                    println!("warning: ran out of VBO space after {}", vbo_offset);
                    break;
                }

                self.vertex_buffer
                    .slice(vbo_offset..vbo_offset + vertices.len())
                    .unwrap()
                    .write(vertices);
                vbo_offset += vertices.len();
            }
        }

        self.vertex_buffer_len = vbo_offset;

        println!(
            "Rendering {} sectors took {:.2}ms - {} vertices",
            vis.len(),
            start.elapsed().as_secs_f64() * 1000.0,
            vbo_offset
        );
    }

    pub fn run(mut self, event_loop: glutin::event_loop::EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            use glutin::event::Event;
            use glutin::event_loop::ControlFlow;
            use glutin::event::WindowEvent::*;
            use glutin::event::{MouseButton, MouseScrollDelta};

            let next_frame_time = std::time::Instant::now() +
                std::time::Duration::from_nanos(16_666_667);

            *control_flow = ControlFlow::WaitUntil(next_frame_time);

            match event {
                Event::WindowEvent { event, .. } => match event {
                    CloseRequested => {
                        *control_flow = ControlFlow::Exit;
                        return;
                    },

                    Resized(new_size) => self.resize(new_size.width as u32, new_size.height as u32),

                    CursorMoved { position, .. } => {
                        if self.dragging {
                            if let Some(prev_position) = self.last_mouse_position {
                                let x_offset = prev_position.x - position.x;
                                let y_offset = prev_position.y - position.y;

                                self.ul_offset.0 += x_offset as f32 * self.scaling_factor;
                                self.ul_offset.1 += y_offset as f32 * self.scaling_factor;

                                self.calculate_projection();
                            }
                        }

                        self.last_mouse_position = Some(position);
                    }

                    MouseInput { state, button: MouseButton::Middle, .. } |
                    MouseInput { state, button: MouseButton::Left, .. } => {
                        use glutin::event::ElementState::*;

                        match state {
                            Pressed => self.dragging = true,
                            Released => {
                                self.dragging = false;

                                self.calculate_projection(); // FIXME: get rid of this
                            }
                        }
                    }

                    // FIXME: Support PixelDelta
                    MouseWheel { delta: MouseScrollDelta::LineDelta(_, v), .. } => {
                        self.zoom_level = cmp::max(-4, self.zoom_level - v as i32);

                        // Keep mouse over the same world position after zooming
                        if let Some(prev) = self.last_mouse_position {
                            let new_scaling_factor = self.get_zooming_factor();

                            let shift_x = (prev.x as f32) * (self.scaling_factor - new_scaling_factor);
                            let shift_y = (prev.y as f32) * (self.scaling_factor - new_scaling_factor);

                            self.ul_offset.0 += shift_x;
                            self.ul_offset.1 += shift_y;
                        }

                        self.calculate_projection();
                    }

                    CursorLeft { .. } => {
                        self.dragging = false;
                    }

                    _ => (),
                },
                Event::NewEvents(cause) => match cause {
                    glutin::event::StartCause::ResumeTimeReached { .. } => (),
                    glutin::event::StartCause::Init => (),
                    _ => return,
                },
                _ => return,
            }

            let ortho_matrix: &[[f32; 4]; 4] = self.ortho_matrix.as_ref();

            let uniforms = uniform! {
                matrix: *ortho_matrix,
                tex: self.spr_atlas.texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Linear)
            };

            let draw_params =
                glium::DrawParameters { blend: glium::Blend::alpha_blending(), ..Default::default() };

            // drawing a frame
            let mut target = self.display.draw();
            target.clear_color(0.5, 0.5, 0.5, 1.0);

            if self.vertex_buffer_len > 0 {
                target.draw(self.vertex_buffer.slice(0..self.vertex_buffer_len).unwrap(),
                      NoIndices(PrimitiveType::Points),
                      &self.program,
                      &uniforms,
                      &draw_params)
                .unwrap();
            }

            target.finish().unwrap();
        });
    }
}
