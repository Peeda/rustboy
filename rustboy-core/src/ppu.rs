use bitflags::bitflags;
use std::cell::RefCell;
use std::rc::Rc;

use crate::mem::Mem;
const LINE_LEN: u32 = 456;
const FRAME_LEN: u32 = 70224;
const OAM_END: u32 = 79;
const DRAW_BEGIN: u32 = 80;
const DRAW_END: u32 = 251;
const HBLANK_BEGIN: u32 = 252;
const HBLANK_END: u32 = 455;
const SCY_ADDR: u16 = 0xFF42;
const SCX_ADDR: u16 = 0xFF43;
const LCDC_ADDR: u16 = 0xFF40;

const MAP_PIXEL_LEN: u32 = 256;
const MAP_PIXEL_SIZE: u32 = 256 * 256;
const MAP_WIDTH: u16 = 32;
const TILE_WIDTH:u16 = 8;
const TILE_BYTES:u16 = 16;
const BLOCK_ZERO: u16 = 0x8000;
const BLOCK_ONE: u16 = 0x8800;
const BLOCK_TWO: u16 = 0x9000;
const PALETTE_ADDR:u16 = 0xFF47;
const WHITE: u32 = 0xff000000;
const LIGHT_GREY: u32 = 0xff555555;
const DARK_GREY: u32 = 0xffaaaaaa;
const BLACK: u32 = 0xffffffff;
bitflags! {
    struct LCDC: u8 {
        const LCD_ENABLE = 1 << 7;
        const WIN_MAP_ADDR = 1 << 6;
        const WINDOW = 1 << 5;
        const TILE_ADDR_MODE = 1 << 4;
        const BG_MAP_ADDR = 1 << 3;
        const OBJ_SIZE = 1 << 2;
        const OBJ_ENABLE = 1 << 1;
        const PRIORITY = 1 << 0;
    }
}
pub struct PPU {
    dots: u32,
    bus: Rc<RefCell<dyn Mem>>,
    mode: Mode,
}
#[derive(PartialEq, Eq)]
enum Mode {
    Search,
    Draw,
    HBlank,
    VBlank,
}
struct Buffer {
    pub length: u32,
    pub width: u32,
    pub data: Vec<u32>,
}
impl Buffer {
    pub fn init(length:u32, width:u32) -> Buffer {
        let mut temp = Vec::new();
        temp.resize(length as usize * width as usize, 0);
        Buffer {
            length,
            width,
            data: temp,
        }
    }
    fn set_pixel(&mut self, y: u8, x: u8, val: u32) {
        self.data[(y as u32 * self.length + x as u32) as usize] = val;
    }
    fn write_tile(&mut self, y: u8, x: u8, palette:u8, tile: &[u16; 8]) {
        //write to the corresponding values in the buffer
        for y0 in 0..TILE_WIDTH {
            for x0 in 0..TILE_WIDTH {
                //bit number goes right to left not left to right
                let x1 = TILE_WIDTH - x0 - 1;
                let palette_ind = (tile[y0 as usize] & (0x03 << x1 * 2)) >> x1 * 2;
                debug_assert!(palette_ind <= 3);
                //read the palette's value at the 2 bit palette_ind
                let color_ind = (palette & (0x03 << palette_ind * 2)) >> palette_ind * 2;
                let color = match color_ind {
                    0b00 => WHITE,
                    0b01 => LIGHT_GREY,
                    0b10 => DARK_GREY,
                    0b11 => BLACK,
                    _ => unreachable!()
                };
                let y_loc = y*(TILE_WIDTH as u8) + y0 as u8;
                let x_loc = x*(TILE_WIDTH as u8) + x0 as u8;
                self.set_pixel(y_loc, x_loc, color);
            }
        }
        println!("{y}, {x}");
    }
}
impl PPU {
    pub fn init(bus: Rc<RefCell<dyn Mem>>) -> PPU {
        PPU {
            dots: 0,
            bus,
            mode: Mode::Search,
        }
    }
    //advance the PPU by n CPU clocks, n*4 dots/t cycles
    pub fn tick(&mut self, clocks: u8) {
        self.dots += clocks as u32;
        self.dots %= FRAME_LEN;
        let line = self.dots / LINE_LEN;
        debug_assert!(self.dots < FRAME_LEN);
        debug_assert!(line <= 153);

        match line {
            0..=143 => {
                //not v blank
                if self.mode == Mode::VBlank {
                    self.mode = Mode::Search;
                }
            }
            144..=153 => {
                //v blank
                debug_assert!(self.mode != Mode::Search);
                debug_assert!(self.mode != Mode::Draw);
                if self.mode == Mode::HBlank {
                    //TODO: transition into vblank
                    self.mode = Mode::VBlank;
                }
            }
            _ => unreachable!(),
        }

        //TODO: disable correct memory regions
        //note to self this code still runs during vblank
        match self.dots % LINE_LEN {
            0..=OAM_END => {
                debug_assert!(self.mode != Mode::Draw);
                if self.mode == Mode::HBlank {
                    self.mode = Mode::Search;
                }
            }
            DRAW_BEGIN..=DRAW_END => {
                debug_assert!(self.mode != Mode::HBlank);
                if self.mode == Mode::Search {
                    self.mode = Mode::Draw;
                    //do search
                }
            }
            HBLANK_BEGIN..=HBLANK_END => {
                debug_assert!(self.mode != Mode::Search);
                if self.mode == Mode::Draw {
                    self.mode = Mode::HBlank;
                    //draw the line
                    //let tl_x = bus.read(SCX_ADDR);
                    //let tl_y = bus.read(SCY_ADDR);
                }
            }
            _ => unreachable!(),
        }
    }
    pub fn debug_tiles(&self) -> [u32; 8 * 8 * 384] {
        todo!();
        //let mut out = [0; 8 * 8 * 384];
        //let bus = self.bus.borrow();
        ////$8000 - $97FF
        //let mut i:u16 = 0x8000;
        //while i <= 0x97FF {
        //    let mut merged:[u16; 8] = [0; 8];
        //    for k in 0..8 {
        //        merged[k as usize] = spread(bus.read(i + 2*k)) | ((spread(bus.read(i + 2*k + 1)) << 1));
        //    }
        //    let iterations:u16 = (i - 0x8000) / 16;
        //    write_tile((iterations/16) as u8, (iterations % 16) as u8, 0b11100100, &merged, &mut out[..]);
        //    i += 16;
        //}
        //out
    }
    pub fn calculate_bg_tilemap(&self) -> [u32; MAP_PIXEL_SIZE as usize] {
        let mut buffer = Buffer::init(MAP_PIXEL_LEN, MAP_PIXEL_LEN);
        let bus = self.bus.borrow();
        //I'm just gonna calculate the whole tile map for now
        let lcdc = LCDC::from_bits(bus.read(LCDC_ADDR)).unwrap();
        let bg_addr = if lcdc.contains(LCDC::BG_MAP_ADDR) {
            0x9C00
        } else {
            0x9800
        };
        for y in 0..MAP_WIDTH {
            for x in 0..MAP_WIDTH {
                let tile_ind = bus.read(bg_addr + y * MAP_WIDTH + x);
                //account for addressing mode, get start of 16 byte tile
                let tile_loc = match tile_ind {
                    0..=127 => {
                        if lcdc.contains(LCDC::TILE_ADDR_MODE) {
                            BLOCK_ZERO + tile_ind as u16 * TILE_BYTES
                        } else {
                            BLOCK_TWO + tile_ind as u16 * TILE_BYTES
                        }
                    }
                    128..=255 => BLOCK_ONE + (tile_ind - 128) as u16 * TILE_BYTES,
                };
                //reformat the tile data into one u16 per row, with each two
                //bits encoding a pixel
                let mut merged: [u16; 8] = [0; 8];
                for i in 0..8 {
                    let byte_one = bus.read(tile_loc + 2 * i as u16);
                    let byte_two = bus.read(tile_loc + 2 * i as u16 + 1);
                    merged[i] = spread(byte_one) | (spread(byte_two) << 1);
                }
                let palette = bus.read(PALETTE_ADDR);
                buffer.write_tile(y.try_into().unwrap(), x.try_into().unwrap(), palette, &merged);
            }
        }
        buffer.data.try_into().expect("wrong size.")
    }
}
fn spread(val: u8) -> u16 {
    let mut out: u16 = 0;
    for i in 0..8 {
        if val & (1 << i) > 0 {
            out |= 1u16 << (2 * i);
        }
    }
    out
}
