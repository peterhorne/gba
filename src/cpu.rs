use bit::{Bit, Bits, SetBit};
use std::ops::{Index, IndexMut};

pub struct Cpu {
    // r0-7:  Unbanked registers
    // r8-14: Banked registers
    // r13:   Stack pointer (SP)
    // r14:   Link register (LR)
    // r15:   Program counter (PC)
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
        let pc = Register(15);
        let cached_pc = self.registers[pc];
        let condition = instruction.bits(28..32);

        if self.condition_passed(condition) {
            match instruction.bits(25..28) {
                0b000 => {
                    match instruction.bits(4..8) {
                        0b0000 if !instruction.bit(20) => {
                            self.status_register_access_instructions(instruction);
                        },
                        0b0001 if instruction.bits(4..28) == 0x120001 => {
                            self.branch_and_exchange(instruction);
                        },
                        0b1001 if instruction.bit(24) => {
                            self.semaphore_instructions(instruction);
                        },
                        0b1001 /* !instruction.bit(24) */ => {
                            self.multiply_instructions(instruction);
                        },
                        0b1011 | 0b1101 | 0b1111 => {
                            self.load_and_store_halfword_or_signed_byte(instruction);
                        },
                        _ => {
                            self.data_processing(instruction);
                        }
                    }
                },
                0b001 if instruction.bit(20) => {
                    self.data_processing(instruction);
                },
                0b001 /* !instruction.bit(20) */ => {
                    self.status_register_access_instructions(instruction);
                },
                0b011 if instruction.bit(4) => {
                    panic!("undefined")
                },
                0b010 | 0b011 /* !instruction.bit(4) */ => {
                    self.load_and_store_word_or_unsigned_byte_instructions(instruction);
                },
                0b100 => {
                    self.load_and_store_multiple_instructions(instruction);
                },
                0b101 => {
                    self.branch(instruction);
                },
                0b110 => {
                    self.coprocessor_data_transfer(instruction);
                },
                0b111 if instruction.bit(24) => {
                    self.software_interrupt(instruction);
                },
                0b111 /* !instruction.bit(24) */ => {
                    self.coprocessor_data_operation(instruction);
                    self.coprocessor_register_transfer(instruction);
                },
                _ => { unreachable!(); }
            }
        }

