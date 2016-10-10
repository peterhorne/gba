use bit::{Bit, Bits, SetBit};
use std::ops::{Index, IndexMut};

pub struct Cpu {
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

    pub fn pc(&self) -> u32 {
        self.registers[Register(15)]
    }

    pub fn execute(&mut self, instruction: u32) {
        let condition = instruction.bits(28..32);
        if !self.condition_passed(condition) { return; }

        // Data processing instruction
        if instruction.bits(25..28) == 1 {
            let opcode = instruction.bits(21..25);
            let s = instruction.bit(20);
            let rn = Register(instruction.bits(16..20));
            let rd = Register(instruction.bits(12..16));
            let operand2 = self.addr_mode_1(s, instruction.bits(0..12));

            match opcode {
                0b0000 => { self.and(s, rd, rn, operand2); },
                0b0001 => { self.eor(s, rd, rn, operand2); },
                0b0010 => { self.sub(s, rd, rn, operand2); },
                0b0011 => { self.rsb(s, rd, rn, operand2); },
                0b0100 => { self.add(s, rd, rn, operand2); },
                0b0101 => { self.adc(s, rd, rn, operand2); },
                0b0110 => { self.sbc(s, rd, rn, operand2); },
                0b0111 => { self.rsc(s, rd, rn, operand2); },
                0b1000 => { self.tst(s, rn, operand2); },
                0b1001 => { self.teq(s, rn, operand2); },
                0b1010 => { self.cmp(s, rn, operand2); },
                0b1011 => { self.cmn(s, rn, operand2); },
                0b1100 => { self.orr(s, rd, rn, operand2); },
                0b1101 => { self.mov(s, rd, operand2); },
                0b1110 => { self.bic(s, rd, rn, operand2); },
                0b1111 => { self.mvn(s, rd, operand2); },
                _      => { unreachable!() },
            }

            return;
        }

        // Multiply
        if instruction.bits(22..28) == 0 && instruction.bits(4..8) == 0b1001 {
            let a = instruction.bit(21);
            let s = instruction.bit(20);
            let rd = Register(instruction.bits(16..20));
            let rn = Register(instruction.bits(12..16));
            let rs = Register(instruction.bits(8..12));
            let rm = Register(instruction.bits(0..4));

            if a {
                self.mla(s, rd, rm, rs, rn);
            } else {
                self.mul(s, rd, rm, rs);
            }

            return;
        }

        // Multiply long
        if instruction.bits(23..28) == 1 && instruction.bits(4..8) == 0b1001 {
            let opcode = instruction.bits(21..23);
            let s = instruction.bit(20);
            let rd_hi = Register(instruction.bits(16..20));
            let rd_lo = Register(instruction.bits(12..16));
            let rs = Register(instruction.bits(8..12));
            let rm = Register(instruction.bits(0..4));

            match opcode {
                0b00 => { self.umull(s, rd_hi, rd_lo, rm, rs) },
                0b01 => { self.umlal(s, rd_hi, rd_lo, rm, rs) },
                0b10 => { self.smull(s, rd_hi, rd_lo, rm, rs) },
                0b11 => { self.smlal(s, rd_hi, rd_lo, rm, rs) },
                _    => { unreachable!() }
            }

            return;
        }

        // Swap
        if instruction.bits(23..28) == 0b10
        && instruction.bits(20..22) == 0
        && instruction.bits(4..11) == 0b1001 {
            let b = instruction.bit(22);
            let rn = Register(instruction.bits(16..20));
            let rd = Register(instruction.bits(12..16));
            let rm = Register(instruction.bits(0..4));

            if b {
                self.swpb(rd, rm, rn);
            } else {
                self.swp(rd, rm, rn);
            }

            return;
        }

        panic!("instruction not recognised");
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
            _      => { unreachable!() },
        }
    }

    // Flags

    fn n(&self) -> bool {
        self.cpsr.bit(31)
    }

    fn z(&self) -> bool {
        self.cpsr.bit(30)
    }

    fn c(&self) -> bool {
        self.cpsr.bit(29)
    }

    fn v(&self) -> bool {
        self.cpsr.bit(28)
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

    fn adc(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: adc");
    }

    fn add(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: add");
    }

    fn and(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: and");
    }

    fn b(&mut self) {
        println!("Instruction: b");
    }

    fn bic(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: bic");
    }

    fn bx(&mut self) {
        println!("Instruction: bx");
    }

    fn cdp(&mut self) {
        println!("Instruction: cdp");
    }

    fn cmn(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: cmn");
    }

    fn cmp(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: cmp");
    }

    fn eor(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: eor");
    }

    fn ldc(&mut self) {
        println!("Instruction: ldc");
    }

    fn ldm1(&mut self) {
        println!("Instruction: ldm1");
    }

    fn ldm2(&mut self) {
        println!("Instruction: ldm2");
    }

    fn ldm3(&mut self) {
        println!("Instruction: ldm3");
    }

    fn ldr(&mut self) {
        println!("Instruction: ldr");
    }

    fn ldrb(&mut self) {
        println!("Instruction: ldrb");
    }

    fn ldrbt(&mut self) {
        println!("Instruction: ldrbt");
    }

    fn ldrh(&mut self) {
        println!("Instruction: ldrh");
    }

    fn ldrsb(&mut self) {
        println!("Instruction: ldrsb");
    }

    fn ldrsh(&mut self) {
        println!("Instruction: ldrsh");
    }

    fn ldrt(&mut self) {
        println!("Instruction: ldrt");
    }

    fn mcr(&mut self) {
        println!("Instruction: mcr");
    }

    fn mla(&mut self, s: bool, rd: Register, rm: Register, rs: Register, rn: Register) {
        println!("Instruction: mla");
    }

    fn mov(&mut self, s: bool, rd: Register, operand2: (u32, bool)) {
        println!("Instruction: mov");
    }

    fn mrc(&mut self) {
        println!("Instruction: mrc");
    }

    fn mrs(&mut self) {
        println!("Instruction: mrs");
    }

    fn msr(&mut self) {
        println!("Instruction: msr");
    }

    fn mul(&mut self, s: bool, rd: Register, rm: Register, rs: Register) {
        println!("Instruction: mul");
    }

    fn mvn(&mut self, s: bool, rd: Register, operand2: (u32, bool)) {
        println!("Instruction: mvn");
    }

    fn orr(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: orr");
    }

    fn rsb(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: rsb");
    }

    fn rsc(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: rsc");
    }

    fn sbc(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: sbc");
    }

    fn smlal(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: smlal");
    }

    fn smull(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: smull");
    }

    fn stc(&mut self) {
        println!("Instruction: stc");
    }

    fn stm1(&mut self) {
        println!("Instruction: stm1");
    }

    fn stm2(&mut self) {
        println!("Instruction: stm2");
    }

    fn str(&mut self) {
        println!("Instruction: str");
    }

    fn strb(&mut self) {
        println!("Instruction: strb");
    }

    fn strbt(&mut self) {
        println!("Instruction: strbt");
    }

    fn strh(&mut self) {
        println!("Instruction: strh");
    }

    fn strt(&mut self) {
        println!("Instruction: strt");
    }

    fn sub(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: sub");
    }

    fn swi(&mut self) {
        println!("Instruction: swi");
    }

    fn swp(&mut self, rd: Register, rm: Register, rn: Register) {
        println!("Instruction: swp");
    }

    fn swpb(&mut self, rd: Register, rm: Register, rn: Register) {
        println!("Instruction: swpb");
    }

    fn teq(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: teq");
    }

    fn tst(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: tst");
    }

    fn umlal(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: umlal");
    }

    fn umull(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: umull");
    }

    // Addressing modes

    // Returns (shifter_operand, shifter_carry_out)
    fn addr_mode_1(&self, s: bool, operand: u32) -> (u32, bool) {
        let shifter_operand: u32;
        let shifter_carry_out: bool;

        // 32-bit immediate
        if s {
            let rotate_imm = operand.bits(8..12);
            let immed_8 = operand.bits(0..8);

            shifter_operand = immed_8.rotate_right(rotate_imm * 2);
            shifter_carry_out = if rotate_imm == 0 {
                self.c()
            } else {
                shifter_operand.bit(31)
            };

        // Register
        } else if operand.bits(4..12) == 0 {
            let rm = Register(operand.bits(0..4));
            let rm_val = self.registers[rm];

            shifter_operand = rm_val;
            shifter_carry_out = self.c();

        // Register shift
        } else if operand.bit(4) {
            let rs = Register(operand.bits(8..12));
            let shift = operand.bits(5..7);
            let rm = Register(operand.bits(0..4));
            let rs_val = self.registers[rs];
            let rm_val = self.registers[rm];

            match shift {
                // Logical shift left
                0b00 => {
                    let part = rs_val.bits(0..8) as u8;
                    if part == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = self.c();
                    } else if part < 32 {
                        shifter_operand = rm_val << part;
                        shifter_carry_out = rm_val.bit(32 - part);
                    } else if part == 32 {
                        shifter_operand = 0;
                        shifter_carry_out = rm_val.bit(0);
                    } else /* part > 32 */ {
                        shifter_operand = 0;
                        shifter_carry_out = false;
                    }
                },

                // Logical shift right
                0b01 => {
                    let part = rs_val.bits(0..8) as u8;
                    if part == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = self.c();
                    } else if part < 32 {
                        shifter_operand = rm_val >> part;
                        shifter_carry_out = rm_val.bit(part - 1);
                    } else if part == 32 {
                        shifter_operand = 0;
                        shifter_carry_out = rm_val.bit(31);
                    } else /* part > 32 */ {
                        shifter_operand = 0;
                        shifter_carry_out = false;
                    }
                },

                // Arithmetic shift right
                0b10 => {
                    let part = rs_val.bits(0..8) as u8;
                    if part == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = self.c();
                    } else if part < 32 {
                        shifter_operand = (rm_val as i32 >> part) as u32;
                        shifter_carry_out = rm_val.bit(part - 1);
                    } else /* part >= 32 */ {
                        shifter_operand = if rm_val.bit(31) { 0xFFFFFFFF } else { 0 };
                        shifter_carry_out = rm_val.bit(31);
                    }
                },

                // Rotate right
                0b11 => {
                    let part = rs_val.bits(0..8);
                    let part2 = rs_val.bits(0..4);

                    if part == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = self.c();
                    } else if part2 == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = rm_val.bit(31);
                    } else /* part2 > 0 */ {
                        shifter_operand = rm_val.rotate_right(part2);
                        shifter_carry_out = rm_val.bit(part2 as u8 - 1);
                    }
                },

                _ => { unreachable!() },
            }

        // Immediate shift
        } else {
            let shift_imm = operand.bits(7..12);
            let shift = operand.bits(5..7);
            let rm = Register(operand.bits(0..4));
            let rm_val = self.registers[rm];

            match shift {
                // Logical shift left
                0b00 => {
                    if shift_imm == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = self.c();
                    } else {
                        shifter_operand = rm_val << shift_imm;
                        shifter_carry_out = rm_val.bit(32 - shift_imm as u8);
                    }
                },

                // Logical shift right
                0b01 => {
                    if shift_imm == 0 {
                        shifter_operand = 0;
                        shifter_carry_out = rm_val.bit(31);
                    } else {
                        shifter_operand = rm_val >> shift_imm;
                        shifter_carry_out = rm_val.bit(shift_imm as u8 - 1);
                    }
                },

                // Arithmetic shift right
                0b10 => {
                    if shift_imm == 0 {
                        shifter_operand = if rm_val.bit(31) { 0xFFFFFFFF } else { 0 };
                        shifter_carry_out = rm_val.bit(31);
                    } else {
                        shifter_operand = (rm_val as i32 >> shift_imm) as u32;
                        shifter_carry_out = rm_val.bit(shift_imm as u8 - 1);
                    }
                },

                // Rotate right
                0b11 => {
                    if shift_imm == 0 {
                        // Rotate right with extend
                        let c_flag = if self.c() { 1 } else { 0 };
                        shifter_operand = (c_flag << 31) | (rm_val >> 1);
                        shifter_carry_out = rm_val.bit(0);
                    } else {
                        shifter_operand = rm_val.rotate_right(shift_imm);
                        shifter_carry_out = rm_val.bit(shift_imm as u8 - 1);
                    }
                },

                _ => { unreachable!() },
            }

        };

        (shifter_operand, shifter_carry_out)
    }
}

// Newtype to prevent a register's index being mistaken for it's value.
#[derive(Clone, Copy)]
pub struct Register(pub u32);

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
