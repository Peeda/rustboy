use raylib::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use rustboy_core::mem::{FlatMem, Mem};
use rustboy_core::ppu::PPU;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(1280, 760)
        .title("Hello, World")
        .build();

    let mut mem = FlatMem::default();
    let ram = std::fs::read("ram.dmp").unwrap();
    for b in ram.iter().enumerate() {
        mem.write(b.0.try_into().unwrap(), *b.1);
    }
    mem.write(0xff47, 0b00011011);
    let bus = Rc::new(RefCell::new(mem));
    let ppu = PPU::init(bus);
    let buffer = ppu.calculate_tilemap(true);

    let tex1 = {
        let img = Image::gen_image_color(256, 256, Color::PURPLE);
        let mut tex = rl.load_texture_from_image(&thread, &img).unwrap();
        unsafe {
            tex.update_texture(&std::mem::transmute::<[u32;65536], [u8;262144]>(buffer));
        }
        tex
    };
    let debug_tiles = {
        let img = Image::gen_image_color(24*8, 16*8, Color::PURPLE);
        let mut tex = rl.load_texture_from_image(&thread, &img).unwrap();
        let tile_buffer = ppu.debug_tiles();
        unsafe {
            tex.update_texture(&std::mem::transmute::<[u32;24576], [u8;98304]>(tile_buffer));
        }
        tex
    };
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);

        d.draw_texture(&tex1, 300, 300, Color::WHITE);
        d.draw_texture_ex(&debug_tiles, math::Vector2::new(12., 12.), 0., 2., Color::WHITE);
    }
}
