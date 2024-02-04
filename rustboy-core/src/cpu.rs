use crate::tables::{FlagEffect, CLOCK, ALT_CLOCK, ZERO_FLAG, SUB_FLAG, HALF_FLAG, CARRY_FLAG};
use bitflags::bitflags;
const Z:usize = 0;
const N:usize = 1;
const H:usize = 2;
const C:usize = 3;
bitflags! {
    struct GbFlags: u8 {
        const Z = 1 << 7;
        const N = 1 << 6;
        const H = 1 << 5;
        const C = 1 << 4;
        const _ = !0;
    }
}
#[allow(non_snake_case)]
pub struct CPU {
    regs:Registers,
    SP: u16,
    PC: u16,
}
impl CPU {
    fn fetch_byte(&mut self) -> u8 {
        let val = self.read_mem(self.PC);
        self.PC += 1;
        val
    }
    fn read_r8(&self, ind:u8) -> u8 {
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
    fn write_r8(&mut self, ind:u8, data:u8) {
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
    fn read_r16(&self, ind:u8) -> u16 {
        self.helper_read_r16(ind, true)
    }
    fn write_r16(&mut self, ind:u8, val:u16) {
        self.helper_write_r16(ind, val, true);
    }
    fn read_r16_sp(&self, ind:u8) -> u16 {
        self.helper_read_r16(ind, false)
    }
    fn write_r16_sp(&mut self, ind:u8, val:u16) {
        self.helper_write_r16(ind, val, false);
    }
    fn helper_read_r16(&self, ind:u8, variant:bool) -> u16 {
        match ind {
            0 => self.regs.read_bc(),
            1 => self.regs.read_de(),
            2 => self.regs.read_hl(),
            3 => {
                if variant {
                    self.SP
                } else {
                    self.regs.read_af()
                }
            }
            _ => unreachable!()
        }
    }
    fn helper_write_r16(&mut self, ind:u8, val:u16, variant:bool) {
        match ind {
            0 => self.regs.write_bc(val),
            1 => self.regs.write_de(val),
            2 => self.regs.write_hl(val),
            3 => {
                if variant {
                    self.SP = val;
                } else {
                    self.regs.write_af(val);
                }
            }
            _ => unreachable!()
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
    fn set_flag(&mut self, i:u8, val:bool) {
        match i {
            0 => {
                if val {
                    self.regs.F |= GbFlags::Z;
                } else {
                    self.regs.F -= GbFlags::Z;
                }
            }
            1 => {
                if val {
                    self.regs.F |= GbFlags::N;
                } else {
                    self.regs.F -= GbFlags::N;
                }
            }
            2 => {
                if val {
                    self.regs.F |= GbFlags::H;
                } else {
                    self.regs.F -= GbFlags::H;
                }
            }
            3 => {
                if val {
                    self.regs.F |= GbFlags::C;
                } else {
                    self.regs.F -= GbFlags::C;
                }
            }
            _ => panic!()
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
        let mut extra_cycles:Option<bool> = None;
        //Index in order ZNHC
        let mut flag_effects:[Option<bool>; 4] = [None,None,None,None];
        //based on https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html
        match x {
            0 => {
                match z {
                    0 => {
                        match y {
                            0 => {},
                            1 => {
                                let addr_lsb = self.fetch_byte() as u16;
                                let addr_msb = self.fetch_byte() as u16;
                                let addr = addr_msb << 8 | addr_lsb;
                                self.write_mem(addr, self.PC.to_le_bytes()[0]);
                                self.write_mem(addr + 1, self.PC.to_le_bytes()[1]);
                            }
                            2 => {
                                //STOP
                                todo!()
                            }
                            3 => {
                                //TODO: make an abstraction over jumping probably
                                let shift = self.fetch_byte() as i8;
                                if shift >= 0 {
                                    self.PC = self.PC.wrapping_add(shift as u16);
                                } else {
                                    self.PC = self.PC.wrapping_sub((shift * -1) as u16);
                                }
                            }
                            4..=7 => {
                                let shift = self.fetch_byte() as i8;
                                if self.check_cond(y - 4) {
                                    extra_cycles = Some(true);
                                    if shift >= 0 {
                                        self.PC = self.PC.wrapping_add(shift as u16);
                                    } else {
                                        self.PC = self.PC.wrapping_sub((shift * -1) as u16);
                                    }
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                    1 => {
                        if q == 0 {
                            //16 bit immediate load
                            let lsb = self.fetch_byte() as u16;
                            let msb = self.fetch_byte() as u16;
                            let val = msb << 8 | lsb;
                            self.write_r16(p, val);
                        } else if q == 1 {
                            //16 bit add
                            let val = self.read_r16(p);
                            flag_effects[H] = Some((self.regs.read_hl() & 0xFFF) + (val & 0xFFF) > 0xFFF);
                            flag_effects[C] = Some(self.regs.read_hl() as u32 + val as u32 > u16::MAX.into());
                            self.regs.write_hl(self.regs.read_hl().wrapping_add(val));
                        } else {unreachable!()}
                    }
                    2 => {
                        match (q,p) {
                            (0,0) => {
                                self.write_mem(self.regs.read_bc(), self.regs.A);
                            }
                            (0,1) => {
                                self.write_mem(self.regs.read_de(), self.regs.A);
                            }
                            (0,2) => {
                                self.write_mem(self.regs.read_hl(), self.regs.A);
                                self.regs.write_hl(self.regs.read_hl().wrapping_add(1));
                            }
                            (0,3) => {
                                self.write_mem(self.regs.read_hl(), self.regs.A);
                                self.regs.write_hl(self.regs.read_hl().wrapping_sub(1));
                            }
                            (1,0) => {
                                self.regs.A = self.read_mem(self.regs.read_bc());
                            }
                            (1,1) => {
                                self.regs.A = self.read_mem(self.regs.read_de());
                            }
                            (1,2) => {
                                self.regs.A = self.read_mem(self.regs.read_hl());
                                self.regs.write_hl(self.regs.read_hl().wrapping_add(1));
                            }
                            (1,3) => {
                                self.regs.A = self.read_mem(self.regs.read_hl());
                                self.regs.write_hl(self.regs.read_hl().wrapping_sub(1));
                            }
                            _ => unreachable!()
                        }
                    }
                    3 => {
                        if q == 0 {
                            //16 bit inc
                            self.write_r16(p, self.read_r16(p).wrapping_add(1));
                        } else if q == 1 {
                            //16 bit dec
                            self.write_r16(p, self.read_r16(p).wrapping_sub(1));
                        } else {unreachable!()}
                    }
                    4 => {
                        //8 bit inc
                        flag_effects[H] = Some((self.read_r8(y) & 0x0F) + 1 > 0x0F);
                        self.write_r8(y, self.read_r8(y).wrapping_add(1));
                        flag_effects[Z] = Some(self.regs.A == 0);
                    }
                    5 => {
                        //8 bit dec
                        flag_effects[H] = Some(self.read_r8(y) & 0x0F == 0);
                        self.write_r8(y, self.read_r8(y).wrapping_sub(1));
                        flag_effects[Z] = Some(self.regs.A == 0);
                    }
                    6 => {
                        //immediate load r8
                        let val = self.fetch_byte();
                        self.write_r8(y, val);
                    }
                    7 => {
                        match y {
                            0 => {
                                //rlca
                                flag_effects[C] = Some(self.regs.A & 1 << 7 > 0);
                                self.regs.A = self.regs.A.rotate_left(1);
                            }
                            1 => {
                                //rrca
                                flag_effects[C] = Some(self.regs.A & 1 > 0);
                                self.regs.A = self.regs.A.rotate_right(1);
                            }
                            2 => {
                                //rla
                                let carry = self.regs.F.contains(GbFlags::C);
                                flag_effects[C] = Some(self.regs.A & 1 << 7 > 0);
                                self.regs.A <<= 1;
                                if carry {
                                    self.regs.A |= 1;
                                } else {
                                    self.regs.A &= !1;
                                }
                            }
                            3 => {
                                //rra
                                let carry = self.regs.F.contains(GbFlags::C);
                                flag_effects[C] = Some(self.regs.A & 1 > 0);
                                self.regs.A >>= 1;
                                if carry {
                                    self.regs.A |= 1 << 7;
                                } else {
                                    self.regs.A &= !(1 << 7);
                                }
                            }
                            4 => {
                                //daa
                                todo!()
                            }
                            5 => {
                                //cpl
                                self.regs.A = !self.regs.A;
                            }
                            6 => {
                                //scf
                            }
                            7 => {
                                //ccf
                                flag_effects[C] = Some(!self.regs.F.contains(GbFlags::C));
                            }
                            _ => unreachable!()
                        }
                    }
                    _ => unreachable!()
                }
            }
            1 => {
                //8 bit LD from register
                if !(y == 7 && z == 7) {
                    self.write_r8(y, self.read_r8(z));
                }
            }
            2 => {
                self.arithmetic_eight(y, self.read_r8(z), &mut flag_effects);
            }
            3 => {
                match z {
                    //immediate 8 bit arithmetic
                    1..=5 | 7 => todo!(),
                    6 => {
                        let val = self.fetch_byte();
                        self.arithmetic_eight(y, val, &mut flag_effects);
                    }
                    _ => todo!(),
                }
            }
            _ => unreachable!()
        }
        for i in 0..4 {
            let flag_effect = match i {
                0 => ZERO_FLAG[opcode as usize],
                1 => SUB_FLAG[opcode as usize],
                2 => HALF_FLAG[opcode as usize],
                3 => CARRY_FLAG[opcode as usize],
                _ => unreachable!()
            };
            match flag_effect {
                FlagEffect::Set | FlagEffect::Unset => {
                    if flag_effects[i as usize].is_some() {
                        eprintln!("{:X} doesn't need to provide flag {}",opcode,i);
                        let val = flag_effects[i as usize].unwrap();
                        if flag_effect == FlagEffect::Set {
                            assert!(val, "{:X} gave wrong flag {}",opcode,i);
                        } else {
                            assert!(!val, "{:X} gave wrong flag {}",opcode,i);
                        }
                    }
                    if flag_effect == FlagEffect::Set {
                        self.set_flag(i, true);
                    } else {
                        self.set_flag(i, false);
                    }
                }
                FlagEffect::NoEffect => {
                    assert!(flag_effects[i as usize].is_none(), 
                    "{:X} shouldn't modify flag {}", opcode, i);
                }
                FlagEffect::Conditional => {
                    self.set_flag(i, flag_effects[i as usize].expect(
                        format!("{:X} didn't modify flag {}",opcode,i).as_str()
                    ));
                }
            }
        }
        if ALT_CLOCK[opcode as usize] == 0 || !extra_cycles.expect(format!(
        "{:X} didn't provide extra cycles condition",opcode).as_str()) {
            CLOCK[opcode as usize]
        } else {
            ALT_CLOCK[opcode as usize]
        }
    }
    fn arithmetic_eight(&mut self, id:u8, val:u8, flag_effects:&mut [Option<bool>;4]) {
        match id {
            0 => {
                flag_effects[H] = Some((self.regs.A & 0x0F) + (val & 0x0F) > 0x0F);
                flag_effects[C] = Some(self.regs.A as u16 + val as u16 > u8::MAX.into());
                self.regs.A = self.regs.A.wrapping_add(val);
            }
            1 => {
                let carry = if self.regs.F.contains(GbFlags::C) {1} else {0};
                flag_effects[H] = Some((self.regs.A & 0x0F) + (val & 0x0F) + carry as u8 > 0x0F);
                flag_effects[C] = Some(self.regs.A as u16 + val as u16 + carry > u8::MAX.into());
                self.regs.A = self.regs.A.wrapping_add(val).wrapping_add(carry as u8);
            }
            2 => {
                flag_effects[H] = Some((self.regs.A & 0x0F) < (val & 0x0F));
                flag_effects[C] = Some(self.regs.A < val);
                self.regs.A = self.regs.A.wrapping_sub(val);
            }
            3 => {
                let carry = if self.regs.F.contains(GbFlags::C) {1} else {0};
                flag_effects[H] = Some((self.regs.A & 0x0F) < (val & 0x0F + carry as u8));
                flag_effects[C] = Some((self.regs.A as u16) < val as u16 + carry);
                self.regs.A = self.regs.A.wrapping_sub(val).wrapping_sub(carry as u8);
            }
            4 => self.regs.A &= val,
            5 => self.regs.A ^= val,
            6 => self.regs.A |= val,
            7 => {
                if self.regs.A < val {
                    flag_effects[C] = Some(true);
                }
                if (self.regs.A & 0x0F) < (val & 0x0F) {
                    flag_effects[H] = Some(true);
                }
            }
            _ => unreachable!()
        }
        flag_effects[Z] = Some(self.regs.A == 0);
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
