use bitflags::{bitflags, Flags};
bitflags! {
    struct GbFlags: u8 {
        const Z = 1 << 7;
        const N = 1 << 6;
        const H = 1 << 5;
        const C = 1 << 4;
        const _ = !0;
    }
}
//clock cycles taken for instructions, uses lower value for variable length opcodes
pub const CLOCK: [u8; 256] = [
     4, 12,  8,  8,  4,  4,  8,  4, 20,  8,  8,  8,  4,  4,  8,  4,
     4, 12,  8,  8,  4,  4,  8,  4, 12,  8,  8,  8,  4,  4,  8,  4,
     8, 12,  8,  8,  4,  4,  8,  4,  8,  8,  8,  8,  4,  4,  8,  4,
     8, 12,  8,  8, 12, 12, 12,  4,  8,  8,  8,  8,  4,  4,  8,  4,
     4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
     4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
     4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
     8,  8,  8,  8,  8,  8,  4,  8,  4,  4,  4,  4,  4,  4,  8,  4,
     4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
     4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
     4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
     4,  4,  4,  4,  4,  4,  8,  4,  4,  4,  4,  4,  4,  4,  8,  4,
     8, 12, 12, 16, 12, 16,  8, 16,  8, 16, 12,  4, 12, 24,  8, 16,
     8, 12, 12,  4, 12, 16,  8, 16,  8, 16, 12,  4, 12,  4,  8, 16,
    12, 12,  8,  4,  4, 16,  8, 16, 16,  4, 16,  4,  4,  4,  8, 16,
    12, 12,  8,  4,  4, 16,  8, 16, 12,  8, 16,  4,  4,  4,  8, 16,
];
pub const CB_CLOCK: [u8; 256] = [
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 12, 8, 8, 8, 8, 8, 8, 8, 12, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
    8, 8, 8, 8, 8, 8, 16, 8, 8, 8, 8, 8, 8, 8, 16, 8,
];
#[allow(non_snake_case)]
pub struct CPU {
    regs:Registers,
    SP: u16,
    PC: u16,
}
impl CPU {
    fn read_from_ind(&self, ind:u8) -> u8 {
        match ind {
            0 => self.regs.B,
            1 => self.regs.C,
            2 => self.regs.D,
            3 => self.regs.E,
            4 => self.regs.H,
            5 => self.regs.L,
            6 => self.read_mem(self.regs.read_hl()),
            7 => self.regs.A,
            _ => panic!("Invalid index")
        }
    }
    fn write_from_ind(&mut self, ind:u8, data:u8) {
        match ind {
            0 => self.regs.B = data,
            1 => self.regs.C = data,
            2 => self.regs.D = data,
            3 => self.regs.E = data,
            4 => self.regs.H = data,
            5 => self.regs.L = data,
            6 => self.write_mem(self.regs.read_hl(), data),
            7 => self.regs.A = data,
            _ => panic!("Invalid index"),
        }
    }
    fn check_cond(&self, ind:u8) -> bool {
        match ind {
            0 => !self.regs.F.contains(GbFlags::Z),
            1 => self.regs.F.contains(GbFlags::Z),
            2 => !self.regs.F.contains(GbFlags::C),
            3 => self.regs.F.contains(GbFlags::C),
            _ => unreachable!()
        }
    }
    pub fn execute(&mut self, opcode: u8) -> u8 {
        self.PC += 1;
        //bits 6 and 7
        let x = (opcode & 0b11000000) >> 6;
        //bits 3,4,5
        let y = (opcode & 0b00111000) >> 3;
        //bits 0,1,2
        let z = opcode & 0b00000111;
        //bits 4,5
        let p = y >> 1;
        //bit 3
        let q = y % 2;
        let mut extra_cycles = 0;
        match x {
            0 => {
                match z {
                    0 => {
                        match y {
                            0 => {},
                            2 => {
                                //STOP
                                todo!()
                            }
                            3 => {
                                //TODO: make an abstraction over jumping probably
                                let shift = self.read_mem(self.PC) as i8;
                                self.PC += 1;
                                if shift >= 0 {
                                    self.PC.wrapping_add(shift as u16);
                                } else {
                                    self.PC.wrapping_sub((shift * -1) as u16);
                                }
                            }
                            4..=7 => {
                                let shift = self.read_mem(self.PC) as i8;
                                self.PC += 1;
                                if self.check_cond(y - 4) {
                                    if shift >= 0 {
                                        self.PC.wrapping_add(shift as u16);
                                    } else {
                                        self.PC.wrapping_sub((shift * -1) as u16);
                                    }
                                }
                            }
                        }
                    }
                    7 => {
                        match y {
                            0 => {
                                if self.regs.A & 1 << 7 > 0 {
                                    self.regs.F |= GbFlags::C;
                                }
                                self.regs.A = self.regs.A.rotate_left(1);
                                self.regs.F -= GbFlags::Z | GbFlags::N | GbFlags::H;
                            }
                            1 => {
                                if self.regs.A & 1 > 0 {
                                    self.regs.F |= GbFlags::C;
                                }
                                self.regs.A = self.regs.A.rotate_right(1);
                                self.regs.F -= GbFlags::Z | GbFlags::N | GbFlags::H;
                            }
                            2 => {
                                let carry = self.regs.F.contains(GbFlags::C);
                                if self.regs.A & 1 << 7 > 0 {
                                    self.regs.F |= GbFlags::C;
                                }
                                self.regs.A << 1;
                                if carry {
                                    self.regs.A |= 1;
                                } else {
                                    self.regs.A &= !1;
                                }
                                self.regs.F -= GbFlags::Z | GbFlags::N | GbFlags::H;
                            }
                            3 => {
                                let carry = self.regs.F.contains(GbFlags::C);
                                if self.regs.A & 1 > 0 {
                                    self.regs.F |= GbFlags::C;
                                }
                                self.regs.A >> 1;
                                if carry {
                                    self.regs.A |= 1 << 7;
                                } else {
                                    self.regs.A &= !(1 << 7);
                                }
                                self.regs.F -= GbFlags::Z | GbFlags::N | GbFlags::H;
                            }
                            4 => {
                                //daa
                                todo!()
                            }
                            5 => {
                                self.regs.A = !self.regs.A;
                                self.regs.F |= GbFlags::N | GbFlags::H;
                            }
                            6 => {
                                self.regs.F |= GbFlags::C;
                            }
                            7 => {
                                if self.regs.F.contains(GbFlags::C) {
                                    self.regs.F -= GbFlags::C;
                                } else {
                                    self.regs.F |= GbFlags::C;
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                }
            }
            1 => {
                //8 bit LD from register
                if !(y == 7 && z == 7) {
                    self.write_from_ind(y, self.read_from_ind(z));
                }
            }
            2 => {
                self.arithmetic_eight(y, self.read_from_ind(z));
            }
            3 => {
                match z {
                    //immediate 8 bit arithmetic
                    0b110 => {
                        self.arithmetic_eight(y, self.read_mem(self.PC));
                        self.PC += 1;
                    }
                    _ => todo!()
                }
            }
            _ => unreachable!()
        }
        CLOCK[opcode as usize] + extra_cycles
    }
    fn arithmetic_eight(&mut self, id:u8, val:u8) {
        match id {
            0 => {
                self.regs.F -= GbFlags::N;
                if self.regs.A as u16 + val as u16 > u8::MAX.into() {
                    self.regs.F |= GbFlags::C;
                }
                if (self.regs.A & 0x0F) + (val & 0x0F) > 0x0F {
                    self.regs.F |= GbFlags::H;
                }
                self.regs.A = self.regs.A.wrapping_add(val);
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
            }
            1 => {
                let carry = if self.regs.F.intersects(GbFlags::C) {1} else {0};
                self.regs.F -= GbFlags::N;
                if self.regs.A as u16 + val as u16 + carry > u8::MAX.into() {
                    self.regs.F |= GbFlags::C;
                }
                if (self.regs.A & 0x0F) + (val & 0x0F) + carry as u8 > 0x0F {
                    self.regs.F |= GbFlags::H;
                }
                self.regs.A = self.regs.A.wrapping_add(val).wrapping_add(carry as u8);
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
            }
            2 => {
                self.regs.F |= GbFlags::N;
                if self.regs.A < val {
                    self.regs.F |= GbFlags::C;
                }
                if (self.regs.A & 0x0F) < (val & 0x0F) {
                    self.regs.F |= GbFlags::H;
                }
                self.regs.A = self.regs.A.wrapping_sub(val);
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
            }
            3 => {
                let carry = if self.regs.F.intersects(GbFlags::C) {1} else {0};
                self.regs.F |= GbFlags::N;
                if (self.regs.A as u16) < val as u16 + carry {
                    self.regs.F |= GbFlags::C;
                }
                if (self.regs.A & 0x0F) < (val & 0x0F + carry as u8) {
                    self.regs.F |= GbFlags::H;
                }
                self.regs.A = self.regs.A.wrapping_sub(val).wrapping_sub(carry as u8);
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
            }
            4 => {
                self.regs.A &= val;
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
                self.regs.F -= GbFlags::N | GbFlags:: C;
                self.regs.F |= GbFlags::H;
            }
            5 => {
                self.regs.A ^= val;
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
                self.regs.F -= GbFlags::N | GbFlags::H | GbFlags::C;
            }
            6 => {
                self.regs.A |= val;
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
                self.regs.F -= GbFlags::N | GbFlags::H | GbFlags::C;
            }
            7 => {
                self.regs.F |= GbFlags::N;
                if self.regs.A < val {
                    self.regs.F |= GbFlags::C;
                }
                if (self.regs.A & 0x0F) < (val & 0x0F) {
                    self.regs.F |= GbFlags::H;
                }
                if self.regs.A == 0 { self.regs.F |= GbFlags::Z }
            }
            _ => unreachable!()
        }
    }
    fn read_mem(&self, addr:u16) -> u8 {
        todo!()
    }
    fn write_mem(&self, addr:u16, data:u8) {
        todo!()
    }
}
impl Default for CPU {
    fn default() -> Self {
        CPU {
            regs:Registers::default(),
            SP: 0xFFFE,
            PC: 0x0100,
        }
    }
}

#[allow(non_snake_case)]
struct Registers {
    A: u8,
    F: GbFlags,
    B: u8,
    C: u8,
    D: u8,
    E: u8,
    H: u8,
    L: u8,
}
macro_rules! read_16 {
    ($name:ident, $high:ident, $low:ident) => {
        fn $name(&self) -> u16 {
            (self.$high as u16) << 8 | self.$low as u16
        }
    }
}
macro_rules! write_16 {
    ($name:ident, $high:ident, $low:ident) => {
        fn $name(&mut self, val: u16) {
            self.$high = (val >> 8) as u8;
            self.$low = (val & 0xFF) as u8;
        }
    }
}
impl Registers {
    fn read_af(&self) -> u16 {
        (self.A as u16) << 8 | self.F.bits() as u16
    }
    fn write_af(&mut self, val: u16) {
        self.A = (val >> 8) as u8;
        self.F = GbFlags::from_bits_retain((val & 0xFF) as u8);
    }
    read_16!(read_bc, B, C);
    read_16!(read_de, D, E);
    read_16!(read_hl, H, L);
    write_16!(write_bc, B, C);
    write_16!(write_de, D, E);
    write_16!(write_hl, H, L);
}
impl Default for Registers {
    fn default() -> Self {
        Registers {
            A: 0x01,
            F: GbFlags::from_bits_retain(0xB0),
            B: 0x00,
            C: 0x13,
            D: 0x00,
            E: 0xD8,
            H: 0x01,
            L: 0x4D,
        }
    }
}
