use bit::{Bit, Bits, SetBit, SetBits};
use bus::{Read, Write};
use cpu::{Cpu, LR, PC};
use instruction::{
    Condition,
    Instruction,
    AddressMode1,
    AddressMode2,
    AddressMode3,
    AddressingOffset,
    AddressingMode,
    ShiftDirection,
};

pub fn execute(cpu: &mut Cpu, inst: Instruction) {
    if condition_passed(cpu, inst.condition()) {
        println!("Executing {}", inst);
    } else {
        println!("Skipping {}", inst);
        return;
    }

    match inst {
        Instruction::B { l, signed_immed, .. } => {
            if l {
                let pc_val = cpu.regs[PC];
                cpu.regs[LR] = pc_val + 4;
            }

            cpu.regs[PC] += sign_extend(signed_immed, 24) << 2;
        },

        Instruction::Bx { rm, .. } => {
            let rm_val = cpu.regs[rm];
            cpu.cpsr.set_t(rm_val.bit(0));
            cpu.regs[PC] = rm_val & 0xFFFFFFFE;
        },

        Instruction::And { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let result = cpu.regs[rn] & shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Instruction::Eor { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let result = cpu.regs[rn] | shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Instruction::Sub { s, rd, rn, operand2, .. } => {
			let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
			let rn_val = cpu.regs[rn];
			let result = rn_val - shifter_operand;
			cpu.regs[rd] = result;

			if s && rd == PC {
				cpu.cpsr = cpu.spsr;
			} else if s {
				cpu.cpsr.set_n(result.bit(31));
				cpu.cpsr.set_z(result == 0);
				cpu.cpsr.set_c(!borrow_from(rn_val, shifter_operand));
				cpu.cpsr.set_v(overflow_from_sub(rn_val, shifter_operand, result));
			}
        },

        Instruction::Rsb { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = shifter_operand - rn_val;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(!borrow_from(shifter_operand, rn_val));
                cpu.cpsr.set_v(overflow_from_sub(shifter_operand, rn_val, result));
            }
        },

        Instruction::Add { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result_long = rn_val as u64 + shifter_operand as u64;
            let result = result_long as u32;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(carry_from(result_long));
                cpu.cpsr.set_v(overflow_from_add(rn_val, shifter_operand, result));
            }
        },

        Instruction::Adc { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let c_flag = if cpu.cpsr.c() { 1 } else { 0 };
            let result_long = rn_val as u64 + shifter_operand as u64 + c_flag as u64;
            let result = result_long as u32;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(carry_from(result_long));
                cpu.cpsr.set_v(overflow_from_add(rn_val, shifter_operand, result));
            }
        },

        Instruction::Sbc { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let not_c_flag = if cpu.cpsr.c() { 0 } else { 1 };
            let result = rn_val - shifter_operand - not_c_flag;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(!borrow_from(rn_val, shifter_operand + not_c_flag));
                cpu.cpsr.set_v(overflow_from_sub(rn_val, shifter_operand + not_c_flag, result));
            }
        },

        Instruction::Rsc { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let not_c_flag = if cpu.cpsr.c() { 0 } else { 1 };
            let result = shifter_operand - rn_val - not_c_flag;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(!borrow_from(shifter_operand, rn_val + not_c_flag));
                cpu.cpsr.set_v(overflow_from_sub(shifter_operand, rn_val + not_c_flag, result));
            }
        },

        Instruction::Tst { rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val & shifter_operand;
            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(shifter_carry_out);
        },

        Instruction::Teq { rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val | shifter_operand;
            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(shifter_carry_out);
        },

        Instruction::Cmp { rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val - shifter_operand;
            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(!borrow_from(rn_val, shifter_operand));
            cpu.cpsr.set_z(overflow_from_sub(rn_val, shifter_operand, result));
        },

        Instruction::Cmn { rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result_long = rn_val as u64 + shifter_operand as u64;
            let result = result_long as u32;

            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(carry_from(result_long));
            cpu.cpsr.set_z(overflow_from_add(rn_val, shifter_operand, result));
        },

        Instruction::Orr { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val | shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Instruction::Mov { s, rd, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            cpu.regs[rd] = shifter_operand;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(shifter_operand.bit(31));
                cpu.cpsr.set_z(shifter_operand == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Instruction::Bic { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val & !shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Instruction::Mvn { s, rd, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let result = !shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == PC {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Instruction::Mul { s, rd, rm, rs, .. } => {
            unimplemented!();
        },

        Instruction::Mla { s, rd, rn, rm, rs, .. } => {
            unimplemented!();
        },

        Instruction::Umull { s, rd_hi, rd_lo, rm, rs, .. } => {
            unimplemented!();
        },

        Instruction::Umlal { s, rd_hi, rd_lo, rm, rs, .. } => {
            unimplemented!();
        },

        Instruction::Smull { s, rd_hi, rd_lo, rm, rs, .. } => {
            unimplemented!();
        },

        Instruction::Smlal { s, rd_hi, rd_lo, rm, rs, .. } => {
            unimplemented!();
        },

        Instruction::Msr { c, x, s, f, r, address, .. } => {
            let (operand, _) = addr_mode_1(cpu, address);
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
        },

        Instruction::Mrs { r, rd, .. } => {
            cpu.regs[rd] = if r { cpu.spsr.to_bits() } else { cpu.cpsr.to_bits() };
        },

        Instruction::Ldrh { rd, address, .. } => {
            unimplemented!();
        },

        Instruction::Ldrsb { rd, address, .. } => {
            unimplemented!();
        },

        Instruction::Ldrsh { rd, address, .. } => {
            unimplemented!();
        },

        Instruction::Strh { rd, address, .. } => {
            unimplemented!();
        },

        Instruction::Ldrbt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            cpu.regs[rd] = cpu.memory.read_byte(address) as u32;
        },

        Instruction::Ldrt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            let value = cpu.memory.read_word(address);
            let rotation = address.bits(0..2);
            cpu.regs[rd] = value.rotate_right(8 * rotation);
        },

        Instruction::Ldrb { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            cpu.regs[rd] = cpu.memory.read_byte(address) as u32;
        },

        Instruction::Ldr { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            let value = cpu.memory.read_word(address);
            let rotation = address.bits(0..2);
            let result = value.rotate_right(8 * rotation);

            cpu.regs[rd] = if rd == PC {
                result & 0xFFFFFFFC
            } else {
                result
            }
        },

        Instruction::Strbt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            cpu.memory.write_byte(address, cpu.regs[rd] as u8);
        },

        Instruction::Strt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            cpu.memory.write_word(address, cpu.regs[rd]);
        },

        Instruction::Strb { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            cpu.memory.write_byte(address, cpu.regs[rd] as u8);
        },

        Instruction::Str { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            cpu.memory.write_word(address, cpu.regs[rd]);
        },

        Instruction::Ldm1 { .. } => {
            unimplemented!();
        },

        Instruction::Ldm2 { .. } => {
            unimplemented!();
        },

        Instruction::Ldm3 { .. } => {
            unimplemented!();
        },

        Instruction::Stm1 { .. } => {
            unimplemented!();
        },

        Instruction::Stm2 { .. } => {
            unimplemented!();
        },

        Instruction::Swpb { rd, rm, rn, .. } => {
            unimplemented!();
        },

        Instruction::Swp { rd, rm, rn, .. } => {
            unimplemented!();
        },

        Instruction::Swi { immediate, .. } => {
            unimplemented!();
        },

        Instruction::Cdp { .. } => {
            unimplemented!();
        },

        Instruction::Ldc { .. } => {
            unimplemented!();
        },

        Instruction::Mcr { .. } => {
            unimplemented!();
        },

        Instruction::Mrc { .. } => {
            unimplemented!();
        },

        Instruction::Stc { .. } => {
            unimplemented!();
        },
    };
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

// Address modes

// Returns (shifter_operand, shifter_carry_out)
fn addr_mode_1(cpu: &Cpu, address: AddressMode1) -> (u32, bool) {
    let shifter_operand: u32;
    let shifter_carry_out: bool;

    match address {
        AddressMode1::Immediate { value, rotate } => {
            shifter_operand = (value as u32).rotate_right((rotate as u32) * 2);
            shifter_carry_out = if rotate == 0 {
                cpu.cpsr.c()
            } else {
                shifter_operand.bit(31)
            };
        },
        AddressMode1::Shift { rm, shift, shift_imm } => {
            let rm_val = cpu.regs[rm];
            let shift_imm = match shift_imm {
                AddressingOffset::Immediate(value) =>  value ,
                AddressingOffset::Register(rs) =>  cpu.regs[rs].bits(0..8) as u16,
                AddressingOffset::ScaledRegister { .. } => unreachable!(),
            };

            match shift {
                ShiftDirection::Lsl => {
                    if shift_imm == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = cpu.cpsr.c();
                    } else if shift_imm < 32 {
                        shifter_operand = rm_val << shift_imm;
                        shifter_carry_out = rm_val.bit(32 - shift_imm as u8);
                    } else if shift_imm == 32 {
                        shifter_operand = 0;
                        shifter_carry_out = rm_val.bit(0);
                    } else /* shift_imm > 32 */ {
                        shifter_operand = 0;
                        shifter_carry_out = false;
                    }
                },
                ShiftDirection::Lsr => {
                    if shift_imm == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = cpu.cpsr.c();
                    } else if shift_imm < 32 {
                        shifter_operand = rm_val >> shift_imm;
                        shifter_carry_out = rm_val.bit(shift_imm as u8 - 1);
                    } else if shift_imm == 32 {
                        shifter_operand = 0;
                        shifter_carry_out = rm_val.bit(31);
                    } else /* shift_imm > 32 */ {
                        shifter_operand = 0;
                        shifter_carry_out = false;
                    }
                },
                ShiftDirection::Asr => {
                    if shift_imm == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = cpu.cpsr.c();
                    } else if shift_imm < 32 {
                        shifter_operand = (rm_val as i32 >> shift_imm) as u32;
                        shifter_carry_out = rm_val.bit(shift_imm as u8 - 1);
                    } else /* shift_imm >= 32 */ {
                        shifter_operand = if rm_val.bit(31) { 0xFFFFFFFF } else { 0 };
                        shifter_carry_out = rm_val.bit(31);
                    }
                },
                ShiftDirection::Ror => {
                    let shift_imm2 = shift_imm.bits(0..4);

                    if shift_imm == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = cpu.cpsr.c();
                    } else if shift_imm2 == 0 {
                        shifter_operand = rm_val;
                        shifter_carry_out = rm_val.bit(31);
                    } else /* shift_imm2 > 0 */ {
                        shifter_operand = rm_val.rotate_right(shift_imm2 as u32);
                        shifter_carry_out = rm_val.bit(shift_imm2 as u8 - 1);
                    }
                },
                ShiftDirection::Rrx => {
                    let c_flag = cpu.cpsr.c() as u32;
                    shifter_operand = (c_flag << 31) | (rm_val >> 1);
                    shifter_carry_out = rm_val.bit(0);
                },
            }
        },
    };

    (shifter_operand, shifter_carry_out)
}

