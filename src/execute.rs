use bit::{Bit, Bits, SetBit, SetBits};
use cpu::{Cpu, Register};
use instruction::{
    Condition,
    Instruction,
    Operation,
    AddressingMode1,
    AddressingMode2,
    AddressingMode3
};

pub fn execute(cpu: &mut Cpu, inst: Instruction) {
    let pc = Register(15);
    let cached_pc = cpu.regs[pc];

    if condition_passed(cpu, inst.condition) {
        match inst.operation {
            Operation::Branch { l, signed_immed } => {
                b(cpu, l, signed_immed);
            },

            Operation::BranchAndExchange { rm } => {
                bx(cpu, rm);
            },

            Operation::Coprocessor { operation, coprocessor, opcode1, opcode2, crd, crn, crm } => {
                use instruction::CoprocessorOperation::*;
                match operation {
                    Cdp => { cdp(cpu, coprocessor, opcode1, opcode2, crd, crn, crm) },
                    Ldc => { ldc(cpu) },
                    Mcr => { mcr(cpu) },
                    Mrc => { mrc(cpu) },
                    Stc => { stc(cpu) },
                };
            },

            Operation::DataProcessing { operation, s, rn, rd, address } => {
                let operand2 = addr_mode_1(cpu, address);
                use instruction::DataProcessingOperation::*;
                match operation {
                    And => { and(cpu, s, rd, rn, operand2) },
                    Eor => { eor(cpu, s, rd, rn, operand2) },
                    Sub => { sub(cpu, s, rd, rn, operand2) },
                    Rsb => { rsb(cpu, s, rd, rn, operand2) },
                    Add => { add(cpu, s, rd, rn, operand2) },
                    Adc => { adc(cpu, s, rd, rn, operand2) },
                    Sbc => { sbc(cpu, s, rd, rn, operand2) },
                    Rsc => { rsc(cpu, s, rd, rn, operand2) },
                    Tst => { tst(cpu, s,     rn, operand2) },
                    Teq => { teq(cpu, s,     rn, operand2) },
                    Cmp => { cmp(cpu, s,     rn, operand2) },
                    Cmn => { cmn(cpu, s,     rn, operand2) },
                    Orr => { orr(cpu, s, rd, rn, operand2) },
                    Mov => { mov(cpu, s, rd,     operand2) },
                    Bic => { bic(cpu, s, rd, rn, operand2) },
                    Mvn => { mvn(cpu, s, rd,     operand2) },
                };
            },

            Operation::LoadAndStoreHalfwordOrSignedByte { operation, rd, address } => {
                let address = addr_mode_3(cpu, address);

                use instruction::LoadAndStoreHalfwordOrSignedByteOperation::*;
                match operation {
                    Ldrh  => { ldrh(cpu, rd, address) },
                    Ldrsb => { ldrsb(cpu, rd, address) },
                    Ldrsh => { ldrsh(cpu, rd, address) },
                    Strh  => { strh(cpu, rd, address) },
                };
            },

            Operation::LoadAndStoreMultiple { operation } => {
                use instruction::LoadAndStoreMultipleOperation::*;
                match operation {
                    Ldm1 => { ldm1(cpu) },
                    Ldm2 => { ldm2(cpu) },
                    Ldm3 => { ldm3(cpu) },
                    Stm1 => { stm1(cpu) },
                    Stm2 => { stm2(cpu) },
                };
            },

            Operation::LoadAndStoreWordOrUnsignedByte { operation, rd, address } => {
                let address = addr_mode_2(cpu, address);

                use instruction::LoadAndStoreWordOrUnsignedByteOperation::*;
                match operation {
                    Ldrbt => { ldrbt(cpu, rd, address) }
                    Ldrt  => { ldrt(cpu, rd, address) }
                    Ldrb  => { ldrb(cpu, rd, address) }
                    Ldr   => { ldr(cpu, rd, address) }
                    Strbt => { strbt(cpu, address, rd) }
                    Strt  => { strt(cpu, address, rd) }
                    Strb  => { strb(cpu, address, rd) }
                    Str   => { str(cpu, address, rd) }
                };
            },

            Operation::Multiply { operation, s, rd, rn, rm, rs } => {
                use instruction::MultiplyOperation::*;
                match operation {
                    Mul   => { mul(  cpu, s, rd,     rm, rs) },
                    Mla   => { mla(  cpu, s, rd, rn, rm, rs) },
                    Umull => { umull(cpu, s, rd, rn, rm, rs) },
                    Umlal => { umlal(cpu, s, rd, rn, rm, rs) },
                    Smull => { smull(cpu, s, rd, rn, rm, rs) },
                    Smlal => { smlal(cpu, s, rd, rn, rm, rs) },
                };
            },

            Operation::Semaphore { b, rd, rm, rn } => {
                if b {
                    swpb(cpu, rd, rm, rn);
                } else {
                    swp(cpu, rd, rm, rn);
                }
            },

            Operation::SoftwareInterrupt { immediate } => {
                swi(cpu, immediate);
            },

            Operation::StatusRegister { operation, r, f, s, x, c, rd, address } => {
                let address = addr_mode_1(cpu, address);
                use instruction::StatusRegisterOperation::*;
                match operation {
                    Msr => { msr(cpu, c, x, s, f, r, address.0) },
                    Mrs => { mrs(cpu, r, rd) },
                };
            },
        }
    }

    if cached_pc == cpu.regs[pc] {
        cpu.regs[pc] += 4;
    }
}

