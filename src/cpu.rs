use bit::{Bit, Bits, SetBit, SetBits};
use std::ops::{Index, IndexMut};

pub struct Cpu {
    // r0-7:  Unbanked registers
    // r8-14: Banked registers
    // r13:   Stack pointer (SP)
    // r14:   Link register (LR)
    // r15:   Program counter (PC)
    regs: Registers,

    // Current Program Status Register
    cpsr: u32,

    // Saved Program Status Register
    spsr: u32,

    memory: Memory,
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

    pub fn execute(&mut self, inst: u32) {
        let pc = Register(15);
        let cached_pc = self.regs[pc];
        let condition = inst.bits(28..32);

        if self.condition_passed(condition) {
            if inst.bits(24..28) == 0b0000 && inst.bits(4..8) == 0b1001 {
                self.multiply(inst);
            }

            else if inst.bits(26..28) == 0b00
                && inst.bits(23..25) == 0b10
                && !inst.bit(20)
                && !(!inst.bit(25) && inst.bit(7) && inst.bit(4)) {

                let bit25   = inst.bit(25);
                let nibble2 = inst.bits(4..8);
                let op1     = inst.bits(21..23);

                if (!bit25 && nibble2 == 0b0000) || bit25 && (op1 == 0b01 || op1 == 0b11) {
                    self.status_register(inst);
                }

                if !bit25 && nibble2 == 0b0001 && op1 == 0b01 {
                    self.branch_and_exchange(inst);
                }

            }

            else if inst.bits(25..28) == 0b000
                && inst.bit(7)
                && inst.bit(4)
                && !(!inst.bit(24) && inst.bits(5..7) == 0b00) {

                let bits20to25 = inst.bits(20..25);
                let op1        = inst.bits(5..7);

                if (bits20to25 == 0b10000 || bits20to25 == 0b10100) && op1 == 0b00 {
                    self.semaphore(inst);
                } else {
                    self.load_and_store_halfword_or_signed_byte(inst);
                }
            }

            else {
                match inst.bits(24..28) {
                    0b0000...0b0011 => { self.data_processing(inst) },
                    0b0100...0b0111 => { self.load_and_store_word_or_unsigned_byte_insts(inst) },
                    0b1000...0b1001 => { self.load_and_store_multiple(inst) },
                    0b1010...0b1011 => { self.branch(inst) },
                    0b1100...0b1110 => { self.coprocessor(inst) },
                    0b1111          => { self.software_interrupt(inst) },
                    _ => { unreachable!() }
                }
            }
        }

        if cached_pc == self.regs[pc] {
            self.regs[pc] += 4;
        }
    }

    fn branch(&mut self, inst: u32) {
        let l = inst.bit(24);
        let signed_immed = inst.bits(0..24);
        self.b(l, signed_immed);
    }

    fn branch_and_exchange(&mut self, inst: u32) {
        let rn = Register(inst.bits(0..4));
        self.bx(rn);
    }

    fn coprocessor(&mut self, inst: u32) {
        if inst.bit(25) {
            if inst.bit(20) {
                if inst.bit(4) {
                    self.mrc();
                } else {
                    let opcode_1 = inst.bits(20..24);
                    let crn = inst.bits(16..20);
                    let crd = inst.bits(12..16);
                    let coprocessor = inst.bits(8..12);
                    let opcode_2 = inst.bits(5..8);
                    let crm = inst.bits(0..4);

                    self.cdp(coprocessor, opcode_1, crd, crn, crm, opcode_2)
                }
            } else {
                self.mcr();
            }
        } else {
            if inst.bit(20) {
                self.ldc();
            } else {
                self.stc();
            }
        }
    }