        if cached_pc == self.registers[pc] {
            self.registers[pc] += 4;
        }
    }

    fn branch(&mut self, instruction: u32) {
        let l = instruction.bit(24);
        let signed_immed = instruction.bits(0..24);
        self.b(l, signed_immed);
    }

    fn branch_and_exchange(&mut self, instruction: u32) {
        let rn = Register(instruction.bits(0..4));
        self.bx(rn);
    }

    fn coprocessor_data_operation(&mut self, instruction: u32) {
        let opcode_1 = instruction.bits(20..24);
        let crn = instruction.bits(16..20);
        let crd = instruction.bits(12..16);
        let coprocessor = instruction.bits(8..12);
        let opcode_2 = instruction.bits(5..8);
        let crm = instruction.bits(0..4);

        self.cdp(coprocessor, opcode_1, crd, crn, crm, opcode_2)
    }

    fn coprocessor_data_transfer(&mut self, instruction: u32) {
        if instruction.bit(20) {
            self.ldc();
        } else {
            self.stc();
        }
    }

    fn coprocessor_register_transfer(&mut self, instruction: u32) {
        if instruction.bit(20) {
            self.mrc();
        } else {
            self.mcr();
        }
    }

    fn data_processing(&mut self, instruction: u32) {
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
        };
    }

    fn load_and_store_halfword_or_signed_byte(&mut self, instruction: u32) {
        if instruction.bits(25..28) == 0
        && instruction.bit(7)
        && instruction.bit(4) {
            let p = instruction.bit(24);
            let u = instruction.bit(23);
            let i = instruction.bit(22);
            let w = instruction.bit(21);
            let l = instruction.bit(20);
            let rn = Register(instruction.bits(16..20));
            let rd = Register(instruction.bits(12..16));
            let offset_a = instruction.bits(8..12);
            let offset_b = instruction.bits(0..4);
            let address = self.addr_mode_3(p, u, i, w, rn, offset_a, offset_b);

            match instruction.bits(4..8) {
                0b1011 => {
                    if l {
                        self.ldrh(rd, address);
                    } else {
                        self.strh(rd, address);
                    }
                },
                0b1101 => { self.strh(rd, address); },
                0b1111 => { self.ldrsh(rd, address); },
                _ => unreachable!(),
            }
        }
    }

    fn load_and_store_multiple_instructions(&mut self, instruction: u32) {
        match (instruction.bit(20), instruction.bit(22), instruction.bit(15)) {
            (true,  true, true)  => self.ldm3(),
            (true,  true, false) => self.ldm2(),
            (true,  false, _)    => self.ldm1(),
            (false, true,  _)    => self.stm2(),
            (false, false, _)    => self.stm1(),
        }
    }

    fn load_and_store_word_or_unsigned_byte_instructions(&mut self, instruction: u32) {
        let i = instruction.bit(25);
        let p = instruction.bit(24);
        let u = instruction.bit(23);
        let b = instruction.bit(22);
        let w = instruction.bit(21);
        let l = instruction.bit(20);
        let rn = Register(instruction.bits(16..20));
        let rd = Register(instruction.bits(12..16));
        let offset = instruction.bits(0..12);
        let address = self.addr_mode_2(i, p, u, w, rn, offset);

        if l {
            if !p && w {
                if b { self.ldrbt() } else { self.ldrt() }
            } else {
                if b { self.ldrb() } else { self.ldr() }
            }
        } else {
            if !p && w {
                if b { self.strbt() } else { self.strt() }
            } else {
                if b { self.strb() } else { self.str() }
            }
        }
    }

    fn multiply_instructions(&mut self, instruction: u32) {
        let long = instruction.bit(23);
        let s = instruction.bit(20);
        let rd = Register(instruction.bits(16..20)); // rd_hi (if long)
        let rn = Register(instruction.bits(12..16)); // rd_lo (if long)
        let rs = Register(instruction.bits(8..12));
        let rm = Register(instruction.bits(0..4));

        if long {
            match instruction.bits(21..23) {
                0b00 => { self.umull(s, rd, rn, rm, rs) },
                0b01 => { self.umlal(s, rd, rn, rm, rs) },
                0b10 => { self.smull(s, rd, rn, rm, rs) },
                0b11 => { self.smlal(s, rd, rn, rm, rs) },
                _    => { unreachable!() }
            }
        } else {
            if instruction.bit(21) {
                self.mla(s, rd, rm, rs, rn);
            } else {
                self.mul(s, rd, rm, rs);
            }
        }
    }

    fn semaphore_instructions(&mut self, instruction: u32) {
        let b = instruction.bit(22);
        let rn = Register(instruction.bits(16..20));
        let rd = Register(instruction.bits(12..16));
        let rm = Register(instruction.bits(0..4));

        if b {
            self.swpb(rd, rm, rn);
        } else {
            self.swp(rd, rm, rn);
        }
    }

    fn software_interrupt(&mut self, instruction: u32) {
        let immediate = instruction.bits(0..24);
        self.swi(immediate);
    }

    fn status_register_access_instructions(&mut self, instruction: u32) {
        if instruction.bit(21) {
            let r = instruction.bit(22);
            let f = instruction.bit(19);
            let s = instruction.bit(18);
            let x = instruction.bit(17);
            let c = instruction.bit(16);
            let address = (); // TODO

            self.msr();
        } else {
            self.mrs();
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
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let c_flag = if self.c() { 1 } else { 0 };
        let result_long = rn_val as u64 + shifter_operand as u64 + c_flag as u64;
        let result = result_long as u32;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(carry_from(result_long));
            self.set_v(overflow_from_add(rn_val, shifter_operand, result));
        }
    }

    fn add(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: add");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result_long = rn_val as u64 + shifter_operand as u64;
        let result = result_long as u32;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(carry_from(result_long));
            self.set_v(overflow_from_add(rn_val, shifter_operand, result));
        }
    }

    fn and(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: and");
        let (shifter_operand, shifter_carry_out) = operand2;
        let result = self.registers[rn] & shifter_operand;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(shifter_carry_out);
        }
    }

    fn b(&mut self, l: bool, signed_immed: u32) {
        println!("Instruction: b");
        if l { self.registers[Register(14)] += 4; }
        let mask = 1 << (24 - 1);
        let sign_extended = (signed_immed ^ mask) - mask;
        self.registers[Register(15)] += sign_extended << 2;
    }

    fn bic(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: bic");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result = rn_val & !shifter_operand;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(shifter_carry_out);
        }
    }

    fn bx(&mut self, rn: Register) {
        println!("Instruction: bx");
        unimplemented!();
    }

    fn cdp(&mut self, coprocessor: u32, opcode_1: u32, crd: u32, crn: u32, crm: u32, opcode_2: u32) {
        println!("Instruction: cdp");
        unimplemented!();
    }

    fn cmn(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: cmn");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result_long = rn_val as u64 + shifter_operand as u64;
        let result = result_long as u32;

        self.set_n(result.bit(31));
        self.set_z(result == 0);
        self.set_c(carry_from(result_long));
        self.set_z(overflow_from_add(rn_val, shifter_operand, result));
    }

    fn cmp(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: cmp");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result = rn_val - shifter_operand;
        self.set_n(result.bit(31));
        self.set_z(result == 0);
        self.set_c(!borrow_from(rn_val, shifter_operand));
        self.set_z(overflow_from_sub(rn_val, shifter_operand, result));
unimplemented!();
    }

    fn eor(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: eor");
        let (shifter_operand, shifter_carry_out) = operand2;
        let result = self.registers[rn] | shifter_operand;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(shifter_carry_out);
        }
    }

    fn ldc(&mut self) {
        println!("Instruction: ldc");
        unimplemented!();
    }

    fn ldm1(&mut self) {
        println!("Instruction: ldm1");
        unimplemented!();
    }

    fn ldm2(&mut self) {
        println!("Instruction: ldm2");
        unimplemented!();
    }

    fn ldm3(&mut self) {
        println!("Instruction: ldm3");
        unimplemented!();
    }

    fn ldr(&mut self) {
        println!("Instruction: ldr");
        unimplemented!();
    }

    fn ldrb(&mut self) {
        println!("Instruction: ldrb");
        unimplemented!();
    }

    fn ldrbt(&mut self) {
        println!("Instruction: ldrbt");
        unimplemented!();
    }

    fn ldrh(&mut self, rd: Register, address: u32) {
        println!("Instruction: ldrh");
        unimplemented!();
    }

    fn ldrsb(&mut self) {
        println!("Instruction: ldrsb");
        unimplemented!();
    }

    fn ldrsh(&mut self, rd: Register, address: u32) {
        println!("Instruction: ldrsh");
        unimplemented!();
    }

    fn ldrt(&mut self) {
        println!("Instruction: ldrt");
        unimplemented!();
    }

    fn mcr(&mut self) {
        println!("Instruction: mcr");
        unimplemented!();
    }

    fn mla(&mut self, s: bool, rd: Register, rm: Register, rs: Register, rn: Register) {
        println!("Instruction: mla");
        unimplemented!();
    }

    fn mov(&mut self, s: bool, rd: Register, operand2: (u32, bool)) {
        println!("Instruction: mov");
        let (shifter_operand, shifter_carry_out) = operand2;
        self.registers[rd] = shifter_operand;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(shifter_operand.bit(31));
            self.set_z(shifter_operand == 0);
            self.set_c(shifter_carry_out);
        }
    }

    fn mrc(&mut self) {
        println!("Instruction: mrc");
        unimplemented!();
    }

    fn mrs(&mut self) {
        println!("Instruction: mrs");
        unimplemented!();
    }

    fn msr(&mut self) {
        println!("Instruction: msr");
        unimplemented!();
    }

    fn mul(&mut self, s: bool, rd: Register, rm: Register, rs: Register) {
        println!("Instruction: mul");
        unimplemented!();
    }

    fn mvn(&mut self, s: bool, rd: Register, operand2: (u32, bool)) {
        println!("Instruction: mvn");
        let (shifter_operand, shifter_carry_out) = operand2;
        let result = !shifter_operand;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(shifter_carry_out);
        }
    }

    fn orr(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: orr");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result = rn_val | shifter_operand;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(shifter_carry_out);
        }
    }

    fn rsb(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: rsb");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result = shifter_operand - rn_val;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(!borrow_from(shifter_operand, rn_val));
            self.set_v(overflow_from_sub(shifter_operand, rn_val, result));
        }
    }

    fn rsc(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: rsc");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let not_c_flag = if self.c() { 0 } else { 1 };
        let result = shifter_operand - rn_val - not_c_flag;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(!borrow_from(shifter_operand, rn_val + not_c_flag));
            self.set_v(overflow_from_sub(shifter_operand, rn_val + not_c_flag, result));
        }
    }

    fn sbc(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: sbc");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let not_c_flag = if self.c() { 0 } else { 1 };
        let result = rn_val - shifter_operand - not_c_flag;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(!borrow_from(rn_val, shifter_operand + not_c_flag));
            self.set_v(overflow_from_sub(rn_val, shifter_operand + not_c_flag, result));
        }
    }

    fn smlal(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: smlal");
        unimplemented!();
    }

    fn smull(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: smull");
        unimplemented!();
    }

    fn stc(&mut self) {
        println!("Instruction: stc");
        unimplemented!();
    }

    fn stm1(&mut self) {
        println!("Instruction: stm1");
        unimplemented!();
    }

    fn stm2(&mut self) {
        println!("Instruction: stm2");
        unimplemented!();
    }

    fn str(&mut self) {
        println!("Instruction: str");
        unimplemented!();
    }

    fn strb(&mut self) {
        println!("Instruction: strb");
        unimplemented!();
    }

    fn strbt(&mut self) {
        println!("Instruction: strbt");
        unimplemented!();
    }

    fn strh(&mut self, rd: Register, address: u32) {
        println!("Instruction: strh");
        unimplemented!();
    }

    fn strt(&mut self) {
        println!("Instruction: strt");
        unimplemented!();
    }

    fn sub(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: sub");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result = rn_val - shifter_operand;
        self.registers[rd] = result;

        if s && rd == Register(15) {
            self.cpsr = self.spsr;
        } else if s {
            self.set_n(result.bit(31));
            self.set_z(result == 0);
            self.set_c(!borrow_from(rn_val, shifter_operand));
            self.set_v(overflow_from_sub(rn_val, shifter_operand, result));
        }
    }

    fn swi(&mut self, immediate: u32) {
        println!("Instruction: swi");
        unimplemented!();
    }

    fn swp(&mut self, rd: Register, rm: Register, rn: Register) {
        println!("Instruction: swp");
        unimplemented!();
    }

    fn swpb(&mut self, rd: Register, rm: Register, rn: Register) {
        println!("Instruction: swpb");
        unimplemented!();
    }

    fn teq(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: teq");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result = rn_val | shifter_operand;
        self.set_n(result.bit(31));
        self.set_z(result == 0);
        self.set_c(shifter_carry_out);
    }

    fn tst(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: tst");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.registers[rn];
        let result = rn_val & shifter_operand;
        self.set_n(result.bit(31));
        self.set_z(result == 0);
        self.set_c(shifter_carry_out);
    }

    fn umlal(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: umlal");
        unimplemented!();
    }

    fn umull(&mut self, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
        println!("Instruction: umull");
        unimplemented!();
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

    fn addr_mode_2(&mut self, i: bool, p: bool, u: bool, w: bool, rn: Register, offset_12: u32) -> u32 {
        let offset = if i {
            let shift_imm = offset_12.bits(7..12);
            let shift = offset_12.bits(5..7);
            let rm = Register(offset_12.bits(0..4));
            let rm_val = self.registers[rm];

            let index = match shift {
                0b00 => { // Logical shift left
                    rm_val << shift_imm
                },
                0b01 => { // Logical shift right
                    if shift_imm == 0 { 0 } else { rm_val >> shift_imm }
                },
                0b10 => { // Arithmetic shift right
                    if shift_imm == 0 {
                        if rm_val.bit(31) { 0xFFFFFFFF } else { 0 }
                    } else {
                        (rm_val as i32 >> shift_imm) as u32
                    }
                },
                0b11 => {
                    if shift_imm == 0 { // Rotate right with extend
                        (if self.c() { 1 } else { 0 }) << 31 | rm_val >> 1
                    } else { // Rotate right
                        rm_val.rotate_right(shift_imm)
                    }
                },
                _ => unreachable!(),
            };

            self.registers[rm]
        } else {
            offset_12
        };

        let rn_val = self.registers[rn];
        let value = if u { rn_val + offset } else { rn_val - offset };
        let address = if p { value } else { rn_val };
        if !p || w { self.registers[rn] = value };

        address
    }

    fn addr_mode_3(&mut self, p: bool, u: bool, i: bool, w: bool, rn: Register, offset_a: u32, offset_b: u32) -> u32 {
        if !p && w { panic!("unpredictable"); }

        let offset = if i {
            (offset_a << 4) | offset_b
        } else {
            self.registers[Register(offset_b)] // rm
        };
        let rn_val = self.registers[rn];
        let value = if u { rn_val + offset } else { rn_val - offset };
        let address = if p { value } else { rn_val };
        if !p || w { self.registers[rn] = value };

        address
    }
}

// Arithmetic flags

fn carry_from(result_long: u64) -> bool {
    result_long & 0x100000000 == 0x100000000
}

fn borrow_from(operand1: u32, operand2: u32) -> bool {
    operand1 < operand2
}

fn overflow_from_add(operand1: u32, operand2: u32, result: u32) -> bool {
    operand1.bit(31) == operand2.bit(31) && result.bit(31) != operand1.bit(31)
}

fn overflow_from_sub(operand1: u32, operand2: u32, result: u32) -> bool {
    operand1.bit(31) != operand2.bit(31) && result.bit(31) != operand1.bit(31)
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