fn condition_passed(cpu: &Cpu, condition: Condition) -> bool {
    let z = cpu.cpsr.z();
    let c = cpu.cpsr.c();
    let n = cpu.cpsr.n();
    let v = cpu.cpsr.v();

    match condition {
        Condition::Eq => {  z },
        Condition::Ne => { !z },
        Condition::Cs => {  c },
        Condition::Cc => { !c },
        Condition::Mi => {  n },
        Condition::Pl => { !n },
        Condition::Vs => {  v },
        Condition::Vc => { !v },
        Condition::Hi => {  c && !z },
        Condition::Ls => { !c ||  z },
        Condition::Ge => {  n ==  v },
        Condition::Lt => {  n !=  v },
        Condition::Gt => { !z &&  n == v },
        Condition::Le => {  z ||  n != v },
        Condition::Al => { true },
        Condition::Nv => { panic!("unpredictable") },
    }
}

// Instructions

fn adc(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: adc");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let c_flag = if cpu.cpsr.c() { 1 } else { 0 };
    let result_long = rn_val as u64 + shifter_operand as u64 + c_flag as u64;
    let result = result_long as u32;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(carry_from(result_long));
        cpu.cpsr.set_v(overflow_from_add(rn_val, shifter_operand, result));
    }
}

fn add(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: add");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result_long = rn_val as u64 + shifter_operand as u64;
    let result = result_long as u32;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(carry_from(result_long));
        cpu.cpsr.set_v(overflow_from_add(rn_val, shifter_operand, result));
    }
}

fn and(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: and");
    let (shifter_operand, shifter_carry_out) = operand2;
    let result = cpu.regs[rn] & shifter_operand;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(shifter_carry_out);
    }
}

fn b(cpu: &mut Cpu, l: bool, signed_immed: u32) {
    println!("Instruction: b");
    if l {
        let pc_val = cpu.regs[Register(15)];
        cpu.regs[Register(14)] = pc_val + 4;
    }

    let sign_extended = (((signed_immed as i32) << 8) >> 8) as u32;
    let target = (sign_extended << 2) + 8;
    cpu.regs[Register(15)] += target;
}

fn bic(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: bic");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result = rn_val & !shifter_operand;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(shifter_carry_out);
    }
}

fn bx(cpu: &mut Cpu, rm: Register) {
    println!("Instruction: bx");
    let rm_val = cpu.regs[rm];
    cpu.cpsr.set_t(rm_val.bit(0));
    cpu.regs[Register(15)] = rm_val & 0xFFFFFFFE;
}


fn cdp(cpu: &mut Cpu, coprocessor: u32, opcode1: u32, opcode2: u32, crd: u32, crn: u32, crm: u32) {
    println!("Instruction: cdp");
    unimplemented!();
}

fn cmn(cpu: &mut Cpu, s: bool, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: cmn");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result_long = rn_val as u64 + shifter_operand as u64;
    let result = result_long as u32;

    cpu.cpsr.set_n(result.bit(31));
    cpu.cpsr.set_z(result == 0);
    cpu.cpsr.set_c(carry_from(result_long));
    cpu.cpsr.set_z(overflow_from_add(rn_val, shifter_operand, result));
}

