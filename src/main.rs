extern crate byteorder;
#[macro_use] extern crate glium;
extern crate image;

mod helpers;
mod spritecontainer;

use std::fs::File;
use std::path::Path;

use glium::{DisplayBuild, Surface};
use glium::glutin;
use glium::index::PrimitiveType;

use spritecontainer::SpriteContainer;

fn main() {
    println!("hi!");

    let display = glutin::WindowBuilder::new()
        .with_title(format!("Map Editor"))
        .with_vsync()
        .build_glium()
        .unwrap();

    let mut f = File::open("/home/kornholi/data/tibia/tibia1050/Tibia.spr").unwrap();
    let mut data = &mut f;

    let spr = SpriteContainer::new(data).unwrap();
    let sprite = spr.get_sprite(data, 200).unwrap();
    //sprite.save(Path::new("200.png")).unwrap();

    let texture = glium::texture::SrgbTexture2d::new(&display, sprite);

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(&display, 
            vec![
                Vertex { position: [-1.0, -1.0], tex_coords: [0.0, 0.0] },
                Vertex { position: [-1.0,  1.0], tex_coords: [0.0, 1.0] },
                Vertex { position: [ 1.0,  1.0], tex_coords: [1.0, 1.0] },
                Vertex { position: [ 1.0, -1.0], tex_coords: [1.0, 0.0] }
            ]
        )
    };

    // building the index buffer
    let index_buffer = glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip,
                                               vec![1 as u16, 2, 0, 3]);

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: include_str!("shaders/140.vert"),
            fragment: include_str!("shaders/140.frag")
        },
    ).unwrap();
    
    // the main loop
    start_loop(|| {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: texture.sampled().magnify_filter(glium::uniforms::MagnifySamplerFilter::Nearest)
        };

        let draw_params = glium::DrawParameters {
            blending_function:
                Some(glium::BlendingFunction::Addition { 
                                        source: glium::LinearBlendingFactor::SourceAlpha,
                                        destination: glium::LinearBlendingFactor::OneMinusSourceAlpha }),
            .. Default::default()
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.draw(&vertex_buffer, &index_buffer, &program, &uniforms, &draw_params).unwrap();
        target.finish();

        // polling and handling the events received by the window
        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return Action::Stop,
                _ => ()
            }
        }

        Action::Continue
    });
}

use std::thread;
extern crate clock_ticks;

pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(mut callback: F) where F: FnMut() -> Action {
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
