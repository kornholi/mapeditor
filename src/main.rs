extern crate byteorder;
extern crate cgmath;
extern crate clock_ticks;
#[macro_use] extern crate enum_primitive;
extern crate encoding;
#[macro_use] extern crate glium;
extern crate image;
extern crate num;

mod helpers;
mod rootwindow;
mod datcontainer;
mod spritecontainer;
mod opentibia;

use std::fs::File;
use std::path::Path;

use glium::glutin;
use glium::{DisplayBuild};

use datcontainer::DatContainer;
use spritecontainer::SpriteContainer;
use rootwindow::RootWindow;

use opentibia::binaryfile::Node;
use helpers::ReadExt;

fn main() {
    println!("hi!");

    let mut f = File::open(r"o:\#\Tibia\Tibia1072\Tibia.spr").unwrap();
    let mut data = &mut f;

    let spr = SpriteContainer::new(data).unwrap();
    let sprite = spr.get_sprite(data, 200).unwrap();

    let mut f = File::open(r"o:\#\Tibia\Tibia1072\Tibia.dat").unwrap();
    let mut data = &mut f;
   
    let spr = DatContainer::new(data).unwrap();
    
    let mut f = File::open(r"o:\#\Tibia\sample_data\RealMap.otbm").unwrap();
    let mut data = std::io::BufReader::new(&mut f);

    let version = data.read_u32().unwrap();
    println!("otbm ver {}", version);

    let start = clock_ticks::precise_time_ms();
    let node = Node::deserialize(&mut data, false);
    let end = clock_ticks::precise_time_ms();

    println!("otbm node load took {}ms", end - start);

    //println!("root {:?}", node);
    //let mut indent = 0;

    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf);

    return ();

    /*let display = glutin::WindowBuilder::new()
        .with_title(format!("Map Editor"))
        .with_dimensions(800, 500)
        .with_vsync()
        .build_glium()
        .unwrap();

    let texture = glium::texture::SrgbTexture2d::new(&display, sprite);

    let mut root = RootWindow::new(display, texture);
    
    root.resize(800, 500);
    root.run();

    println!("done");*/
}