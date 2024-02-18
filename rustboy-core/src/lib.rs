pub mod cpu;
pub mod mem;
pub mod ppu;
pub mod tables;

struct GameBoy {
    cpu: cpu::CPU,
    ppu: ppu::PPU,
}
impl GameBoy {
    pub fn step(&mut self) {
        let clocks = self.cpu.tick();
        self.ppu.tick(clocks);
    }
}