    fn data_processing(&mut self, inst: u32) {
        let i = inst.bit(25);
        let opcode = inst.bits(21..25);
        let s = inst.bit(20);
        let rn = Register(inst.bits(16..20));
        let rd = Register(inst.bits(12..16));
        let operand2 = self.addr_mode_1(i, inst.bits(0..12));

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

    fn load_and_store_halfword_or_signed_byte(&mut self, inst: u32) {
        if inst.bits(25..28) == 0
        && inst.bit(7)
        && inst.bit(4) {
            let p = inst.bit(24);
            let u = inst.bit(23);
            let i = inst.bit(22);
            let w = inst.bit(21);
            let l = inst.bit(20);
            let rn = Register(inst.bits(16..20));
            let rd = Register(inst.bits(12..16));
            let offset_a = inst.bits(8..12);
            let offset_b = inst.bits(0..4);
            let address = self.addr_mode_3(p, u, i, w, rn, offset_a, offset_b);

            match inst.bits(4..8) {
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

    fn load_and_store_multiple(&mut self, inst: u32) {
        match (inst.bit(20), inst.bit(22), inst.bit(15)) {
            (true,  true, true)  => self.ldm3(),
            (true,  true, false) => self.ldm2(),
            (true,  false, _)    => self.ldm1(),
            (false, true,  _)    => self.stm2(),
            (false, false, _)    => self.stm1(),
        }
    }

    fn load_and_store_word_or_unsigned_byte_insts(&mut self, inst: u32) {
        let i = inst.bit(25);
        let p = inst.bit(24);
        let u = inst.bit(23);
        let b = inst.bit(22);
        let w = inst.bit(21);
        let l = inst.bit(20);
        let rn = Register(inst.bits(16..20));
        let rd = Register(inst.bits(12..16));
        let offset = inst.bits(0..12);
        let address = self.addr_mode_2(i, p, u, w, rn, offset);

        let t = !p && w;
        if      l  && t  && b  { self.ldrbt(rd, address); }
        else if l  && t  && !b { self.ldrt(rd, address); }
        else if l  && !t && b  { self.ldrb(rd, address); }
        else if l  && !t && !b { self.ldr(rd, address); }
        else if !l && t  && b  { self.strbt(address, rd); }
        else if !l && t  && !b { self.strt(address, rd); }
        else if !l && !t && b  { self.strb(address, rd); }
        else if !l && !t && !b { self.str(address, rd); }
        else { unreachable!(); }
    }

    fn multiply(&mut self, inst: u32) {
        let long = inst.bit(23);
        let s = inst.bit(20);
        let rd = Register(inst.bits(16..20)); // rd_hi (if long)
        let rn = Register(inst.bits(12..16)); // rd_lo (if long)
        let rs = Register(inst.bits(8..12));
        let rm = Register(inst.bits(0..4));

        if long {
            match inst.bits(21..23) {
                0b00 => { self.umull(s, rd, rn, rm, rs) },
                0b01 => { self.umlal(s, rd, rn, rm, rs) },
                0b10 => { self.smull(s, rd, rn, rm, rs) },
                0b11 => { self.smlal(s, rd, rn, rm, rs) },
                _    => { unreachable!() }
            }
        } else {
            if inst.bit(21) {
                self.mla(s, rd, rm, rs, rn);
            } else {
                self.mul(s, rd, rm, rs);
            }
        }
    }

    fn semaphore(&mut self, inst: u32) {
        let b = inst.bit(22);
        let rn = Register(inst.bits(16..20));
        let rd = Register(inst.bits(12..16));
        let rm = Register(inst.bits(0..4));

        if b {
            self.swpb(rd, rm, rn);
        } else {
            self.swp(rd, rm, rn);
        }
    }

    fn software_interrupt(&mut self, inst: u32) {
        let immediate = inst.bits(0..24);
        self.swi(immediate);
    }

    fn status_register(&mut self, inst: u32) {
        let i = inst.bit(25);
        let r = inst.bit(22);
        let f = inst.bit(19);
        let s = inst.bit(18);
        let x = inst.bit(17);
        let c = inst.bit(16);
        let rd = Register(inst.bits(12..16));
        let operand = self.addr_mode_1(i, inst.bits(0..12));

        if inst.bit(21) {
            self.msr(c, x, s, f, r, operand.0);
        } else {
            self.mrs(r, rd);
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

    // Program status register modes

    fn in_a_priviledged_mode(&self) -> bool {
        // Not user mode
        self.cpsr.bits(0..5) != 0b10000
    }

    fn current_mode_has_spsr(&self) -> bool {
        // Not User or System mode
        let mode = self.cpsr.bits(0..5);
        mode != 0b10000 && mode != 0b11111
    }

    // Instructions

    fn adc(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: adc");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.regs[rn];
        let c_flag = if self.c() { 1 } else { 0 };
        let result_long = rn_val as u64 + shifter_operand as u64 + c_flag as u64;
        let result = result_long as u32;
        self.regs[rd] = result;

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
        let rn_val = self.regs[rn];
        let result_long = rn_val as u64 + shifter_operand as u64;
        let result = result_long as u32;
        self.regs[rd] = result;

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
        let result = self.regs[rn] & shifter_operand;
        self.regs[rd] = result;

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
        if l {
            let pc_val = self.regs[Register(15)];
            self.regs[Register(14)] = pc_val + 4;
        }

        let sign_extended = (((signed_immed as i32) << 8) >> 8) as u32;
        let target = (sign_extended << 2) + 8;
        self.regs[Register(15)] += target;
    }

    fn bic(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: bic");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.regs[rn];
        let result = rn_val & !shifter_operand;
        self.regs[rd] = result;

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
        let rn_val = self.regs[rn];
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
        let rn_val = self.regs[rn];
        let result = rn_val - shifter_operand;
        self.set_n(result.bit(31));
        self.set_z(result == 0);
        self.set_c(!borrow_from(rn_val, shifter_operand));
        self.set_z(overflow_from_sub(rn_val, shifter_operand, result));
    }

    fn eor(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: eor");
        let (shifter_operand, shifter_carry_out) = operand2;
        let result = self.regs[rn] | shifter_operand;
        self.regs[rd] = result;

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

    fn ldr(&mut self, rd: Register, address: u32) {
        println!("Instruction: ldr");
        let value = self.memory.read_word(address);
        let rotation = address.bits(0..2);
        let result = value.rotate_right(8 * rotation);

        self.regs[rd] = if rd == Register(15) {
            result & 0xFFFFFFFC
        } else {
            result
        }
    }

    fn ldrb(&mut self, rd: Register, address: u32) {
        println!("Instruction: ldrb");
        self.regs[rd] = self.memory.read_byte(address);
    }

    fn ldrbt(&mut self, rd: Register, address: u32) {
        println!("Instruction: ldrbt");
        // TODO: signal memory system to act as if CPU is in user mode
        self.regs[rd] = self.memory.read_byte(address);
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

    fn ldrt(&mut self, rd: Register, address: u32) {
        println!("Instruction: ldrt");
        // TODO: signal memory system to act as if CPU is in user mode
        let value = self.memory.read_word(address);
        let rotation = address.bits(0..2);
        self.regs[rd] = value.rotate_right(8 * rotation);
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
        self.regs[rd] = shifter_operand;

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

    fn mrs(&mut self, r: bool, rd: Register) {
        println!("Instruction: mrs");
        self.regs[rd] = if r { self.spsr } else { self.cpsr };
    }

    fn msr(&mut self, c: bool, x: bool, s: bool, f: bool, r: bool, operand: u32) {
        println!("Instruction: msr");
        if r {
            if !self.current_mode_has_spsr() { return; }
            if c { self.spsr.set_bits(0..8,   operand.bits(0..8)); }
            if x { self.spsr.set_bits(8..16,  operand.bits(8..16)); }
            if s { self.spsr.set_bits(16..24, operand.bits(16..24)); }
            if f { self.spsr.set_bits(24..32, operand.bits(24..32)); }
        } else {
            let priviledged = self.in_a_priviledged_mode();
            if c && priviledged { self.cpsr.set_bits(0..8,   operand.bits(0..8)); }
            if x && priviledged { self.cpsr.set_bits(8..16,  operand.bits(8..16)); }
            if s && priviledged { self.cpsr.set_bits(16..24, operand.bits(16..24)); }
            if f                { self.cpsr.set_bits(24..32, operand.bits(24..32)); }
        }
    }

    fn mul(&mut self, s: bool, rd: Register, rm: Register, rs: Register) {
        println!("Instruction: mul");
        unimplemented!();
    }

    fn mvn(&mut self, s: bool, rd: Register, operand2: (u32, bool)) {
        println!("Instruction: mvn");
        let (shifter_operand, shifter_carry_out) = operand2;
        let result = !shifter_operand;
        self.regs[rd] = result;

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
        let rn_val = self.regs[rn];
        let result = rn_val | shifter_operand;
        self.regs[rd] = result;

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
        let rn_val = self.regs[rn];
        let result = shifter_operand - rn_val;
        self.regs[rd] = result;

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
        let rn_val = self.regs[rn];
        let not_c_flag = if self.c() { 0 } else { 1 };
        let result = shifter_operand - rn_val - not_c_flag;
        self.regs[rd] = result;

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
        let rn_val = self.regs[rn];
        let not_c_flag = if self.c() { 0 } else { 1 };
        let result = rn_val - shifter_operand - not_c_flag;
        self.regs[rd] = result;

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

    fn str(&mut self, address: u32, rd: Register) {
        println!("Instruction: str");
        self.memory.write_word(address, self.regs[rd]);
    }

    fn strb(&mut self, address: u32, rd: Register) {
        println!("Instruction: strb");
        self.memory.write_byte(address, self.regs[rd]);
    }

    fn strbt(&mut self, address: u32, rd: Register) {
        println!("Instruction: strbt");
        // TODO: signal memory system to act as if CPU is in user mode
        self.memory.write_byte(address, self.regs[rd]);
    }

    fn strh(&mut self, rd: Register, address: u32) {
        println!("Instruction: strh");
        unimplemented!();
    }

    fn strt(&mut self, address: u32, rd: Register) {
        println!("Instruction: strt");
        // TODO: signal memory system to act as if CPU is in user mode
        self.memory.write_word(address, self.regs[rd]);
    }

    fn sub(&mut self, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: sub");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.regs[rn];
        let result = rn_val - shifter_operand;
        self.regs[rd] = result;

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
        let rn_val = self.regs[rn];
        let result = rn_val | shifter_operand;
        self.set_n(result.bit(31));
        self.set_z(result == 0);
        self.set_c(shifter_carry_out);
    }

    fn tst(&mut self, s: bool, rn: Register, operand2: (u32, bool)) {
        println!("Instruction: tst");
        let (shifter_operand, shifter_carry_out) = operand2;
        let rn_val = self.regs[rn];
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
    fn addr_mode_1(&self, i: bool, operand: u32) -> (u32, bool) {
        let shifter_operand: u32;
        let shifter_carry_out: bool;

        // 32-bit immediate
        if i {
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
            let rm_val = self.regs[rm];

            shifter_operand = rm_val;
            shifter_carry_out = self.c();

        // Register shift
        } else if operand.bit(4) {
            let rs = Register(operand.bits(8..12));
            let shift = operand.bits(5..7);
            let rm = Register(operand.bits(0..4));
            let rs_val = self.regs[rs];
            let rm_val = self.regs[rm];

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
            let rm_val = self.regs[rm];

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
            let rm_val = self.regs[rm];

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

            self.regs[rm]
        } else {
            offset_12
        };

        let rn_val = self.regs[rn];
        let value = if u { rn_val + offset } else { rn_val - offset };
        let address = if p { value } else { rn_val };
        if !p || w { self.regs[rn] = value };

        address
    }

    fn addr_mode_3(&mut self, p: bool, u: bool, i: bool, w: bool, rn: Register, offset_a: u32, offset_b: u32) -> u32 {
        if !p && w { panic!("unpredictable"); }

        let offset = if i {
            (offset_a << 4) | offset_b
        } else {
            self.regs[Register(offset_b)] // rm
        };
        let rn_val = self.regs[rn];
        let value = if u { rn_val + offset } else { rn_val - offset };
        let address = if p { value } else { rn_val };
        if !p || w { self.regs[rn] = value };

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
struct Memory(Box<[u8; 268_435_456]>);

impl Memory {
    fn new() -> Memory {
        Memory(box [0; 268_435_456])
    }

    fn read_byte(&self, address: u32) -> u32 {
        self.0[address as usize] as u32
    }

    fn read_halfword(&self, address: u32) -> u32 {
        (self.0[address as usize] as u32)
        + ((self.0[(address + 1) as usize] as u32) << 8)
    }

    fn read_word(&self, address: u32) -> u32 {
        (self.0[address as usize] as u32)
        + ((self.0[(address + 1) as usize] as u32) << 8)
        + ((self.0[(address + 2) as usize] as u32) << 16)
        + ((self.0[(address + 3) as usize] as u32) << 24)
    }

    fn write_byte(&mut self, address: u32, value: u32) {
        self.0[address as usize] = value as u8;
    }

    fn write_halfword(&mut self, address: u32, value: u32) {
        self.0[address as usize] = value as u8;
        self.0[(address + 1) as usize] = (value >> 8) as u8;
    }

    fn write_word(&mut self, address: u32, value: u32) {
        self.0[address as usize] = value as u8;
        self.0[(address + 1) as usize] = (value >> 8) as u8;
        self.0[(address + 2) as usize] = (value >> 16) as u8;
        self.0[(address + 3) as usize] = (value >> 24) as u8;
    }
}
