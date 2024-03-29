//TODO: 
//halt and stop behavior
//interrupts
use crate::tables::*;
use crate::mem::{Mem,FlatMem};
use bitflags::bitflags;
//used to index into flag effect arrays
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
#[allow(non_snake_case)]
pub struct CPU {
    regs:Registers,
    SP: u16,
    PC: u16,
    mem: Box<dyn Mem>,
}
impl Default for CPU {
    fn default() -> Self {
        CPU {
            regs:Registers::default(),
            SP: 0xFFFE,
            PC: 0x0100,
            mem: Box::new(FlatMem::default()),
        }
    }
}
impl CPU {
    //abstract over read/write for later bus
    fn read_mem(&self, addr:u16) -> u8 {
        self.mem.read(addr)
    }
    fn write_mem(&mut self, addr:u16, data:u8) {
        self.mem.write(addr, data);
    }
    fn borrow_mem(&mut self, addr:u16) -> &mut u8 {
        self.mem.borrow_mem(addr)
    }
    fn fetch_byte(&mut self) -> u8 {
        let val = self.read_mem(self.PC);
        self.PC = self.PC.wrapping_add(1);
        val
    }
    fn fetch_u16(&mut self) -> u16 {
        let lsb:u16 = self.fetch_byte().into();
        let msb:u16 = self.fetch_byte().into();
        msb << 8 | lsb
    }
    fn push_stack(&mut self, val:u16) {
        self.SP = self.SP.wrapping_sub(1);
        self.write_mem(self.SP, val.to_le_bytes()[1]);
        self.SP = self.SP.wrapping_sub(1);
        self.write_mem(self.SP, val.to_le_bytes()[0]);
    }
    fn pop_stack(&mut self) -> u16 {
        let lsb:u16 = self.read_mem(self.SP).into();
        self.SP = self.SP.wrapping_add(1);
        let msb:u16 = self.read_mem(self.SP).into();
        self.SP = self.SP.wrapping_add(1);
        msb << 8 | lsb
    }
    fn call(&mut self, addr:u16) {
        self.push_stack(self.PC);
        self.PC = addr;
    }
    fn borrow_r8(&mut self, ind:u8) -> &mut u8 {
        match ind {
            0 => &mut self.regs.B,
            1 => &mut self.regs.C,
            2 => &mut self.regs.D,
            3 => &mut self.regs.E,
            4 => &mut self.regs.H,
            5 => &mut self.regs.L,
            6 => self.borrow_mem(self.regs.read_hl()),
            7 => &mut self.regs.A,
            _ => panic!("Invalid index")
        }
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
        self.helper_read_r16(ind, false)
    }
    fn write_r16_sp(&mut self, ind:u8, val:u16) {
        match ind {
            0 => self.regs.write_bc(val),
            1 => self.regs.write_de(val),
            2 => self.regs.write_hl(val),
            3 => self.SP = val,
            _ => unreachable!()
        }
    }
    fn read_r16_sp(&self, ind:u8) -> u16 {
        self.helper_read_r16(ind, true)
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
    fn check_cond(&self, ind:u8) -> bool {
        match ind {
            0 => !self.regs.F.contains(GbFlags::Z),
            1 => self.regs.F.contains(GbFlags::Z),
            2 => !self.regs.F.contains(GbFlags::C),
            3 => self.regs.F.contains(GbFlags::C),
            _ => unreachable!()
        }
    }
    fn set_flag(&mut self, id:u8, val:bool) {
        match id {
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
    pub fn tick(&mut self) -> u8{
        let opcode = self.fetch_byte();
        self.execute(opcode)
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
                flag_effects[H] = Some((self.regs.A & 0x0F) < ((val & 0x0F) + carry as u8));
                flag_effects[C] = Some((self.regs.A as u16) < val as u16 + carry);
                self.regs.A = self.regs.A.wrapping_sub(val).wrapping_sub(carry as u8);
            }
            4 => self.regs.A &= val,
            5 => self.regs.A ^= val,
            6 => self.regs.A |= val,
            7 => {
                flag_effects[C] = Some(self.regs.A < val);
                flag_effects[H] = Some((self.regs.A & 0x0F) < (val & 0x0F));
            }
            _ => unreachable!()
        }
        if id != 7 {
            flag_effects[Z] = Some(self.regs.A == 0);
        } else {
            flag_effects[Z] = Some(self.regs.A == val);
        }
    }
    pub fn execute(&mut self, opcode: u8) -> u8 {
        //replace with cb opcode if triggered
        let mut executed = opcode;
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
        let mut prefixed = false;
        match x {
            0 => {
                match z {
                    0 => {
                        match y {
                            0 => {},
                            1 => {
                                let addr_lsb:u16 = self.fetch_byte().into();
                                let addr_msb:u16 = self.fetch_byte().into();
                                let addr = addr_msb << 8 | addr_lsb;
                                self.write_mem(addr, self.SP.to_le_bytes()[0]);
                                self.write_mem(addr + 1, self.SP.to_le_bytes()[1]);
                            }
                            2 => {
                                //STOP
                                _ = self.fetch_byte()
                            }
                            3 => {
                                let shift = unsafe {
                                    std::mem::transmute::<u8,i8>(self.fetch_byte())
                                };
                                if shift >= 0 {
                                    self.PC = self.PC.wrapping_add(shift.try_into().unwrap());
                                } else {
                                    self.PC = self.PC.wrapping_sub((shift as i16 * -1).try_into().unwrap());
                                }
                            }
                            4..=7 => {
                                let shift = unsafe {
                                    std::mem::transmute::<u8,i8>(self.fetch_byte())
                                };
                                if self.check_cond(y - 4) {
                                    extra_cycles = Some(true);
                                    if shift >= 0 {
                                        self.PC = self.PC.wrapping_add(shift.try_into().unwrap());
                                    } else {
                                        self.PC = self.PC.wrapping_sub((shift as i16 * -1).try_into().unwrap());
                                    }
                                } else {
                                    extra_cycles = Some(false);
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                    1 => {
                        if q == 0 {
                            //16 bit immediate load
                            let val = self.fetch_u16();
                            self.write_r16_sp(p, val);
                        } else if q == 1 {
                            //16 bit add
                            let val = self.read_r16_sp(p);
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
                            self.write_r16_sp(p, self.read_r16_sp(p).wrapping_add(1));
                        } else if q == 1 {
                            //16 bit dec
                            self.write_r16_sp(p, self.read_r16_sp(p).wrapping_sub(1));
                        } else {unreachable!()}
                    }
                    4 => {
                        //8 bit inc
                        flag_effects[H] = Some((self.read_r8(y) & 0x0F) + 1 > 0x0F);
                        self.write_r8(y, self.read_r8(y).wrapping_add(1));
                        flag_effects[Z] = Some(self.read_r8(y) == 0);
                        flag_effects[N] = Some(false);
                    }
                    5 => {
                        //8 bit dec
                        flag_effects[H] = Some(self.read_r8(y) & 0x0F == 0);
                        self.write_r8(y, self.read_r8(y).wrapping_sub(1));
                        flag_effects[Z] = Some(self.read_r8(y) == 0);
                        flag_effects[N] = Some(true);
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
                                flag_effects[C] = Some(self.regs.A & (1 << 7) > 0);
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
                                let mut offset = 0;
                                if (!self.regs.F.contains(GbFlags::N) && self.regs.A & 0xF > 0x9) 
                                || self.regs.F.contains(GbFlags::H) {
                                    offset |= 0x06;
                                }
                                if (!self.regs.F.contains(GbFlags::N) && self.regs.A > 0x99)
                                || self.regs.F.contains(GbFlags::C) {
                                    offset |= 0x60;
                                    flag_effects[C] = Some(true);
                                } else {
                                    flag_effects[C] = Some(false);
                                }
                                if self.regs.F.contains(GbFlags::N) {
                                    self.regs.A = self.regs.A.wrapping_sub(offset);
                                } else {
                                    self.regs.A = self.regs.A.wrapping_add(offset);
                                }
                                flag_effects[Z] = Some(self.regs.A == 0);
                                flag_effects[H] = Some(false);
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
                    0 => {
                        match y {
                            0..=3 => {
                                if self.check_cond(y) {
                                    extra_cycles = Some(true);
                                    let addr = self.pop_stack();
                                    self.PC = addr;
                                } else {
                                    extra_cycles = Some(false);
                                }
                            }
                            4 => {
                                let offset:u16 = self.fetch_byte().into();
                                self.write_mem(0xFF00 + offset, self.regs.A);
                            }
                            5 => {
                                let next = self.fetch_byte();
                                flag_effects[C] = Some((self.SP & 0xFF) + next as u16 > u8::MAX.into());
                                flag_effects[H] = Some((self.SP & 0x0F) + (next & 0x0F) as u16 > 0x0F_u16);
                                let shift = unsafe {
                                    std::mem::transmute::<u8,i8>(next)
                                };
                                if shift >= 0 {
                                    let val:u16 = shift.try_into().unwrap();
                                    self.SP = self.SP.wrapping_add(val);
                                } else {
                                    let val:u16 = (shift as i16 * -1).try_into().unwrap();
                                    self.SP = self.SP.wrapping_sub(val);
                                }
                            }
                            6 => {
                                let offset:u16 = self.fetch_byte().into();
                                self.regs.A = self.read_mem(0xFF00 + offset);
                            }
                            7 => {
                                let next = self.fetch_byte();
                                let shift = unsafe {
                                    std::mem::transmute::<u8,i8>(next)
                                };
                                flag_effects[C] = Some((self.SP & 0xFF) + next as u16 > u8::MAX.into());
                                flag_effects[H] = Some((self.SP & 0x0F) + (next & 0x0F) as u16 > 0x0F_u16);
                                if shift >= 0 {
                                    let val:u16 = shift.try_into().unwrap();
                                    self.regs.write_hl(self.SP.wrapping_add(val));
                                } else {
                                    let val:u16 = (shift as i16 * -1).try_into().unwrap();
                                    self.regs.write_hl(self.SP.wrapping_sub(val));
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                    1 => {
                        match q {
                            0 => {
                                let val = self.pop_stack();
                                if opcode != 0xF1 {
                                    assert!(p != 3);
                                    self.write_r16_sp(p, val);
                                } else {
                                    self.regs.A = ((val & 0xFF00) >> 8).try_into().unwrap();
                                    flag_effects[Z] = Some(val & (1 << 7) > 0);
                                    flag_effects[N] = Some(val & (1 << 6) > 0);
                                    flag_effects[H] = Some(val & (1 << 5) > 0);
                                    flag_effects[C] = Some(val & (1 << 4) > 0);
                                }
                            }
                            1 => {
                                match p {
                                    0 => {
                                        let addr = self.pop_stack();
                                        self.PC = addr;
                                    }
                                    1 => {
                                        let addr = self.pop_stack();
                                        self.PC = addr;
                                        //TODO: put an interrupt here
                                    }
                                    2 => self.PC = self.regs.read_hl(),
                                    3 => self.SP = self.regs.read_hl(),
                                    _ => unreachable!()
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                    2 => {
                        match y {
                            0..=3 => {
                                let addr = self.fetch_u16();
                                if self.check_cond(y) {
                                    extra_cycles = Some(true);
                                    self.PC = addr;
                                } else {
                                    extra_cycles = Some(false);
                                }
                            }
                            4 => {
                                self.write_mem(0xFF00 + self.regs.C as u16, self.regs.A);
                            }
                            5 => {
                                let addr = self.fetch_u16();
                                self.write_mem(addr, self.regs.A);
                            }
                            6 => {
                                self.regs.A = self.read_mem(0xFF00 + self.regs.C as u16);
                            }
                            7 => {
                                let addr = self.fetch_u16();
                                self.regs.A = self.read_mem(addr);
                            }
                            _ => unreachable!()
                        }
                    }
                    3 => {
                        match y {
                            0 => {
                                self.PC = self.fetch_u16();
                            }
                            1 => {
                                //cb
                                prefixed = true;
                                let next = self.fetch_byte();
                                executed = next;
                                let x = (next & 0b11000000) >> 6;
                                let y = (next & 0b00111000) >> 3;
                                let z = next & 0b00000111;
                                match x {
                                    0 => {
                                        //rotate operation y with register z
                                        match y {
                                            0 => {
                                                //RLC
                                                flag_effects[C] = Some(*self.borrow_r8(z) & (1 << 7) > 0);
                                                *self.borrow_r8(z) = self.borrow_r8(z).rotate_left(1);
                                            }
                                            1 => {
                                                //RRC
                                                flag_effects[C] = Some(*self.borrow_r8(z) & 1 > 0);
                                                *self.borrow_r8(z) = self.borrow_r8(z).rotate_right(1);
                                            }
                                            2 => {
                                                //RL
                                                let carry = self.regs.F.contains(GbFlags::C);
                                                flag_effects[C] = Some(*self.borrow_r8(z) & 1 << 7 > 0);
                                                *self.borrow_r8(z) <<= 1;
                                                if carry {
                                                    *self.borrow_r8(z) |= 1;
                                                } else {
                                                    *self.borrow_r8(z) &= !1;
                                                }
                                            }
                                            3 => {
                                                //RR
                                                let carry = self.regs.F.contains(GbFlags::C);
                                                flag_effects[C] = Some(*self.borrow_r8(z) & 1 > 0);
                                                *self.borrow_r8(z) >>= 1;
                                                if carry {
                                                    *self.borrow_r8(z) |= 1 << 7;
                                                } else {
                                                    *self.borrow_r8(z) &= !(1 << 7);
                                                }
                                            }
                                            4 => {
                                                //SLA
                                                flag_effects[C] = Some(*self.borrow_r8(z) & (1 << 7) > 0);
                                                *self.borrow_r8(z) <<= 1;
                                            }
                                            5 => {
                                                //SRA
                                                flag_effects[C] = Some(*self.borrow_r8(z) & 1 > 0);
                                                let temp = *self.borrow_r8(z) & (1 << 7);
                                                *self.borrow_r8(z) >>= 1;
                                                *self.borrow_r8(z) |= temp;
                                            }
                                            6 => {
                                                //SWAP
                                                let reg = self.borrow_r8(z);
                                                let low_nibble = *reg & 0xF;
                                                *reg >>= 4;
                                                *reg |= low_nibble << 4;
                                            }
                                            7 => {
                                                //SRL
                                                flag_effects[C] = Some(*self.borrow_r8(z) & 1 > 0);
                                                *self.borrow_r8(z) >>= 1;
                                            }
                                            _ => unreachable!()
                                        }
                                        flag_effects[Z] = Some(*self.borrow_r8(z) == 0);
                                    }
                                    1 => {
                                        let val = self.read_r8(z);
                                        flag_effects[Z] = Some(val & (1 << y) == 0);
                                    }
                                    2 => {
                                        let mut val = self.read_r8(z);
                                        val &= !(1 << y);
                                        self.write_r8(z, val);
                                    }
                                    3 => {
                                        let mut val = self.read_r8(z);
                                        val |= 1 << y;
                                        self.write_r8(z, val);
                                    }
                                    _ => unreachable!()
                                }
                            }
                            2..=5 => panic!("Invalid opcode {:X}", opcode),
                            //DI
                            6 => todo!(),
                            //EI
                            7 => todo!(),
                            _ => unreachable!()
                        }
                    }
                    4 => {
                        match y {
                            0..=3 => {
                                let addr = self.fetch_u16();
                                if self.check_cond(y) {
                                    extra_cycles = Some(true);
                                    self.call(addr);
                                } else {
                                    extra_cycles = Some(false);
                                }
                            }
                            4..=7 => panic!("Invalid opcode {:X}", opcode),
                            _ => unreachable!()
                        }
                    }
                    5 => {
                        match q {
                            0 => {
                                let val = self.read_r16(p);
                                self.push_stack(val);
                            }
                            1 => {
                                match p {
                                    0 => {
                                        let addr = self.fetch_u16();
                                        self.call(addr);
                                    }
                                    1..=3 => panic!("Invalid opcode {:X}", opcode),
                                    _ => unreachable!()
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                    6 => {
                        //immediate 8 bit arithmetic
                        let val = self.fetch_byte();
                        self.arithmetic_eight(y, val, &mut flag_effects);
                    }
                    7 => {
                        let addr = (y as u16) << 3;
                        self.call(addr);
                    }
                    _ => unreachable!(),
                }
            }
            _ => unreachable!()
        }
        for i in 0..4 {
            let flag_effect = match (i, prefixed) {
                (0, false) => ZERO_FLAG[executed as usize],
                (1, false) => SUB_FLAG[executed as usize],
                (2, false) => HALF_FLAG[executed as usize],
                (3, false) => CARRY_FLAG[executed as usize],
                (0, true) => CB_ZERO_FLAG[executed as usize],
                (1, true) => CB_SUB_FLAG[executed as usize],
                (2, true) => CB_HALF_FLAG[executed as usize],
                (3, true) => CB_CARRY_FLAG[executed as usize],
                _ => unreachable!()
            };
            match flag_effect {
                FlagEffect::Set | FlagEffect::Unset => {
                    if flag_effects[i as usize].is_some() {
                        let val = flag_effects[i as usize].unwrap();
                        if flag_effect == FlagEffect::Set {
                            assert!(val, "{:X} gave wrong flag {}, prefixed: {}",executed,i, prefixed);
                        } else {
                            assert!(!val, "{:X} gave wrong flag {}, prefixed: {}",executed,i, prefixed);
                        }
                    }
                    if flag_effect == FlagEffect::Set {
                        self.set_flag(i, true);
                    } else {
                        self.set_flag(i, false);
                    }
                }
                FlagEffect::NoEffect => {
                    if flag_effects[i as usize].is_some() {
                        let flag_type = match i {
                            0 => GbFlags::Z,
                            1 => GbFlags::N,
                            2 => GbFlags::H,
                            3 => GbFlags::C,
                            _ => unreachable!()
                        };
                        assert_eq!(flag_effects[i as usize].unwrap(), 
                        self.regs.F.contains(flag_type), "{:X} shouldn't modify
                        and gave wrong flag {}, prefixed: {}", executed, i, prefixed);
                    }
                }
                FlagEffect::Conditional => {
                    self.set_flag(i, flag_effects[i as usize].expect(
                        format!("{:X} didn't modify flag {}, prefixed: {}",executed,i, prefixed).as_str()
                    ));
                }
            }
        }
        if prefixed {
            CB_CLOCK[executed as usize]
        } else {
            if ALT_CLOCK[executed as usize] == 0 || !extra_cycles.expect(format!(
            "{:X} didn't provide extra cycles condition",executed).as_str()) {
                CLOCK[executed as usize]
            } else {
                ALT_CLOCK[executed as usize]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::vec::Vec;
    use json;
    use crate::cpu::{CPU, GbFlags};
    #[test]
    fn jsmoo() {
        //illegal opcodes
        let mut exclude = vec![0xD3, 0xDB, 0xDD, 0xE3, 0xE4, 0xEB, 0xEC, 0xED, 0xF4, 0xFC, 0xFD];
        //halt
        exclude.push(0x76);
        //prefix
        exclude.push(0xCB);
        //ei and di
        exclude.push(0xF3);
        exclude.push(0xFB);
        //stop
        exclude.push(0x10);
        let unprefixed = 0x00..=0xFF;
        let prefixed = 0x00..=0xFF;
        let opcodes:Vec<(u8,bool)> = (unprefixed.filter(|x| !exclude.contains(x)).map(|x| (x, false)))
            .chain(prefixed.map(|x| (x,true))).collect();
        for (opcode, cb) in opcodes {
            let test_id = format!("{:02x}", opcode);
            let path = if cb {
                format!("test_data/jsmoo/cb {}.json", test_id)
            } else {
                format!("test_data/jsmoo/{}.json", test_id)
            };
            let file_contents = fs::read_to_string(path).unwrap();
            let contents = json::parse(&file_contents).unwrap();
            for test in contents.members() {
                let mut cpu = CPU::default();
                let initial = test["initial"].clone();
                let end = test["final"].clone();
                cpu.regs.A = initial["a"].as_u8().unwrap();
                cpu.regs.B = initial["b"].as_u8().unwrap();
                cpu.regs.C = initial["c"].as_u8().unwrap();
                cpu.regs.D = initial["d"].as_u8().unwrap();
                cpu.regs.E = initial["e"].as_u8().unwrap();
                cpu.regs.F = GbFlags::from_bits_retain(initial["f"].as_u8().unwrap());
                cpu.regs.H = initial["h"].as_u8().unwrap();
                cpu.regs.L = initial["l"].as_u8().unwrap();
                cpu.PC = initial["pc"].as_u16().unwrap();
                cpu.SP = initial["sp"].as_u16().unwrap();
                for entry in initial["ram"].members() {
                    cpu.write_mem(entry[0].as_u16().unwrap(), entry[1].as_u8().unwrap());
                }
                let mut cycles = 0;
                for _ in test["cycles"].members() {
                    cycles += 1;
                }
                //convert m cycles to t cycles
                cycles *= 4;
                while cycles > 0 {
                    cycles -= cpu.tick() as i32;
                }
                assert_eq!(cycles,0, "cycle count didn't line up");
                assert_eq!(cpu.regs.A, end["a"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.regs.B, end["b"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.regs.C, end["c"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.regs.D, end["d"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.regs.E, end["e"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.regs.F.bits(), end["f"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.regs.H, end["h"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.regs.L, end["l"].as_u8().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.PC, end["pc"].as_u16().unwrap(), "failed {}", test["name"]);
                assert_eq!(cpu.SP, end["sp"].as_u16().unwrap(), "failed {}", test["name"]);
                for entry in end["ram"].members() {
                    assert_eq!(cpu.read_mem(entry[0].as_u16().unwrap()), entry[1].as_u8().unwrap(),
                    "failed {}", test["name"]);
                }
                println!("{} passed", test["name"]);
            }
        }
    }
}
