use bit::{Bit, Bits, SetBit, SetBits};
use bus::{Read, Write};
use core::ops::Range;
use decode::{decode_arm, decode_thumb};
use execute::execute;
use instruction::Instruction;
use memory_map::MemoryMap;
use std::cell::RefCell;
use std::ops::{Index, IndexMut};
use std::rc::Rc;

pub struct Cpu {
    // r0-7:  Unbanked registers
    // r8-14: Banked registers
    // r13:   Stack pointer (SP)
    // r14:   Link register (LR)
    // r15:   Program counter (PC)
    pub regs: Registers,

    // Current Program Status Register
    pub cpsr: ProgramStatusRegister,

    // Saved Program Status Register
    pub spsr: ProgramStatusRegister,

    pub memory: MemoryMap,
    pipeline: InstructionPipeline,
}

pub const LR: Register = Register(14);
pub const PC: Register = Register(15);

impl Cpu {
    pub fn new(memory: MemoryMap) -> Cpu {
        Cpu {
            regs: Registers::new(),
            cpsr: ProgramStatusRegister::new(),
            spsr: ProgramStatusRegister::new(),
            memory: memory,
            pipeline: InstructionPipeline::empty(),
        }
    }

    pub fn tick(&mut self) {
        let address = self.regs[PC];

        let fetch = self.fetch();
        let decode = self.decode();
        let execute = self.execute();

        if self.regs[PC] == address {
            self.pipeline = InstructionPipeline::new(fetch, decode, execute);
            self.incr_pc();
        } else {
            // A branch has occurred so flush the pipeline
            self.pipeline = InstructionPipeline::new(None, None, execute);
        }
    }

    fn fetch(&self) -> Option<EncodedInstruction> {
        let address = self.regs[PC];

        let bits = if self.cpsr.t() {
            EncodedInstruction::Thumb(self.memory.read_halfword(address))
        } else {
            EncodedInstruction::Arm(self.memory.read_word(address))
        };

        Some(bits)
    }

    fn decode(&mut self) -> Option<Instruction> {
        self.pipeline.fetch.map(|bits| {
            match bits {
                EncodedInstruction::Thumb(bits) => decode_thumb(bits),
                EncodedInstruction::Arm(bits) => decode_arm(bits),
            }
        })
    }

    fn execute(&mut self) -> Option<Instruction> {
        self.pipeline.decode.map(|inst| {
            execute(self, inst.clone());
            inst
        })
    }

    fn incr_pc(&mut self) {
        let incr = if self.cpsr.t() { 2 } else { 4 };
        self.regs[PC] += incr;
    }
}

pub struct InstructionPipeline {
    fetch: Option<EncodedInstruction>,
    decode: Option<Instruction>,
    execute: Option<Instruction>,
}

impl InstructionPipeline {
    fn new(fetch: Option<EncodedInstruction>,
           decode: Option<Instruction>,
           execute: Option<Instruction>) -> InstructionPipeline {
        InstructionPipeline { fetch, decode, execute }
    }

    fn empty() -> InstructionPipeline {
        InstructionPipeline {
            fetch: None,
            decode: None,
            execute: None
        }
    }
}

#[derive(Clone, Copy)]
enum EncodedInstruction {
    Arm(u32),
    Thumb(u16),
}

#[derive(Clone, Copy)]
pub struct ProgramStatusRegister(u32);

impl ProgramStatusRegister {
    fn new() -> ProgramStatusRegister {
        ProgramStatusRegister(0x1F)
    }

    fn mode(&self) -> ProgramStatusRegisterMode {
        match self.0.bits(0..5) {
            0b10000 => { ProgramStatusRegisterMode::User },
            0b10001 => { ProgramStatusRegisterMode::FIQ },
            0b10010 => { ProgramStatusRegisterMode::IRQ },
            0b10011 => { ProgramStatusRegisterMode::Supervisor },
            0b10111 => { ProgramStatusRegisterMode::Abort },
            0b11011 => { ProgramStatusRegisterMode::Undefined },
            0b11111 => { ProgramStatusRegisterMode::System },
            _       => { panic!("unpredictable!") },
        }
    }

    pub fn to_bits(&self) -> u32 {
        self.0
    }

    // Program status register modes

    pub fn is_priviledged(&self) -> bool {
        self.mode() != ProgramStatusRegisterMode::User
    }

    pub fn has_spsr(&self) -> bool {
        self.mode() != ProgramStatusRegisterMode::User
            && self.mode() != ProgramStatusRegisterMode::System
    }

    // Flags

    pub fn n(&self) -> bool {
        self.0.bit(31)
    }

    pub fn z(&self) -> bool {
        self.0.bit(30)
    }

    pub fn c(&self) -> bool {
        self.0.bit(29)
    }

    pub fn v(&self) -> bool {
        self.0.bit(28)
    }

    pub fn i(&self) -> bool {
        self.0.bit(7)
    }

    pub fn t(&self) -> bool {
        self.0.bit(5)
    }

    pub fn set_n(&mut self, value: bool) {
        self.0.set_bit(31, value);
    }

    pub fn set_z(&mut self, value: bool) {
        self.0.set_bit(30, value);
    }

    pub fn set_c(&mut self, value: bool) {
        self.0.set_bit(29, value);
    }

    pub fn set_v(&mut self, value: bool) {
        self.0.set_bit(28, value);
    }

    pub fn set_i(&mut self, value: bool) {
        self.0.set_bit(7, value);
    }

    pub fn set_t(&mut self, value: bool) {
        self.0.set_bit(5, value);
    }

    pub fn set_bits(&mut self, range: Range<u8>, value: u32) {
        self.0.set_bits(range, value);
    }
}

#[derive(PartialEq)]
enum ProgramStatusRegisterMode {
    User, FIQ, IRQ, Supervisor, Abort, Undefined, System }

// Newtype to prevent a register's index being mistaken for it's value.
#[derive(Clone, Copy, PartialEq)]
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
