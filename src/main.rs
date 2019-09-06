extern crate byteorder;
extern crate cgmath;
extern crate clock_ticks;
#[macro_use]
extern crate enum_primitive;
extern crate encoding;
#[macro_use]
extern crate glium;
extern crate image;
extern crate num;
extern crate toml;
extern crate vec_map;
extern crate lru_cache;

#[macro_use]
extern crate serde_derive;

mod datcontainer;
mod helpers;
mod map;
mod opentibia;
mod renderer;
mod rootwindow;
mod spriteatlas;
mod spritecontainer;

use std::io::Read;
use std::fs::File;

use glium::glutin;

use datcontainer::DatContainer;
use spritecontainer::SpriteContainer;
use renderer::Renderer;
use rootwindow::RootWindow;

use opentibia::itemtypes;
use helpers::ReadExt;

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

    let start = clock_ticks::precise_time_ms();
    let mut map = map::Map::new();

    let mut otbm_map = opentibia::map::Loader::open(&mut data).unwrap();
    let mut tiles = 0;

    otbm_map.load(&mut data, |ref pos, items| {
            tiles += 1;

            let sec = map.get_or_create(pos);
            sec.get_tile(pos).extend_from_slice(items);
        })
        .expect("failed to load OTBM");

    let end = clock_ticks::precise_time_ms();

    println!("OTBM node load took {}ms for {} tiles", end - start, tiles);

    let mut event_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("Map Editor")
        .with_dimensions((1100, 1100).into());
        //.with_vsync()
        //.build()
        //.unwrap();
    let context = glutin::ContextBuilder::new();
    let display = glium::Display::new(window, context,  &event_loop).unwrap();

    let rend = Renderer::<rootwindow::Vertex>::new(dat, otb, map);
    let mut root = RootWindow::new(display, rend, spr);

    root.resize(1100, 1100);
    root.run(&mut event_loop);
}
