use raylib::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::vec::Vec;
use rustboy_core::mem::{FlatMem, Mem};
use rustboy_core::ppu::PPU;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1280, 760)
        .title("Hello, World")
        .build();

    let mut mem = FlatMem::default();
    let bytes1 = std::fs::read("vram.dmp").unwrap();
    let bytes2 = std::fs::read("io_highram.dmp").unwrap();
    let bytes3 = std::fs::read("oam.dmp").unwrap();
    for b in bytes1.iter().enumerate() {
        mem.write((0x8000 + b.0) as u16, *b.1);
    }
    for b in bytes2.iter().enumerate() {
        mem.write((0xFF00 + b.0) as u16, *b.1);
    }
    for b in bytes3.iter().enumerate() {
        mem.write((0xFE00 + b.0) as u16, *b.1);
    }
    mem.write(0xff47, 0b11100100);
    let bus = Rc::new(RefCell::new(mem));
    let ppu = PPU::init(bus);
    let buffer = ppu.calculate_bg_tilemap();

    let tex1 = {
        let img = Image::gen_image_color(256, 256, Color::PURPLE);
        let mut tex = rl.load_texture_from_image(&thread, &img).unwrap();
        unsafe {
            tex.update_texture(&std::mem::transmute::<[u32;65536], [u8;262144]>(buffer));
        }
        tex
    };
    //let debug_tiles = {
    //    let img = Image::gen_image_color(256, 256, Color::PURPLE);
    //    let mut tex = rl.load_texture_from_image(&thread, &img).unwrap();
    //    let tile_buffer = ppu.debug_tiles();
    //    unsafe {
    //        tex.update_texture(&std::mem::transmute::<[u32;24576], [u8;98304]>(tile_buffer));
    //    }
    //    tex
    //};
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        d.draw_texture(&tex1, 0, 0, Color::WHITE);
        //d.draw_texture(&debug_tiles, 0, 0, Color::WHITE);
    }
}
