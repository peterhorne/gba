use bit::{BitAt, SetBit};
use std::ops::{Index, IndexMut};

struct Cpu {
    // General purpose registers
    // Register 15 is the Program Counter (PC)
    registers: Registers,

    // Current Program Status Register
    cpsr: u32,

    // Saved Program Status Register
    spsr: u32,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            registers: Registers::new(),
            cpsr: 0,
            spsr: 0,
        }
    }

    pub fn execute(&mut self, instruction: u32) {
        let condition = instruction.bits_at(28, 31);
        if !self.condition_passed { return };

        // Data processing instruction
        if instruction.bits_at(25, 27) == 0b001 {
            let opcode = instruction.bits_at(21, 24);
            let s = instruction.bit_at(20);
            let rn = Register::new(instruction.bits_at(16, 19));
            let rd = Register::new(instruction.bits_at(12, 15));
            let operand2 = self.addr_mode_1(s, instruction.bits_at(0, 11));
        }
    }

    fn condition_passed(&self, condition: u32) -> bool {
        let z = self.z();
        let c = self.c();
        let n = self.n();
        let v = self.v();

        match condition {
            // EQ, Equal
            0b0000 => { z },

            // NE, Not equal
            0b0001 => { !z },

            // CS/HS, Carry set/unsigned higher or same
            0b0010 => { c },

            // CC/LO, Carry clear/unsigned lower
            0b0011 => { !c },

            // MI, Minus/Negative
            0b0100 => { n },

            // PL, Plus/positive or zero
            0b0101 => { !n },

            // VS, Overflow
            0b0110 => { v },

            // VC, No overflow
            0b0111 => { !v },

            // HI, Unsigned higher
            0b1000 => { c && !z },

            // LS, Unsigned lower or same
            0b1001 => { !c || z },

            // GE, Signed greater than or equal
            0b1010 => { n == v },

            // LT, Signed less than
            0b1011 => { n != v },

            // GT, Signed greater than
            0b1100 => { !z && n == v },

            // LE, Signed less than or equal
            0b1101 => { z || n != v },

            // AL, Always (unconditional)
            0b1110 => { true },

            // NV
            0b1111 => { panic!("unpredictable") },
        }
    }

    // Flags

    fn n(&self) -> bool {
        self.cpsr.bit_at(31)
    }

    fn z(&self) -> bool {
        self.cpsr.bit_at(30)
    }

    fn c(&self) -> bool {
        self.cpsr.bit_at(29)
    }

    fn v(&self) -> bool {
        self.cpsr.bit_at(28)
    }

    fn set_n(&mut self, value: bool) {
        self.cpsr.set_bit(31, value);
    }

    fn set_z(&mut self, value: bool) {
        self.cpsr.set_bit(30, value);
    }

    fn set_c(&mut self, value: bool) {
        self.cpsr.set_bit(29, value);
    }

    fn set_v(&mut self, value: bool) {
        self.cpsr.set_bit(28, value);
    }

    // Instructions

    fn adc(&mut self) { }

    fn add(&mut self) { }

    fn and(&mut self) { }

    fn b(&mut self) { }

    fn bic(&mut self) { }

    fn bx(&mut self) { }

    fn cdp(&mut self) { }

    fn cmn(&mut self) { }

    fn cmp(&mut self) { }

    fn eor(&mut self) { }

    fn ldc(&mut self) { }

    fn ldm1(&mut self) { }

    fn ldm2(&mut self) { }

    fn ldm3(&mut self) { }

    fn ldr(&mut self) { }

    fn ldrb(&mut self) { }

    fn ldrbt(&mut self) { }

    fn ldrh(&mut self) { }

    fn ldrsb(&mut self) { }

    fn ldrsh(&mut self) { }

    fn ldrt(&mut self) { }

    fn mcr(&mut self) { }

    fn mla(&mut self) { }

    fn mov(&mut self) { }

    fn mrc(&mut self) { }

    fn mrs(&mut self) { }

    fn msr(&mut self) { }

    fn mul(&mut self) { }

    fn mvn(&mut self) { }

    fn orr(&mut self) { }

    fn rsb(&mut self) { }

    fn rsc(&mut self) { }

    fn sbc(&mut self) { }

    fn smlal(&mut self) { }

    fn smull(&mut self) { }

    fn stc(&mut self) { }

    fn stm1(&mut self) { }

    fn stm2(&mut self) { }

    fn str(&mut self) { }

    fn strb(&mut self) { }

    fn strbt(&mut self) { }

    fn strh(&mut self) { }

    fn strt(&mut self) { }

    fn sub(&mut self) { }

    fn swi(&mut self) { }

    fn swp(&mut self) { }

    fn swpb(&mut self) { }

    fn teq(&mut self) { }

    fn tst(&mut self) { }

    fn umlal(&mut self) { }

    fn umull(&mut self) { }

    // Addressing modes

    // Returns (shifter_operand, shifter_carry_out)
    fn addr_mode_1(&self, s: bool, operand: u32) -> (u32, u32) {
        // 32-bit immediate
        if s {
            let rotate_imm = operand.bits_at(8, 11);
            let immed_8 = operand.bits_at(0, 7);

            let shifter_operand = immed_8.rotate_right(rotate_imm * 2);
            let shifter_carry_out = if rotate_imm == 0 {
                self.c()
            } else {
                shifter_operand.bit_at(31)
            };

            (shifter_operand, shifter_carry_out)

        // Register
        } else if operand.bits_at(4, 11) == 0 {
            let rm = Registers::new(operand.bits_at(0, 3));
            let rm_val = self.registers[rm];

            let shifter_operand = rm_val;
            let shifter_carry_out = self.c();

            (shifter_operand, shifter_carry_out)

        // Register shift
        } else if operand.bit_at(4) {
            let rs = Register::new(operand.bits_at(8, 11));
            let shift = operand.bits_at(5, 6);
            let rm = Register::new(operand.bits_at(0, 3));
            let rs_val = self.registers[rs];
            let rm_val = self.registers[rm];

            match shift {
                // Logical shift left
                0b00 => {
                    let part = rs_val.bits_at(0, 7);
                    if part == 0 {
                        let shifter_operand = rm_val;
                        let shifter_carry_out = self.c();
                    } else if part < 32 {
                        let shifter_operand = rm_val << part;
                        let shifter_carry_out = rm_val.bit_at(32 - part);
                    } else if part == 32 {
                        let shifter_operand = 0;
                        let shifter_carry_out = rm.val.bit_at(0);
                    } else /* part > 32 */ {
                        let shifter_operand = 0;
                        let shifter_carry_out = 0;
                    }
                },

                // Logical shift right
                0b01 => {
                    let part = rs_val.bits_at(0, 7);
                    if part == 0 {
                        let shifter_operand = rm_val;
                        let shifter_carry_out = self.c();
                    } else if part < 32 {
                        let shifter_operand = rm_val >> part;
                        let shifter_carry_out = rm_val.bit_at(part - 1);
                    } else if part == 32 {
                        let shifter_operand = 0;
                        let shifter_carry_out = rm.val.bit_at(31);
                    } else /* part > 32 */ {
                        let shifter_operand = 0;
                        let shifter_carry_out = 0;
                    }
                },

                // Arithmetic shift right
                0b10 => {
                    let part = rs_val.bits_at(0, 7);
                    if part == 0 {
                        let shifter_operand = rm_val;
                        let shifter_carry_out = self.c();
                    } else if part < 32 {
                        let shifter_operand = (rm_val as i32) >> part;
                        let shifter_carry_out = rm_val.bit_at(part - 1);
                    } else if part >= 32 {
                        let shifter_operand = if rm_val.bit_at(31) { 0xFFFFFFFF } else { 0 };
                        let shifter_carry_out = rm_val.bit_at(31);
                    }
                },

                // Rotate right
                0b11 => {
                    let part = rs_val.bits_at(0, 7);
                    let part2 = rs_val.bits_at(0,4);

                    if part == 0 {
                        let shifter_operand = rm_val;
                        let shifter_carry_out = self.c();
                    } else if part2 == 0 {
                        let shifter_operand = rm_val;
                        let shifter_carry_out = rm_val.bit_at(31);
                    } else /* part2 > 0 */ {
                        let shifter_operand = rm_val.rotate_right(part2);
                        let shifter_carry_out = rm_val.bit_at(part2 - 1);
                    }
                },
            }

            (shifter_operand, shifter_carry_out)

        // Immediate shift
        } else {
            let shift_imm = operand.bits_at(7, 11);
            let shift = operand.bits_at(5, 6);
            let rm = Register::new(operand.bits_at(0, 3));
            let rm_val = self.registers[rm];

            match shift {
                // Logical shift left
                0b00 => {
                    if shift_imm == 0 {
                        let shifter_operand = rm_val;
                        let shifter_carry_out = self.c();
                    } else {
                        let shifter_operand = rm_val << shift_imm;
                        let shifter_carry_out = rm_val.bits_at(32 - shift_imm);
                    }
                },

                // Logical shift right
                0b01 => {
                    if shift_imm == 0 {
                        let shifter_operand = 0;
                        let shifter_carry_out = rm_val.bits_at(31);
                    } else {
                        let shifter_operand = rm_val >> shift_imm;
                        let shifter_carry_out = rm_val.bits_at(shift_imm - 1);
                    }
                },

                // Arithmetic shift right
                0b10 => {
                    if shift_imm == 0 {
                        let shifter_operand = if rm_val.bits_at(31) { 0xFFFFFFFF } else { 0 };
                        let shifter_carry_out = rm_val.bits_at(31);
                    } else {
                        let shifter_operand = (rm_val as i32) >> shift_imm;
                        let shifter_carry_out = rm_val[shift_imm - 1];
                    }
                },

                // Rotate right
                0b11 => {
                    if shift_imm == 0 {
                        // TODO: Rotate right with extend
                    } else {
                        let shifter_operand = rm_val.rotate_right(shift_imm);
                        let shifter_carry_out = rm_val[shift_imm - 1];
                    }
                },
            }

            (shifter_operand, shifter_carry_out)
        }
    }
}

// Newtype to prevent a register's index being mistaken for it's value.
#[derive(Clone, Copy)]
pub struct Register(pub u8);

pub struct Registers([u32; 16]);

impl Registers {
    fn new() -> Registers { Registers([0; 16]) }
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
