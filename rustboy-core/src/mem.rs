pub trait Mem {
    fn read(&self, addr:u16) -> u8;
    fn write(&mut self, addr:u16, val:u8);
    fn borrow_mem(&mut self, addr:u16);
}
pub struct FlatMem {
    rom: [u8; 0x10000]
}
pub struct Bus {
    rom: [u8; 0x8000],
    vram: [u8; 0x2000],
    eram: [u8; 0x2000],
    wram: [u8; 0x2000],
}
impl Mem for Bus {
    fn read(&self, addr:u16) -> u8 {
        match addr {
            0..=0x7FFF => {
                self.rom[addr as usize]
            }
            0x8000..=0x9FFF => {
                self.vram[(addr & 0x1FFF) as usize]
            }
            0xA000..=0xBFFF => {
                self.eram[(addr & 0x1FFF) as usize]
            }
            //range is longer because of shadow wram
            0xC000..=0xFDFF => {
                self.wram[(addr & 0x1FFF) as usize]
            }
            0xFE00..=0xFE9F => {
                //oam
                todo!()
            }
            0xFEA0..=0xFEFF => {
                panic!("unusable memory area")
            }
            0xFF00..=0xFF7F => {
                //io
                todo!()
            }
            0xFF80..=0xFFFE => {
                //hram
                todo!()
            }
            0xFFFF => {
                //interrupt register
                todo!()
            }
        }
    }
    fn write(&mut self, addr:u16, val:u8) {
        todo!();
    }
    fn borrow_mem(&mut self, addr:u16) {
        todo!();
    }
}