fn addr_mode_2(cpu: &mut Cpu, address: AddressMode2) -> u32 {
    let AddressMode2 { rn, offset, addressing, u } = address;

    let offset_val = match offset {
        AddressingOffset::Immediate(offset) => offset as u32,
        AddressingOffset::Register(rm) => cpu.regs[rm],
        AddressingOffset::ScaledRegister { rm, shift, shift_imm } => {
            let rm_val = cpu.regs[rm];
            match shift {
                ShiftDirection::Lsl => {
                    rm_val << shift_imm
                },
                ShiftDirection::Lsr => {
                    if shift_imm == 0 { 0 } else { rm_val >> shift_imm }
                },
                ShiftDirection::Asr => {
                    if shift_imm == 0 {
                        if rm_val.bit(31) { 0xFFFFFFFF } else { 0 }
                    } else {
                        (rm_val as i32 >> shift_imm) as u32
                    }
                },
                ShiftDirection::Rrx => {
                    (if cpu.cpsr.c() { 1 } else { 0 }) << 31 | rm_val >> 1
                },
                ShiftDirection::Ror => {
                    rm_val.rotate_right(shift_imm as u32)
                },
            }
        },
    };

    let rn_val = cpu.regs[rn];
    let value = if u { rn_val + offset_val } else { rn_val - offset_val };

    match addressing {
        AddressingMode::Offset => {
            value
        },
        AddressingMode::PreIndexed => {
            cpu.regs[rn] = value;
            value
        },
        AddressingMode::PostIndexed => {
            cpu.regs[rn] = value;
            rn_val
        },
    }
}

fn addr_mode_3(cpu: &mut Cpu, address: AddressMode3) -> u32 {
    let AddressMode3 { rn, offset, addressing, u } = address;

    let offset_val = match offset {
        AddressingOffset::Immediate(byte) => byte as u32,
        AddressingOffset::Register(rm) => cpu.regs[rm],
        AddressingOffset::ScaledRegister { .. } => unreachable!(),
    };

    let rn_val = cpu.regs[rn];
    let value = if u { rn_val + offset_val } else { rn_val - offset_val };

    match addressing {
        AddressingMode::Offset => {
            value
        },
        AddressingMode::PreIndexed => {
            cpu.regs[rn] = value;
            value
        },
        AddressingMode::PostIndexed => {
            cpu.regs[rn] = value;
            rn_val
        },
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

fn sign_extend(operand: u32, size: u8) -> u32 {
    let shift = 32 - size;
    (((operand as i32) << shift) >> shift) as u32
}
