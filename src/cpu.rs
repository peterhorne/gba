use bit::{Bit, Bits, SetBit, SetBits};
use bus::{Read, Write};
use core::ops::Range;
use decode::decode;
use execute::execute;
use instruction::{EncodedInstruction, Instruction};
use interrupt_controller::InterruptController;
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
    interrupts: Rc<RefCell<InterruptController>>,
    pipeline: Pipeline,
}

pub const LR: Register = Register(14);
pub const PC: Register = Register(15);

impl Cpu {
    pub fn new(
        memory: MemoryMap,
        interrupts: Rc<RefCell<InterruptController>>,
    ) -> Cpu {
        Cpu {
            regs: Registers::new(),
            cpsr: ProgramStatusRegister::new(),
            spsr: ProgramStatusRegister::new(),
            memory: memory,
            interrupts: interrupts,
            pipeline: Pipeline::new(),
        }
    }

    pub fn tick(&mut self) {
        let pc = self.regs[PC];

        self.pipeline
            .enqueue(pc)
            .map(|addr| self.fetch(addr))
            .map(|inst| decode(inst))
            .map(|inst| execute(self, inst));

        if self.regs[PC] == pc {
            self.incr_pc();
        } else {
            // a branch has occurred
            self.pipeline.flush();
        }

        if self.interrupts.borrow().is_asserted() {
            self.handle_interrupt()
        }
    }

    fn fetch(&self, address: u32) -> EncodedInstruction {
        if self.cpsr.t() {
            EncodedInstruction::Thumb(self.memory.read_halfword(address))
        } else {
            EncodedInstruction::Arm(self.memory.read_word(address))
        }
    }

    fn incr_pc(&mut self) {
        let incr = if self.cpsr.t() { 2 } else { 4 };
        self.regs[PC] += incr;
    }

    fn handle_interrupt(&mut self) {
        self.interrupts.borrow_mut().reset();
        // R14_irq = address of next instruction to be executed + 4
        // SPSR_irq = CPSR
        // CPSR[4:0] = 0b10010
        // CPSR[5] = 0
        // /* CPSR[6] is unchanged */
        // CPSR[7] = 1
        // PC    = 0x00000018
    }
}

/// A fixed length queue of instruction addresses for the CPU to process.
struct Pipeline((Option<u32>, Option<u32>));

impl Pipeline {
    fn new() -> Pipeline {
        Pipeline((None, None))
    }

    /// Add a new address to the pipeline, shifting the last address off the
    /// end and returning it.
    fn enqueue(&mut self, address: u32) -> Option<u32> {
        let (a, b) = self.0;
        self.0 = (Some(address), a);
        b
    }

    /// Empty the pipeline.
    fn flush(&mut self) {
        self.0 = (None, None);
    }
}

#[derive(Clone, Copy)]
pub struct ProgramStatusRegister(u32);

impl ProgramStatusRegister {
    fn new() -> ProgramStatusRegister {
        ProgramStatusRegister(0x1F)
    }

    fn mode(&self) -> ProgramStatusRegisterMode {
        match self.0.bits(0..5) {
            0b10000 => ProgramStatusRegisterMode::User,
            0b10001 => ProgramStatusRegisterMode::FIQ,
            0b10010 => ProgramStatusRegisterMode::IRQ,
            0b10011 => ProgramStatusRegisterMode::Supervisor,
            0b10111 => ProgramStatusRegisterMode::Abort,
            0b11011 => ProgramStatusRegisterMode::Undefined,
            0b11111 => ProgramStatusRegisterMode::System,
            _ => panic!("unpredictable!"),
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
    User,
    FIQ,
    IRQ,
    Supervisor,
    Abort,
    Undefined,
    System,
}

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