fn cmp(cpu: &mut Cpu, s: bool, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: cmp");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result = rn_val - shifter_operand;
    cpu.cpsr.set_n(result.bit(31));
    cpu.cpsr.set_z(result == 0);
    cpu.cpsr.set_c(!borrow_from(rn_val, shifter_operand));
    cpu.cpsr.set_z(overflow_from_sub(rn_val, shifter_operand, result));
}

fn eor(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: eor");
    let (shifter_operand, shifter_carry_out) = operand2;
    let result = cpu.regs[rn] | shifter_operand;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(shifter_carry_out);
    }
}

fn ldc(cpu: &mut Cpu) {
    println!("Instruction: ldc");
    unimplemented!();
}

fn ldm1(cpu: &mut Cpu) {
    println!("Instruction: ldm1");
    unimplemented!();
}

fn ldm2(cpu: &mut Cpu) {
    println!("Instruction: ldm2");
    unimplemented!();
}

fn ldm3(cpu: &mut Cpu) {
    println!("Instruction: ldm3");
    unimplemented!();
}

fn ldr(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: ldr");
    let value = cpu.memory.read_word(address);
    let rotation = address.bits(0..2);
    let result = value.rotate_right(8 * rotation);

    cpu.regs[rd] = if rd == Register(15) {
        result & 0xFFFFFFFC
    } else {
        result
    }
}

fn ldrb(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: ldrb");
    cpu.regs[rd] = cpu.memory.read_byte(address);
}

fn ldrbt(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: ldrbt");
    // TODO: signal memory system to act as if CPU is in user mode
    cpu.regs[rd] = cpu.memory.read_byte(address);
}

fn ldrh(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: ldrh");
    unimplemented!();
}

fn ldrsb(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: ldrsb");
    unimplemented!();
}

fn ldrsh(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: ldrsh");
    unimplemented!();
}

fn ldrt(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: ldrt");
    // TODO: signal memory system to act as if CPU is in user mode
    let value = cpu.memory.read_word(address);
    let rotation = address.bits(0..2);
    cpu.regs[rd] = value.rotate_right(8 * rotation);
}

fn mcr(cpu: &mut Cpu) {
    println!("Instruction: mcr");
    unimplemented!();
}

fn mla(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, rm: Register, rs: Register) {
    println!("Instruction: mla");
    unimplemented!();
}

fn mov(cpu: &mut Cpu, s: bool, rd: Register, operand2: (u32, bool)) {
    println!("Instruction: mov");
    let (shifter_operand, shifter_carry_out) = operand2;
    cpu.regs[rd] = shifter_operand;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(shifter_operand.bit(31));
        cpu.cpsr.set_z(shifter_operand == 0);
        cpu.cpsr.set_c(shifter_carry_out);
    }
}

fn mrc(cpu: &mut Cpu) {
    println!("Instruction: mrc");
    unimplemented!();
}

fn mrs(cpu: &mut Cpu, r: bool, rd: Register) {
    println!("Instruction: mrs");
    cpu.regs[rd] = if r { cpu.spsr.to_bits() } else { cpu.cpsr.to_bits() };
}

fn msr(cpu: &mut Cpu, c: bool, x: bool, s: bool, f: bool, r: bool, operand: u32) {
    println!("Instruction: msr");
    if r {
        if !cpu.cpsr.has_spsr() { return; }
        if c { cpu.spsr.set_bits(0..8,   operand.bits(0..8)); }
        if x { cpu.spsr.set_bits(8..16,  operand.bits(8..16)); }
        if s { cpu.spsr.set_bits(16..24, operand.bits(16..24)); }
        if f { cpu.spsr.set_bits(24..32, operand.bits(24..32)); }
    } else {
        let priviledged = cpu.cpsr.is_priviledged();
        if c && priviledged { cpu.cpsr.set_bits(0..8,   operand.bits(0..8)); }
        if x && priviledged { cpu.cpsr.set_bits(8..16,  operand.bits(8..16)); }
        if s && priviledged { cpu.cpsr.set_bits(16..24, operand.bits(16..24)); }
        if f                { cpu.cpsr.set_bits(24..32, operand.bits(24..32)); }
    }
}

