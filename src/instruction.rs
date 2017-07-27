use cpu::Register;
use std::fmt;

pub enum Instruction {
    // Branch
    B { condition: Condition, l: bool, signed_immed: u32 },
    Bx { condition: Condition, rm: Register },

    // Data processing
    And { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Eor { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Sub { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Rsb { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Add { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Adc { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Sbc { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Rsc { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Tst { condition: Condition, rn: Register, operand2: AddressMode1 },
    Teq { condition: Condition, rn: Register, operand2: AddressMode1 },
    Cmp { condition: Condition, rn: Register, operand2: AddressMode1 },
    Cmn { condition: Condition, rn: Register, operand2: AddressMode1 },
    Orr { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Mov { condition: Condition, s: bool, rd: Register, operand2: AddressMode1 },
    Bic { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Mvn { condition: Condition, s: bool, rd: Register, operand2: AddressMode1 },

    // Multiply
    Mul { condition: Condition, s: bool, rd: Register, rm: Register, rs: Register },
    Mla { condition: Condition, s: bool, rd: Register, rn: Register, rm: Register, rs: Register },
    Umull { condition: Condition, s: bool, rd_lo: Register, rd_hi: Register, rm: Register, rs: Register },
    Umlal { condition: Condition, s: bool, rd_lo: Register, rd_hi: Register, rm: Register, rs: Register },
    Smull { condition: Condition, s: bool, rd_lo: Register, rd_hi: Register, rm: Register, rs: Register },
    Smlal { condition: Condition, s: bool, rd_lo: Register, rd_hi: Register, rm: Register, rs: Register },

    // Status register access
    Msr { condition: Condition, c: bool, x: bool, s: bool, f: bool, r: bool, address: AddressMode1 },
    Mrs { condition: Condition, r: bool, rd: Register },

    // Load/store (halfword or signed byte)
    Ldrh { condition: Condition, rd: Register, address: AddressMode3 },
    Ldrsb { condition: Condition, rd: Register, address: AddressMode3 },
    Ldrsh { condition: Condition, rd: Register, address: AddressMode3 },
    Strh { condition: Condition, rd: Register, address: AddressMode3 },

    // Load/store (word or unsigned byte)
    Ldrbt { condition: Condition, rd: Register, address: AddressMode2 },
    Ldrt { condition: Condition, rd: Register, address: AddressMode2 },
    Ldrb { condition: Condition, rd: Register, address: AddressMode2 },
    Ldr { condition: Condition, rd: Register, address: AddressMode2 },
    Strbt { condition: Condition, rd: Register, address: AddressMode2 },
    Strt { condition: Condition, rd: Register, address: AddressMode2 },
    Strb { condition: Condition, rd: Register, address: AddressMode2 },
    Str { condition: Condition, rd: Register, address: AddressMode2 },

    // LoadAndStoreMultiple
    Ldm1 { condition: Condition }, // TODO
    Ldm2 { condition: Condition }, // TODO
    Ldm3 { condition: Condition }, // TODO
    Stm1 { condition: Condition }, // TODO
    Stm2 { condition: Condition }, // TODO

    // Semaphore
    Swpb { condition: Condition, rd: Register, rm: Register, rn: Register },
    Swp { condition: Condition, rd: Register, rm: Register, rn: Register },

    // ExceptionGenerating
    Swi { condition: Condition, immediate: u32 },

    // Coprocessor
    Cdp { condition: Condition }, // TODO
    Ldc { condition: Condition }, // TODO
    Mcr { condition: Condition }, // TODO
    Mrc { condition: Condition }, // TODO
    Stc { condition: Condition }, // TODO
}

impl Instruction {
    pub fn condition(&self) -> Condition {
        match *self {
            Instruction::B { condition, .. }
             | Instruction::Bx { condition, .. }
             | Instruction::And { condition, .. }
             | Instruction::Eor { condition, .. }
             | Instruction::Sub { condition, .. }
             | Instruction::Rsb { condition, .. }
             | Instruction::Add { condition, .. }
             | Instruction::Adc { condition, .. }
             | Instruction::Sbc { condition, .. }
             | Instruction::Rsc { condition, .. }
             | Instruction::Tst { condition, .. }
             | Instruction::Teq { condition, .. }
             | Instruction::Cmp { condition, .. }
             | Instruction::Cmn { condition, .. }
             | Instruction::Orr { condition, .. }
             | Instruction::Mov { condition, .. }
             | Instruction::Bic { condition, .. }
             | Instruction::Mvn { condition, .. }
             | Instruction::Mul { condition, .. }
             | Instruction::Mla { condition, .. }
             | Instruction::Umull { condition, .. }
             | Instruction::Umlal { condition, .. }
             | Instruction::Smull { condition, .. }
             | Instruction::Smlal { condition, .. }
             | Instruction::Msr { condition, .. }
             | Instruction::Mrs { condition, .. }
             | Instruction::Ldrh { condition, .. }
             | Instruction::Ldrsb { condition, .. }
             | Instruction::Ldrsh { condition, .. }
             | Instruction::Strh { condition, .. }
             | Instruction::Ldrbt { condition, .. }
             | Instruction::Ldrt { condition, .. }
             | Instruction::Ldrb { condition, .. }
             | Instruction::Ldr { condition, .. }
             | Instruction::Strbt { condition, .. }
             | Instruction::Strt { condition, .. }
             | Instruction::Strb { condition, .. }
             | Instruction::Str { condition, .. }
             | Instruction::Ldm1 { condition, .. }
             | Instruction::Ldm2 { condition, .. }
             | Instruction::Ldm3 { condition, .. }
             | Instruction::Stm1 { condition, .. }
             | Instruction::Stm2 { condition, .. }
             | Instruction::Swpb { condition, .. }
             | Instruction::Swp { condition, .. }
             | Instruction::Swi { condition, .. }
             | Instruction::Cdp { condition, .. }
             | Instruction::Ldc { condition, .. }
             | Instruction::Mcr { condition, .. }
             | Instruction::Mrc { condition, .. }
             | Instruction::Stc { condition, .. } => condition,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Condition {
    Eq, // Equal
    Ne, // Not equal
    Cs, // Carry set/unsigned higher or same
    Cc, // Carry clear/unsigned lower
    Mi, // Minus/Negative
    Pl, // Plus/positive or zero
    Vs, // Overflow
    Vc, // No overflow
    Hi, // Unsigned higher
    Ls, // Unsigned lower or same
    Ge, // Signed greater than or equal
    Lt, // Signed less than
    Gt, // Signed greater than
    Le, // Signed less than or equal
    Al, // Always (unconditional)
    Nv, // Never
}

#[derive(PartialEq)]
pub enum AddressMode1 {
    Immediate {
        value: u8,
        rotate: u8,
    },
    Shift {
        rm: Register,
        shift: ShiftDirection,
        shift_imm: AddressingOffset,
    },
}

#[derive(PartialEq)]
pub enum ShiftDirection {
    Asr, // Arithmetic shift right
    Lsl, // Logical shift left
    Lsr, // Logical shift right
    Ror, // Rotate right
    Rrx, // Rotate right with extend
}

#[derive(PartialEq)]
pub enum AddressingOffset {
    Immediate(u16),
    Register(Register),
    ScaledRegister {
        rm: Register,
        shift: ShiftDirection,
        shift_imm: u8,
    }
}

#[derive(PartialEq)]
pub struct AddressMode2 {
    pub rn: Register,
    pub offset: AddressingOffset,
    pub addressing: AddressingMode,
    pub u: bool,
}

#[derive(PartialEq)]
pub struct AddressMode3 {
    pub rn: Register,
    pub offset: AddressingOffset,
    pub addressing: AddressingMode,
    pub u: bool,
}

#[derive(PartialEq)]
pub enum AddressingMode {
    Offset,
    PreIndexed,
    PostIndexed,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match *self {
            Instruction::B { condition, l, signed_immed } => {
                format!("b{}{}\t0x{:x}",
                        format_bool(l, "l"),
                        format_condition(condition),
                        // TODO: add PC
                        ((((signed_immed as i32) << 8) >> 6) as u32) + 8)
            },

            Instruction::Bx { condition, rm } => {
                format!("bx{}\t{}",
                        format_condition(condition),
                        format_register(rm))
            },

            Instruction::And { condition, s, rd, rn, ref operand2 } => {
                format!("and{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Eor { condition, s, rd, rn, ref operand2 } => {
                format!("eor{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Sub { condition, s, rd, rn, ref operand2 } => {
                format!("sub{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Rsb { condition, s, rd, rn, ref operand2 } => {
                format!("rsb{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Add { condition, s, rd, rn, ref operand2 } => {
                format!("add{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Adc { condition, s, rd, rn, ref operand2 } => {
                format!("adc{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Sbc { condition, s, rd, rn, ref operand2 } => {
                format!("sbc{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Rsc { condition, s, rd, rn, ref operand2 } => {
                format!("rsc{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Tst { condition, rn, ref operand2 } => {
                format!("tst{}\t{}, {}",
                        format_condition(condition),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Teq { condition, rn, ref operand2 } => {
                format!("teq{}\t{}, {}",
                        format_condition(condition),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Cmp { condition, rn, ref operand2 } => {
                format!("cmp{}\t{}, {}",
                        format_condition(condition),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Cmn { condition, rn, ref operand2 } => {
                format!("cmn{}\t{}, {}",
                        format_condition(condition),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Orr { condition, s, rd, rn, ref operand2 } => {
                format!("orr{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Mov { condition, s, rd, ref operand2 } => {
                format!("mov{}{}\t{}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_address_mode_1(operand2))
            },

            Instruction::Bic { condition, s, rd, rn, ref operand2 } => {
                format!("bic{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rn),
                        format_address_mode_1(operand2))
            },

            Instruction::Mvn { condition, s, rd, ref operand2 } => {
                format!("mvn{}{}\t{}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_address_mode_1(operand2))
            },

            Instruction::Mul { condition, s, rd, rm, rs } => {
                format!("mul{}{}\t{}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rm),
                        format_register(rs))
            },

            Instruction::Mla { condition, s, rd, rm, rs, rn } => {
                format!("mla{}{}\t{}, {}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd),
                        format_register(rm),
                        format_register(rs),
                        format_register(rn))
            },

            Instruction::Umull { condition, s, rd_lo, rd_hi, rm, rs } => {
                format!("umull{}{}\t{}, {}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd_lo),
                        format_register(rd_hi),
                        format_register(rm),
                        format_register(rs))
            },

            Instruction::Umlal { condition, s, rd_lo, rd_hi, rm, rs } => {
                format!("umlal{}{}\t{}, {}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd_lo),
                        format_register(rd_hi),
                        format_register(rm),
                        format_register(rs))
            },

            Instruction::Smull { condition, s, rd_lo, rd_hi, rm, rs } => {
                format!("smull{}{}\t{}, {}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd_lo),
                        format_register(rd_hi),
                        format_register(rm),
                        format_register(rs))
            },

            Instruction::Smlal { condition, s, rd_lo, rd_hi, rm, rs } => {
                format!("smlal{}{}\t{}, {}, {}, {}",
                        format_condition(condition),
                        format_bool(s, "s"),
                        format_register(rd_lo),
                        format_register(rd_hi),
                        format_register(rm),
                        format_register(rs))
            },

            Instruction::Msr { condition, r, c, x, s, f, ref address } => {
                format!("msr{}\t{}_{}{}{}{}, {}",
                        format_condition(condition),
                        if r { "spsr" } else { "cpsr" },
                        format_bool(c, "c"),
                        format_bool(x, "x"),
                        format_bool(s, "s"),
                        format_bool(f, "f"),
                        format_address_mode_1(address))
            },

            Instruction::Mrs { condition, rd, r } => {
                format!("mrs{}\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        if r { "spsr" } else { "cpsr" })
            },

            Instruction::Ldrh { condition, rd, ref address } => {
                format!("ldr{}h\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_3(address))
            },

            Instruction::Ldrsb { condition, rd, ref address } => {
                format!("ldr{}sb\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_3(address))
            },

            Instruction::Ldrsh { condition, rd, ref address } => {
                format!("ldr{}sh\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_3(address))
            },

            Instruction::Strh { condition, rd, ref address } => {
                format!("str{}h\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_3(address))
            },

            Instruction::Ldrbt { condition, rd, ref address } => {
                format!("ldr{}bt\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Ldrt { condition, rd, ref address } => {
                format!("ldr{}t\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Ldrb { condition, rd, ref address } => {
                format!("ldr{}b\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Ldr { condition, rd, ref address } => {
                format!("ldr{}\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Strbt { condition, rd, ref address } => {
                format!("str{}bt\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Strt { condition, rd, ref address } => {
                format!("str{}t\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Strb { condition, rd, ref address } => {
                format!("str{}b\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Str { condition, rd, ref address } => {
                format!("str{}\t{}, {}",
                        format_condition(condition),
                        format_register(rd),
                        format_address_mode_2(address))
            },

            Instruction::Ldm1 { condition, .. } => {
                format!("ldm1{}",
                        format_condition(condition))
            },

            Instruction::Ldm2 { condition, .. } => {
                format!("ldm2{}",
                        format_condition(condition))
            },

            Instruction::Ldm3 { condition, .. } => {
                format!("ldm3{}",
                        format_condition(condition))
            },

            Instruction::Stm1 { condition, .. } => {
                format!("stm1{}",
                        format_condition(condition))
            },

            Instruction::Stm2 { condition, .. } => {
                format!("stm2{}",
                        format_condition(condition))
            },

            Instruction::Swpb { condition, .. } => {
                format!("swpb{}",
                        format_condition(condition))
            },

            Instruction::Swp { condition, .. } => {
                format!("swp{}",
                        format_condition(condition))
            },

            Instruction::Swi { condition, .. } => {
                format!("swi{}",
                        format_condition(condition))
            },

            Instruction::Cdp { condition, .. } => {
                format!("cdp{}",
                        format_condition(condition))
            },

            Instruction::Ldc { condition, .. } => {
                format!("ldc{}",
                        format_condition(condition))
            },

            Instruction::Mcr { condition, .. } => {
                format!("mcr{}",
                        format_condition(condition))
            },

            Instruction::Mrc { condition, .. } => {
                format!("mrc{}",
                        format_condition(condition))
            },

            Instruction::Stc { condition, .. } => {
                format!("stc{}",
                        format_condition(condition))
            },
        };

        write!(f, "{}", value)
    }
}

// TODO: implement display for AddressMode1
fn format_address_mode_1(address: &AddressMode1) -> String {
    match *address {
        AddressMode1::Immediate { value, rotate } => {
            let immediate = (value as u32).rotate_right((rotate as u32) * 2);
            format!("#{:x}", immediate)
        },

        AddressMode1::Shift { rm, ref shift, ref shift_imm } => {
            let rm = format_register(rm);

            let formatted_amount = match *shift_imm {
                AddressingOffset::Immediate(value) => {
                    (if value == 0 { 32 } else { value }).to_string()
                },
                AddressingOffset::Register(register) => {
                    format_register(register)
                },
                AddressingOffset::ScaledRegister { .. } => {
                    unreachable!()
                },
            };

            match *shift {
                ShiftDirection::Asr => {
                    format!("{}, asr {}", rm, formatted_amount)
                },
                ShiftDirection::Lsl => {
                    if *shift_imm == AddressingOffset::Immediate(0) { return rm };
                    format!("{}, lsl {}", rm, formatted_amount)
                },
                ShiftDirection::Lsr => {
                    format!("{}, lsr {}", rm, formatted_amount)
                },
                ShiftDirection::Ror => {
                    format!("{}, ror {}", rm, formatted_amount)
                },
                ShiftDirection::Rrx => {
                    format!("{}, rrx", rm)
                },
            }
        }
    }
}

// TODO: implement display for AddressMode2
fn format_address_mode_2(address: &AddressMode2) -> String {
    let AddressMode2 { rn, ref offset, ref addressing, u } = *address;
    let formatted_offset = match *offset {
        AddressingOffset::Immediate(byte) => format!("{:x}", byte),
        AddressingOffset::Register(register) => format_register(register),
        AddressingOffset::ScaledRegister { rm, ref shift, ref shift_imm } => {
            // +/-<Rm>, <shift> #<shift_imm>
            let rm = format_register(rm);
            let shift_imm = (if *shift_imm == 0 { 32 } else { *shift_imm }).to_string();

            match *shift {
                ShiftDirection::Asr => {
                    format!("{}, asr {}", rm, shift_imm)
                },
                ShiftDirection::Lsl => {
                    format!("{}, lsl {}", rm, shift_imm)
                },
                ShiftDirection::Lsr => {
                    format!("{}, lsr {}", rm, shift_imm)
                },
                ShiftDirection::Ror => {
                    format!("{}, ror {}", rm, shift_imm)
                },
                ShiftDirection::Rrx => {
                    format!("{}, rrx", rm)
                },
            }
        },
    };

    let signed_offset = format!("#{}{}", format_bool(!u, "-"), formatted_offset);

    match *addressing {
        AddressingMode::Offset =>
            format!("[{}, {}]", format_register(rn), signed_offset),
        AddressingMode::PreIndexed =>
            format!("[{}, {}]!", format_register(rn), signed_offset),
        AddressingMode::PostIndexed =>
            format!("[{}], {}", format_register(rn), signed_offset),
    }
}

// TODO: implement display for AddressMode3
fn format_address_mode_3(address: &AddressMode3) -> String {
    let AddressMode3 { rn, ref offset, ref addressing, u } = *address;
    let formatted_offset = match *offset {
        AddressingOffset::Immediate(byte) => format!("{:x}", byte),
        AddressingOffset::Register(register) => format_register(register),
        AddressingOffset::ScaledRegister { .. } => unreachable!(),
    };

    let signed_offset = format!("#{}{}", format_bool(!u, "-"), formatted_offset);

    match *addressing {
        AddressingMode::Offset => {
            format!("[{}, {}]", format_register(rn), signed_offset)
        },
        AddressingMode::PreIndexed => {
            format!("[{}, {}]!", format_register(rn), signed_offset)
        },
        AddressingMode::PostIndexed => {
            format!("[{}], {}", format_register(rn), signed_offset)
        },
    }
}

fn format_bool(value: bool, string: &str) -> &str {
    if value { string } else { "" }
}

fn format_condition<'a>(condition: Condition) -> &'a str {
    match condition {
        Condition::Eq => { "eq" },
        Condition::Ne => { "ne" },
        Condition::Cs => { "cs" },
        Condition::Cc => { "cc" },
        Condition::Mi => { "mi" },
        Condition::Pl => { "pl" },
        Condition::Vs => { "vs" },
        Condition::Vc => { "vc" },
        Condition::Hi => { "hi" },
        Condition::Ls => { "ls" },
        Condition::Ge => { "ge" },
        Condition::Lt => { "lt" },
        Condition::Gt => { "gt" },
        Condition::Le => { "le" },
        Condition::Al => { "" },
        Condition::Nv => { "nv" },
    }
}

fn format_register(register: Register) -> String {
    match register.0 {
        i @ 0...11 => { format!("r{}", i) },
        12 => { "ip".to_string() },
        13 => { "sp".to_string() },
        14 => { "lr".to_string() },
        15 => { "pc".to_string() },
        _ => { unreachable!() }
    }
}
