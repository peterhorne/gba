use bit::{Bit, Bits, SetBit, SetBits};
use std::ops::{Index, IndexMut};

pub struct Cpu {
    // r0-7:  Unbanked registers
    // r8-14: Banked registers
    // r13:   Stack pointer (SP)
    // r14:   Link register (LR)
    // r15:   Program counter (PC)
    pub regs: Registers,

    // Current Program Status Register
    pub cpsr: u32,

    // Saved Program Status Register
    pub spsr: u32,

    pub memory: Memory,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            regs: Registers::new(),
            cpsr: 0,
            spsr: 0,
            memory: Memory::new(),
        }
    }

    pub fn pc(&self) -> u32 {
        self.regs[Register(15)]
    }

    // Flags

    pub fn t(&self) -> bool {
        self.cpsr.bit(5)
    }

    pub fn n(&self) -> bool {
        self.cpsr.bit(31)
    }

    pub fn z(&self) -> bool {
        self.cpsr.bit(30)
    }

    pub fn c(&self) -> bool {
        self.cpsr.bit(29)
    }

    pub fn v(&self) -> bool {
        self.cpsr.bit(28)
    }

    pub fn set_t(&mut self, value: bool) {
        self.cpsr.set_bit(5, value);
    }

    pub fn set_n(&mut self, value: bool) {
        self.cpsr.set_bit(31, value);
    }

    pub fn set_z(&mut self, value: bool) {
        self.cpsr.set_bit(30, value);
    }

    pub fn set_c(&mut self, value: bool) {
        self.cpsr.set_bit(29, value);
    }

    pub fn set_v(&mut self, value: bool) {
        self.cpsr.set_bit(28, value);
    }

    // Program status register modes

    pub fn in_a_priviledged_mode(&self) -> bool {
        // Not user mode
        self.cpsr.bits(0..5) != 0b10000
    }

    pub fn current_mode_has_spsr(&self) -> bool {
        // Not User or System mode
        let mode = self.cpsr.bits(0..5);
        mode != 0b10000 && mode != 0b11111
    }
}

// Newtype to prevent a register's index being mistaken for it's value.
#[derive(Clone, Copy)]
pub struct Register(pub u32);

pub struct Registers([u32; 16]);

impl Registers {
    fn new() -> Registers {
        Registers([0; 16])
    }
}

impl Index<Register> for Registers {
    type Output = u32;

    fn index(&self, index: Register) -> &u32 {
        &self.0[index.0 as usize]
    }
}

impl IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut u32 {
        &mut self.0[index.0 as usize]
    }
}

impl PartialEq for Register {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

// TODO: memory mapping and alignment
pub struct Memory(Box<[u8; 268_435_456]>);

impl Memory {
    fn new() -> Memory {
        Memory(box [0; 268_435_456])
    }

    pub fn read_byte(&self, address: u32) -> u32 {
        self.0[address as usize] as u32
    }

    pub fn read_halfword(&self, address: u32) -> u32 {
        (self.0[address as usize] as u32)
        + ((self.0[(address + 1) as usize] as u32) << 8)
    }

    pub fn read_word(&self, address: u32) -> u32 {
        (self.0[address as usize] as u32)
        + ((self.0[(address + 1) as usize] as u32) << 8)
        + ((self.0[(address + 2) as usize] as u32) << 16)
        + ((self.0[(address + 3) as usize] as u32) << 24)
    }

    pub fn write_byte(&mut self, address: u32, value: u32) {
        self.0[address as usize] = value as u8;
    }

    pub fn write_halfword(&mut self, address: u32, value: u32) {
        self.0[address as usize] = value as u8;
        self.0[(address + 1) as usize] = (value >> 8) as u8;
    }

    pub fn write_word(&mut self, address: u32, value: u32) {
        self.0[address as usize] = value as u8;
        self.0[(address + 1) as usize] = (value >> 8) as u8;
        self.0[(address + 2) as usize] = (value >> 16) as u8;
        self.0[(address + 3) as usize] = (value >> 24) as u8;
    }
}