fn mul(cpu: &mut Cpu, s: bool, rd: Register, rm: Register, rs: Register) {
    println!("Instruction: mul");
    unimplemented!();
}

fn mvn(cpu: &mut Cpu, s: bool, rd: Register, operand2: (u32, bool)) {
    println!("Instruction: mvn");
    let (shifter_operand, shifter_carry_out) = operand2;
    let result = !shifter_operand;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(shifter_carry_out);
    }
}

fn orr(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: orr");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result = rn_val | shifter_operand;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(shifter_carry_out);
    }
}

fn rsb(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: rsb");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result = shifter_operand - rn_val;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(!borrow_from(shifter_operand, rn_val));
        cpu.cpsr.set_v(overflow_from_sub(shifter_operand, rn_val, result));
    }
}

fn rsc(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: rsc");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let not_c_flag = if cpu.cpsr.c() { 0 } else { 1 };
    let result = shifter_operand - rn_val - not_c_flag;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(!borrow_from(shifter_operand, rn_val + not_c_flag));
        cpu.cpsr.set_v(overflow_from_sub(shifter_operand, rn_val + not_c_flag, result));
    }
}

fn sbc(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: sbc");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let not_c_flag = if cpu.cpsr.c() { 0 } else { 1 };
    let result = rn_val - shifter_operand - not_c_flag;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(!borrow_from(rn_val, shifter_operand + not_c_flag));
        cpu.cpsr.set_v(overflow_from_sub(rn_val, shifter_operand + not_c_flag, result));
    }
}

fn smlal(cpu: &mut Cpu, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
    println!("Instruction: smlal");
    unimplemented!();
}

fn smull(cpu: &mut Cpu, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
    println!("Instruction: smull");
    unimplemented!();
}

fn stc(cpu: &mut Cpu) {
    println!("Instruction: stc");
    unimplemented!();
}

fn stm1(cpu: &mut Cpu) {
    println!("Instruction: stm1");
    unimplemented!();
}

fn stm2(cpu: &mut Cpu) {
    println!("Instruction: stm2");
    unimplemented!();
}

fn str(cpu: &mut Cpu, address: u32, rd: Register) {
    println!("Instruction: str");
    cpu.memory.write_word(address, cpu.regs[rd]);
}

fn strb(cpu: &mut Cpu, address: u32, rd: Register) {
    println!("Instruction: strb");
    cpu.memory.write_byte(address, cpu.regs[rd]);
}

fn strbt(cpu: &mut Cpu, address: u32, rd: Register) {
    println!("Instruction: strbt");
    // TODO: signal memory system to act as if CPU is in user mode
    cpu.memory.write_byte(address, cpu.regs[rd]);
}

fn strh(cpu: &mut Cpu, rd: Register, address: u32) {
    println!("Instruction: strh");
    unimplemented!();
}

fn strt(cpu: &mut Cpu, address: u32, rd: Register) {
    println!("Instruction: strt");
    // TODO: signal memory system to act as if CPU is in user mode
    cpu.memory.write_word(address, cpu.regs[rd]);
}

fn sub(cpu: &mut Cpu, s: bool, rd: Register, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: sub");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result = rn_val - shifter_operand;
    cpu.regs[rd] = result;

    if s && rd == Register(15) {
        cpu.cpsr = cpu.spsr;
    } else if s {
        cpu.cpsr.set_n(result.bit(31));
        cpu.cpsr.set_z(result == 0);
        cpu.cpsr.set_c(!borrow_from(rn_val, shifter_operand));
        cpu.cpsr.set_v(overflow_from_sub(rn_val, shifter_operand, result));
    }
}

fn swi(cpu: &mut Cpu, immediate: u32) {
    println!("Instruction: swi");
    unimplemented!();
}

fn swp(cpu: &mut Cpu, rd: Register, rm: Register, rn: Register) {
    println!("Instruction: swp");
    unimplemented!();
}

fn swpb(cpu: &mut Cpu, rd: Register, rm: Register, rn: Register) {
    println!("Instruction: swpb");
    unimplemented!();
}

fn teq(cpu: &mut Cpu, s: bool, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: teq");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result = rn_val | shifter_operand;
    cpu.cpsr.set_n(result.bit(31));
    cpu.cpsr.set_z(result == 0);
    cpu.cpsr.set_c(shifter_carry_out);
}

