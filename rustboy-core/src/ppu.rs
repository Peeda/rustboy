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
const SCREEN_LEN: u32 = 256;
const SCREEN_SIZE: u32 = 256 * 256;
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
impl PPU {
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
    pub fn calculate_bg_tilemap(&self) -> [u32; SCREEN_SIZE as usize] {
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
        let mut out = [0u32; SCREEN_SIZE as usize];
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
                //write to the corresponding values in the buffer
                for y0 in 0..TILE_WIDTH {
                    for x0 in 0..TILE_WIDTH {
                        let x0 = TILE_WIDTH - x0;
                        let palette_ind = (merged[y0 as usize] & (0x03 << x0 * 2)) >> x0 * 2;
                        debug_assert!(palette_ind <= 3);
                        //read the palette's value at the 2 bit palette_ind
                        let palette = bus.read(PALETTE_ADDR);
                        let color_ind = (palette & (0x03 << palette_ind * 2)) >> palette_ind * 2;
                        let color = match color_ind {
                            0b00 => WHITE,
                            0b01 => LIGHT_GREY,
                            0b10 => DARK_GREY,
                            0b11 => BLACK,
                            _ => unreachable!()
                        };
                        set_pixel((y*TILE_WIDTH + y0).try_into().unwrap(), 
                            (x*TILE_WIDTH + x0).try_into().unwrap(), color, &mut out);
                    }
                }
            }
        }
        out
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
fn set_pixel(y: u8, x: u8, val: u32, buffer: &mut [u32; SCREEN_SIZE as usize]) {
    buffer[(y as u32 * SCREEN_LEN + x as u32) as usize] = val;
}
