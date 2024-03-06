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
    //let ram = std::fs::read("pokemon.dmp").unwrap();
    for b in ram.iter().enumerate() {
        mem.write(b.0.try_into().unwrap(), *b.1);
    }
    mem.write(0xff47, 0b00011011);
    let bus = Rc::new(RefCell::new(mem));
    let mut ppu = PPU::init(bus);

    let mut gb_screen = {
        let img = Image::gen_image_color(160, 144, Color::PURPLE);
        let tex = rl.load_texture_from_image(&thread, &img).unwrap();
        tex
    };

    let mut tile_data = {
        let img = Image::gen_image_color(24*8, 16*8, Color::PURPLE);
        let tex = rl.load_texture_from_image(&thread, &img).unwrap();
        tex
    };
    let mut bg_map = {
        let img = Image::gen_image_color(256, 256, Color::PURPLE);
        let tex = rl.load_texture_from_image(&thread, &img).unwrap();
        tex
    };
    let mut window = {
        let img = Image::gen_image_color(256, 256, Color::PURPLE);
        let tex = rl.load_texture_from_image(&thread, &img).unwrap();
        tex
    };
    while !rl.window_should_close() {
        for _ in 0..100 {
            ppu.tick(4);
        }
        unsafe {
            gb_screen.update_texture(&std::mem::transmute::<[u32;160 * 144], [u8;160 * 144 * 4]>(ppu.screen()));
            tile_data.update_texture(&std::mem::transmute::<[u32;24576], [u8;98304]>(ppu.debug_tiles()));
            bg_map.update_texture(&std::mem::transmute::<[u32;65536], [u8;262144]>(ppu.calculate_tilemap(true)));
            window.update_texture(&std::mem::transmute::<[u32;65536], [u8;262144]>(ppu.calculate_tilemap(false)));
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::WHITE);
        d.draw_fps(10, 10);

        d.draw_text("EEEE", 10, 500, 12, Color::BLACK);
        const TILE_TL:(i32, i32) = (800, 12);
        d.draw_rectangle(TILE_TL.0 - 5, TILE_TL.1 - 5, tile_data.width * 2 + 10, tile_data.height * 2 + 10, Color::RED);
        d.draw_texture_ex(&tile_data, math::Vector2::new(800., 12.), 0., 2., Color::WHITE);

        d.draw_rectangle(96 - 5, 60 - 5, gb_screen.width * 2 + 10, gb_screen.height * 2 + 10, Color::RED);
        d.draw_texture_ex(&gb_screen, math::Vector2::new(96., 60.), 0., 2., Color::WHITE);

        let bg_tl = (725, 28 + tile_data.height * 2);
        let window_tl = (bg_tl.0 + 20 + bg_map.width, bg_tl.1);
        tex_with_outline(&mut d, &bg_map, bg_tl);
        tex_with_outline(&mut d, &window, window_tl);
    }
}
fn tex_with_outline(d: &mut RaylibDrawHandle, tex: &Texture2D, tl: (i32, i32)) {
    d.draw_rectangle(tl.0-5, tl.1-5, tex.width + 10, tex.height + 10, Color::RED);
    d.draw_texture(tex, tl.0, tl.1, Color::WHITE);
}
