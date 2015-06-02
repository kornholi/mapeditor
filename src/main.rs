extern crate byteorder;
extern crate cgmath;
extern crate clock_ticks;
#[macro_use] extern crate glium;
extern crate image;

mod helpers;
mod rootwindow;
mod spritecontainer;

use std::fs::File;
use std::path::Path;

use glium::glutin;
use glium::{DisplayBuild};

use spritecontainer::SpriteContainer;
use rootwindow::RootWindow;

fn main() {
    println!("hi!");

    let display = glutin::WindowBuilder::new()
        .with_title(format!("Map Editor"))
        .with_dimensions(800, 500)
        .with_vsync()
        .build_glium()
        .unwrap();

    let mut f = File::open(r"o:\#\Tibia\Tibia1072\Tibia.spr").unwrap();
    let mut data = &mut f;

    let spr = SpriteContainer::new(data).unwrap();
    let sprite = spr.get_sprite(data, 200).unwrap();
    sprite.save(Path::new("200.png")).unwrap();

    let texture = glium::texture::SrgbTexture2d::new(&display, sprite);

    let mut root = RootWindow::new(display, texture);
    
    root.resize(800, 500);
    root.run();

    println!("done");   
}