fn tst(cpu: &mut Cpu, s: bool, rn: Register, operand2: (u32, bool)) {
    println!("Instruction: tst");
    let (shifter_operand, shifter_carry_out) = operand2;
    let rn_val = cpu.regs[rn];
    let result = rn_val & shifter_operand;
    cpu.cpsr.set_n(result.bit(31));
    cpu.cpsr.set_z(result == 0);
    cpu.cpsr.set_c(shifter_carry_out);
}

fn umlal(cpu: &mut Cpu, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
    println!("Instruction: umlal");
    unimplemented!();
}

fn umull(cpu: &mut Cpu, s: bool, rd_hi: Register, rd_lo: Register, rm: Register, rs: Register) {
    println!("Instruction: umull");
    unimplemented!();
}

// Addressing modes

// Returns (shifter_operand, shifter_carry_out)
fn addr_mode_1(cpu: &Cpu, address: AddressingMode1) -> (u32, bool) {
    let AddressingMode1 { i, operand } = address;
    let shifter_operand: u32;
    let shifter_carry_out: bool;

    // 32-bit immediate
    if i {
        let rotate_imm = operand.bits(8..12);
        let immed_8 = operand.bits(0..8);

        shifter_operand = immed_8.rotate_right(rotate_imm * 2);
        shifter_carry_out = if rotate_imm == 0 {
            cpu.cpsr.c()
        } else {
            shifter_operand.bit(31)
        };

    // Register
    } else if operand.bits(4..12) == 0 {
        let rm = Register(operand.bits(0..4));
        let rm_val = cpu.regs[rm];

        shifter_operand = rm_val;
        shifter_carry_out = cpu.cpsr.c();

    // Register shift
    } else if operand.bit(4) {
        let rs = Register(operand.bits(8..12));
        let shift = operand.bits(5..7);
        let rm = Register(operand.bits(0..4));
        let rs_val = cpu.regs[rs];
        let rm_val = cpu.regs[rm];

        match shift {
            // Logical shift left
            0b00 => {
                let part = rs_val.bits(0..8) as u8;
                if part == 0 {
                    shifter_operand = rm_val;
                    shifter_carry_out = cpu.cpsr.c();
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
                    shifter_carry_out = cpu.cpsr.c();
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
                    shifter_carry_out = cpu.cpsr.c();
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
                    shifter_carry_out = cpu.cpsr.c();
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
        let rm_val = cpu.regs[rm];

        match shift {
            // Logical shift left
            0b00 => {
                if shift_imm == 0 {
                    shifter_operand = rm_val;
                    shifter_carry_out = cpu.cpsr.c();
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
                    let c_flag = if cpu.cpsr.c() { 1 } else { 0 };
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

fn addr_mode_2(cpu: &mut Cpu, address: AddressingMode2) -> u32 {
    let AddressingMode2 { i, p, u, w, rn, offset } = address;

    let offset_val = if i {
        let shift_imm = offset.bits(7..12);
        let shift = offset.bits(5..7);
        let rm = Register(offset.bits(0..4));
        let rm_val = cpu.regs[rm];

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
                    (if cpu.cpsr.c() { 1 } else { 0 }) << 31 | rm_val >> 1
                } else { // Rotate right
                    rm_val.rotate_right(shift_imm)
                }
            },
            _ => unreachable!(),
        };

        cpu.regs[rm]
    } else {
        offset
    };

    let rn_val = cpu.regs[rn];
    let value = if u { rn_val + offset_val } else { rn_val - offset_val };
    let address = if p { value } else { rn_val };
    if !p || w { cpu.regs[rn] = value };

    address
}

fn addr_mode_3(cpu: &mut Cpu, address: AddressingMode3) -> u32 {
    let AddressingMode3 { p, u, i, w, rn, offset_a, offset_b } = address;
    if !p && w { panic!("unpredictable"); }

    let offset = if i {
        (offset_a << 4) | offset_b
    } else {
        cpu.regs[Register(offset_b)] // rm
    };
    let rn_val = cpu.regs[rn];
    let value = if u { rn_val + offset } else { rn_val - offset };
    let address = if p { value } else { rn_val };
    if !p || w { cpu.regs[rn] = value };

    address
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
