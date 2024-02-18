use raylib::prelude::*;

fn spread(val:u8) -> u16 {
    let mut out:u16 = 0;
    for i in 0..8 {
        if val & (1 << i) > 0 {
            out |= 1u16 << (2*i);
        }
    }
    out
}
fn main() {
    let (mut rl, thread) = raylib::init()
        .size(640, 480)
        .title("Hello, World")
        .build();
     
    let img = Image::gen_image_color(8, 8, Color::PURPLE);
    let mut buffer:[u32; 64] = [0; 64];
    //let paddle:[u8; 16] = [0xFF, 0x7E, 0x81, 0x81, 0xFF, 0x7E, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let paddle:[u8; 16] = [0x3c, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x7e, 0x5e, 0x7e, 0x0a, 0x7c, 0x56, 0x38, 0x7c];
    let mut merged:[u16; 8] = [0; 8];
    for i in 0..8 {
        // 00111100
        // 01111110
        // -------- spread
        // 0000010101010000
        // 0010101010101000
        // -------- merge
        // 0010111111111000
        merged[i] = spread(paddle[2*i]) | ((spread(paddle[2*i + 1]) << 1));
    }
    for i in &mut buffer {
        *i = 0x00000000;
    }
    for i in 0..buffer.len() {
        let row = i/8;
        let col = i%8;
        buffer[i] = match (merged[row] & (0x03 << col * 2)) >> col * 2 {
            0b00 => 0xff000000,
            0b01 => 0xffff0000,
            0b10 => 0xff00ff00,
            0b11 => 0xff0000ff,
            _ => unreachable!()
        }
    }
    let mut tex = rl.load_texture_from_image(&thread, &img).unwrap();
    unsafe {
        tex.update_texture(&std::mem::transmute::<[u32;64], [u8;256]>(buffer));
    }

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
         
        d.clear_background(Color::WHITE);
        d.draw_texture(&tex, 12, 12, Color::WHITE);
    }
}
