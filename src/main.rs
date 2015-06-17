#![feature(collections)]

extern crate byteorder;
extern crate cgmath;
extern crate clock_ticks;
#[macro_use] extern crate enum_primitive;
extern crate encoding;
#[macro_use] extern crate glium;
extern crate image;
extern crate num;
extern crate toml;

mod helpers;
mod rootwindow;
mod datcontainer;
mod spriteatlas;
mod spritecontainer;
mod opentibia;
mod map;

use std::io::Read;
use std::fs::File;

use glium::glutin;
use glium::{DisplayBuild};

use datcontainer::DatContainer;
use spritecontainer::SpriteContainer;
use rootwindow::{Renderer, RootWindow};

use spriteatlas::SpriteAtlas;

use opentibia::itemtypes;
use helpers::ReadExt;

fn main() {
    let mut input = String::new();

    File::open("conf.toml").and_then(|mut f| {
        f.read_to_string(&mut input)
    }).unwrap();

    let mut parser = toml::Parser::new(&input);

    let conf = match parser.parse() {
        Some(value) => value,
        None => {
            println!("conf parsing failed:");

            for err in &parser.errors {
                    let (loline, locol) = parser.to_linecol(err.lo);
                    let (hiline, hicol) = parser.to_linecol(err.hi);
                    println!("{}:{}-{}:{} error: {}",
                             loline, locol, hiline, hicol, err.desc);
            }

            return
        }
    };

    // spr
    let mut spr_data = std::io::BufReader::new(File::open(conf["spr"].as_str().unwrap()).unwrap());
    let spr = SpriteContainer::new(&mut spr_data).unwrap();

    // dat
    let mut data = std::io::BufReader::new(File::open(conf["dat"].as_str().unwrap()).unwrap());
    let dat = DatContainer::new(&mut data).unwrap();
    
    // otb
    let mut data = std::io::BufReader::new(File::open(conf["otb"].as_str().unwrap()).unwrap());
    let _version = data.read_u32().unwrap();
    let otb = itemtypes::Container::new(&mut data).unwrap();

    // otbm
    let mut data = std::io::BufReader::new(File::open(conf["map"].as_str().unwrap()).unwrap());
    let _version = data.read_u32().unwrap();

    //let node = Node::deserialize(&mut data, false).unwrap();
    //let node = opentibia::binaryfile::streaming_parser(&mut data, false,
    //    |kind, data| {
    //        println!("node {} with {}b", kind, data.len());
    //    });

    let start = clock_ticks::precise_time_ms();
    let mut otbm_map = opentibia::map::Loader::open(&mut data).unwrap();
    let mut map = map::Map::new();

    let mut tiles = 0;

    otbm_map.load(&mut data, |pos, items| {
        tiles += 1;

        let sec = map.get_or_create(pos);
        sec.get_tile(pos).push_all(items);
    }).unwrap();

    let end = clock_ticks::precise_time_ms();

    println!("otbm node load took {}ms", end - start);
    println!("total {} tiles", tiles);

    //let mut buf = String::new();
    //std::io::stdin().read_line(&mut buf);

    let display = glutin::WindowBuilder::new()
        .with_title(format!("Map Editor"))
        .with_dimensions(1100, 1100)
        //.with_vsync()
        .build_glium()
        .unwrap();

    let rend = Renderer {
        spr: spr,
        spr_data: spr_data,
        dat: dat,
        otb: otb,

        atlas: SpriteAtlas::new(&display),
        map: map,

        vertices: Vec::new()
        //..Default::default()
    };

    let mut root = RootWindow::new(display, rend);
    
    root.resize(1100, 1100);
    root.run();
}