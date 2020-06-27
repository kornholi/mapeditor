#[macro_use]
extern crate glium;

use std::time::Instant;

mod datcontainer;
mod helpers;
mod map;
mod opentibia;
mod renderer;
mod rootwindow;
mod spriteatlas;
mod spritecontainer;

use std::fs::File;
use std::io::Read;

use serde::Deserialize;

use glium::glutin;

use datcontainer::DatContainer;
use renderer::Renderer;
use rootwindow::RootWindow;
use spritecontainer::SpriteContainer;

use helpers::ReadExt;
use opentibia::itemtypes;

#[derive(Deserialize)]
struct Config {
    spr: String,
    dat: String,
    otb: String,
    map: String,
}

fn main() {
    let mut raw_config = String::new();
    File::open("conf.toml")
        .and_then(|mut f| f.read_to_string(&mut raw_config))
        .unwrap();

    let config: Config = match toml::from_str(&raw_config) {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to load config: {}", e);
            return;
        }
    };

    // spr
    let spr_data = std::io::BufReader::new(File::open(config.spr).unwrap());
    let spr = SpriteContainer::new(spr_data).unwrap();

    // dat
    let mut data = std::io::BufReader::new(File::open(config.dat).unwrap());
    let dat = DatContainer::new(&mut data).unwrap();

    // otb
    let mut data = std::io::BufReader::new(File::open(config.otb).unwrap());
    let _version = data.read_u32().unwrap();
    let otb = itemtypes::Container::new(&mut data).unwrap();

    // otbm
    let mut data = std::io::BufReader::new(File::open(config.map).unwrap());
    let _version = data.read_u32().unwrap();

    // let node = Node::deserialize(&mut data, false).unwrap();
    // let node = opentibia::binaryfile::streaming_parser(&mut data, false,
    //    |kind, data| {
    //        println!("node {} with {}b", kind, data.len());
    //    });

    let start = Instant::now();
    let mut map = map::Map::new();

    let mut otbm_map = opentibia::map::Loader::open(&mut data).unwrap();
    let mut tiles = 0;

    otbm_map
        .load(&mut data, |ref pos, items| {
            tiles += 1;

            let sec = map.get_or_create(pos);
            sec.get_tile(pos).extend_from_slice(items);
        })
        .expect("failed to load OTBM");

    let dur = start.elapsed().as_secs_f64();

    println!(
        "OTBM node load took {:.2}ms for {} tiles",
        dur * 1000.,
        tiles
    );

    let event_loop = glutin::event_loop::EventLoop::new();
    let window = glutin::window::WindowBuilder::new()
        .with_title("Map Editor")
        .with_inner_size(glutin::dpi::PhysicalSize::new(1100, 1100));
    //.with_vsync()
    //.build()
    //.unwrap();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context, &event_loop).unwrap();

    let rend = Renderer::<rootwindow::Vertex>::new(dat, otb, map);
    let mut root = RootWindow::new(display, rend, spr);

    root.resize(1100, 1100);
    root.run(event_loop);
}
