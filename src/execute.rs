use bit::{Bit, Bits, SetBit, SetBits};
use bus::{Read, Write};
use cpu::{Cpu, Register};
use instruction::{
    Condition,
    Instruction,
    AddressMode1,
    AddressMode2,
    AddressMode3
};

pub fn execute(cpu: &mut Cpu, inst: Instruction) {
    if condition_passed(cpu, inst.condition()) {
        println!("Executing {}", inst);
    } else {
        println!("Skipping {}", inst);
        return;
    }

    use self::Instruction::*;
    match inst {
        B { l, signed_immed, .. } => {
            if l {
                let pc_val = cpu.regs[Register(15)];
                cpu.regs[Register(14)] = pc_val + 4;
            }

            let sign_extended = (((signed_immed as i32) << 8) >> 8) as u32;
            let target = (sign_extended << 2) + 8;
            cpu.regs[Register(15)] += target;
        },

        Bx { rm, .. } => {
            let rm_val = cpu.regs[rm];
            cpu.cpsr.set_t(rm_val.bit(0));
            cpu.regs[Register(15)] = rm_val & 0xFFFFFFFE;
        },

        And { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let result = cpu.regs[rn] & shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == Register(15) {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Eor { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let result = cpu.regs[rn] | shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == Register(15) {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Sub { s, rd, rn, operand2, .. } => {
			let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Rsb { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Add { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Adc { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Sbc { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Rsc { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Tst { s, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val & shifter_operand;
            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(shifter_carry_out);
        },

        Teq { s, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val | shifter_operand;
            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(shifter_carry_out);
        },

        Cmp { s, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result = rn_val - shifter_operand;
            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(!borrow_from(rn_val, shifter_operand));
            cpu.cpsr.set_z(overflow_from_sub(rn_val, shifter_operand, result));
        },

        Cmn { s, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let rn_val = cpu.regs[rn];
            let result_long = rn_val as u64 + shifter_operand as u64;
            let result = result_long as u32;

            cpu.cpsr.set_n(result.bit(31));
            cpu.cpsr.set_z(result == 0);
            cpu.cpsr.set_c(carry_from(result_long));
            cpu.cpsr.set_z(overflow_from_add(rn_val, shifter_operand, result));
        },

        Orr { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Mov { s, rd, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            cpu.regs[rd] = shifter_operand;

            if s && rd == Register(15) {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(shifter_operand.bit(31));
                cpu.cpsr.set_z(shifter_operand == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Bic { s, rd, rn, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
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
        },

        Mvn { s, rd, operand2, .. } => {
            let (shifter_operand, shifter_carry_out) = addr_mode_1(cpu, operand2);
            let result = !shifter_operand;
            cpu.regs[rd] = result;

            if s && rd == Register(15) {
                cpu.cpsr = cpu.spsr;
            } else if s {
                cpu.cpsr.set_n(result.bit(31));
                cpu.cpsr.set_z(result == 0);
                cpu.cpsr.set_c(shifter_carry_out);
            }
        },

        Mul { s, rd, rm, rs, .. } => {
            unimplemented!();
        },

        Mla { s, rd, rn, rm, rs, .. } => {
            unimplemented!();
        },

        Umull { s, rd, rn, rm, rs, .. } => {
            unimplemented!();
        },

        Umlal { s, rd, rn, rm, rs, .. } => {
            unimplemented!();
        },

        Smull { s, rd, rn, rm, rs, .. } => {
            unimplemented!();
        },

        Smlal { s, rd, rn, rm, rs, .. } => {
            unimplemented!();
        },

        Msr { c, x, s, f, r, address, .. } => {
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

        Mrs { r, rd, .. } => {
            cpu.regs[rd] = if r { cpu.spsr.to_bits() } else { cpu.cpsr.to_bits() };
        },

        Ldrh { rd, address, .. } => {
            unimplemented!();
        },

        Ldrsb { rd, address, .. } => {
            unimplemented!();
        },

        Ldrsh { rd, address, .. } => {
            unimplemented!();
        },

        Strh { rd, address, .. } => {
            unimplemented!();
        },

        Ldrbt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            cpu.regs[rd] = cpu.memory.read_byte(address) as u32;
        },

        Ldrt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            let value = cpu.memory.read_word(address);
            let rotation = address.bits(0..2);
            cpu.regs[rd] = value.rotate_right(8 * rotation);
        },

        Ldrb { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            cpu.regs[rd] = cpu.memory.read_byte(address) as u32;
        },

        Ldr { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            let value = cpu.memory.read_word(address);
            let rotation = address.bits(0..2);
            let result = value.rotate_right(8 * rotation);

            cpu.regs[rd] = if rd == Register(15) {
                result & 0xFFFFFFFC
            } else {
                result
            }
        },

        Strbt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            cpu.memory.write_byte(address, cpu.regs[rd] as u8);
        },

        Strt { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            // TODO: signal memory system to act as if CPU is in user mode
            cpu.memory.write_word(address, cpu.regs[rd]);
        },

        Strb { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            cpu.memory.write_byte(address, cpu.regs[rd] as u8);
        },

        Str { rd, address, .. } => {
            let address = addr_mode_2(cpu, address);
            cpu.memory.write_word(address, cpu.regs[rd]);
        },

        Ldm1 { .. } => {
            unimplemented!();
        },

        Ldm2 { .. } => {
            unimplemented!();
        },

        Ldm3 { .. } => {
            unimplemented!();
        },

        Stm1 { .. } => {
            unimplemented!();
        },

        Stm2 { .. } => {
            unimplemented!();
        },

        Swpb { rd, rm, rn, .. } => {
            unimplemented!();
        },

        Swp { rd, rm, rn, .. } => {
            unimplemented!();
        },

        Swi { immediate, .. } => {
            unimplemented!();
        },

        Cdp { .. } => {
            unimplemented!();
        },

        Ldc { .. } => {
            unimplemented!();
        },

        Mcr { .. } => {
            unimplemented!();
        },

        Mrc { .. } => {
            unimplemented!();
        },

        Stc { .. } => {
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
    let AddressMode1 { i, operand } = address;
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

fn addr_mode_2(cpu: &mut Cpu, address: AddressMode2) -> u32 {
    let AddressMode2 { i, p, u, w, rn, offset } = address;

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

fn addr_mode_3(cpu: &mut Cpu, address: AddressMode3) -> u32 {
    let AddressMode3 { p, u, i, w, rn, offset_a, offset_b } = address;
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
