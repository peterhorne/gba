use cpu::Register;
use std::fmt;

#[derive(Copy, Clone)]
pub enum Instruction {
    // Branch
    B {
        condition: Condition,
        l: bool,
        signed_immed: u32,
    },
    Bx {
        condition: Condition,
        rm: Register,
    },

    // Data processing
    And {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Eor {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Sub {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Rsb {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Add {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Adc {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Sbc {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Rsc {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Tst {
        condition: Condition,
        rn: Register,
        operand2: AddressMode1,
    },
    Teq {
        condition: Condition,
        rn: Register,
        operand2: AddressMode1,
    },
    Cmp {
        condition: Condition,
        rn: Register,
        operand2: AddressMode1,
    },
    Cmn {
        condition: Condition,
        rn: Register,
        operand2: AddressMode1,
    },
    Orr {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Mov {
        condition: Condition,
        s: bool,
        rd: Register,
        operand2: AddressMode1,
    },
    Bic {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        operand2: AddressMode1,
    },
    Mvn {
        condition: Condition,
        s: bool,
        rd: Register,
        operand2: AddressMode1,
    },

    // Multiply
    Mul {
        condition: Condition,
        s: bool,
        rd: Register,
        rm: Register,
        rs: Register,
    },
    Mla {
        condition: Condition,
        s: bool,
        rd: Register,
        rn: Register,
        rm: Register,
        rs: Register,
    },
    Umull {
        condition: Condition,
        s: bool,
        rd_lo: Register,
        rd_hi: Register,
        rm: Register,
        rs: Register,
    },
    Umlal {
        condition: Condition,
        s: bool,
        rd_lo: Register,
        rd_hi: Register,
        rm: Register,
        rs: Register,
    },
    Smull {
        condition: Condition,
        s: bool,
        rd_lo: Register,
        rd_hi: Register,
        rm: Register,
        rs: Register,
    },
    Smlal {
        condition: Condition,
        s: bool,
        rd_lo: Register,
        rd_hi: Register,
        rm: Register,
        rs: Register,
    },

    // Status register access
    Msr {
        condition: Condition,
        c: bool,
        x: bool,
        s: bool,
        f: bool,
        r: bool,
        address: AddressMode1,
    },
    Mrs {
        condition: Condition,
        r: bool,
        rd: Register,
    },

    // Load/store (halfword or signed byte)
    Ldrh {
        condition: Condition,
        rd: Register,
        address: AddressMode3,
    },
    Ldrsb {
        condition: Condition,
        rd: Register,
        address: AddressMode3,
    },
    Ldrsh {
        condition: Condition,
        rd: Register,
        address: AddressMode3,
    },
    Strh {
        condition: Condition,
        rd: Register,
        address: AddressMode3,
    },

    // Load/store (word or unsigned byte)
    Ldrbt {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },
    Ldrt {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },
    Ldrb {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },
    Ldr {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },
    Strbt {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },
    Strt {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },
    Strb {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },
    Str {
        condition: Condition,
        rd: Register,
        address: AddressMode2,
    },

    // LoadAndStoreMultiple
    Ldm1 {
        condition: Condition,
    }, // TODO
    Ldm2 {
        condition: Condition,
    }, // TODO
    Ldm3 {
        condition: Condition,
    }, // TODO
    Stm1 {
        condition: Condition,
    }, // TODO
    Stm2 {
        condition: Condition,
    }, // TODO

    // Semaphore
    Swpb {
        condition: Condition,
        rd: Register,
        rm: Register,
        rn: Register,
    },
    Swp {
        condition: Condition,
        rd: Register,
        rm: Register,
        rn: Register,
    },

    // ExceptionGenerating
    Swi {
        condition: Condition,
        immediate: u32,
    },

    // Coprocessor
    Cdp {
        condition: Condition,
    }, // TODO
    Ldc {
        condition: Condition,
    }, // TODO
    Mcr {
        condition: Condition,
    }, // TODO
    Mrc {
        condition: Condition,
    }, // TODO
    Stc {
        condition: Condition,
    }, // TODO
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

#[derive(Copy, Clone, PartialEq)]
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

#[derive(Copy, Clone, PartialEq)]
pub enum ShiftDirection {
    Asr, // Arithmetic shift right
    Lsl, // Logical shift left
    Lsr, // Logical shift right
    Ror, // Rotate right
    Rrx, // Rotate right with extend
}

#[derive(Copy, Clone, PartialEq)]
pub enum AddressingOffset {
    Immediate(u16),
    Register(Register),
    ScaledRegister {
        rm: Register,
        shift: ShiftDirection,
        shift_imm: u8,
    },
}

#[derive(Copy, Clone, PartialEq)]
pub struct AddressMode2 {
    pub rn: Register,
    pub offset: AddressingOffset,
    pub addressing: AddressingMode,
    pub u: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub struct AddressMode3 {
    pub rn: Register,
    pub offset: AddressingOffset,
    pub addressing: AddressingMode,
    pub u: bool,
}

#[derive(Copy, Clone, PartialEq)]
pub enum AddressingMode {
    Offset,
    PreIndexed,
    PostIndexed,
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Instruction::B {
                condition,
                l,
                signed_immed,
            } => {
                // TODO: add PC
                let value = ((((signed_immed as i32) << 8) >> 6) as u32) + 8;
                write!(f, "b{}{}\t{:#x}", format_bool(l, "l"), condition, value)
            }

            Instruction::Bx { condition, rm } => {
                write!(f, "bx{}\t{}", condition, rm)
            }

            Instruction::And {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "and{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Eor {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "eor{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Sub {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "sub{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Rsb {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "rsb{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Add {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "add{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Adc {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "adc{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Sbc {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "sbc{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Rsc {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "rsc{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Tst {
                condition,
                rn,
                ref operand2,
            } => write!(f, "tst{}\t{}, {}", condition, rn, operand2),

            Instruction::Teq {
                condition,
                rn,
                ref operand2,
            } => write!(f, "teq{}\t{}, {}", condition, rn, operand2),

            Instruction::Cmp {
                condition,
                rn,
                ref operand2,
            } => write!(f, "cmp{}\t{}, {}", condition, rn, operand2),

            Instruction::Cmn {
                condition,
                rn,
                ref operand2,
            } => write!(f, "cmn{}\t{}, {}", condition, rn, operand2),

            Instruction::Orr {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "orr{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Mov {
                condition,
                s,
                rd,
                ref operand2,
            } => write!(
                f,
                "mov{}{}\t{}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                operand2
            ),

            Instruction::Bic {
                condition,
                s,
                rd,
                rn,
                ref operand2,
            } => write!(
                f,
                "bic{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rn,
                operand2
            ),

            Instruction::Mvn {
                condition,
                s,
                rd,
                ref operand2,
            } => write!(
                f,
                "mvn{}{}\t{}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                operand2
            ),

            Instruction::Mul {
                condition,
                s,
                rd,
                rm,
                rs,
            } => write!(
                f,
                "mul{}{}\t{}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rm,
                rs
            ),

            Instruction::Mla {
                condition,
                s,
                rd,
                rm,
                rs,
                rn,
            } => write!(
                f,
                "mla{}{}\t{}, {}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd,
                rm,
                rs,
                rn
            ),

            Instruction::Umull {
                condition,
                s,
                rd_lo,
                rd_hi,
                rm,
                rs,
            } => write!(
                f,
                "umull{}{}\t{}, {}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd_lo,
                rd_hi,
                rm,
                rs
            ),

            Instruction::Umlal {
                condition,
                s,
                rd_lo,
                rd_hi,
                rm,
                rs,
            } => write!(
                f,
                "umlal{}{}\t{}, {}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd_lo,
                rd_hi,
                rm,
                rs
            ),

            Instruction::Smull {
                condition,
                s,
                rd_lo,
                rd_hi,
                rm,
                rs,
            } => write!(
                f,
                "smull{}{}\t{}, {}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd_lo,
                rd_hi,
                rm,
                rs
            ),

            Instruction::Smlal {
                condition,
                s,
                rd_lo,
                rd_hi,
                rm,
                rs,
            } => write!(
                f,
                "smlal{}{}\t{}, {}, {}, {}",
                condition,
                format_bool(s, "s"),
                rd_lo,
                rd_hi,
                rm,
                rs
            ),

            Instruction::Msr {
                condition,
                r,
                c,
                x,
                s,
                f: f2,
                ref address,
            } => write!(
                f,
                "msr{}\t{}_{}{}{}{}, {}",
                condition,
                if r { "spsr" } else { "cpsr" },
                format_bool(c, "c"),
                format_bool(x, "x"),
                format_bool(s, "s"),
                format_bool(f2, "f"),
                address
            ),

            Instruction::Mrs { condition, rd, r } => write!(
                f,
                "mrs{}\t{}, {}",
                condition,
                rd,
                if r { "spsr" } else { "cpsr" }
            ),

            Instruction::Ldrh {
                condition,
                rd,
                ref address,
            } => write!(f, "ldr{}h\t{}, {}", condition, rd, address),

            Instruction::Ldrsb {
                condition,
                rd,
                ref address,
            } => write!(f, "ldr{}sb\t{}, {}", condition, rd, address),

            Instruction::Ldrsh {
                condition,
                rd,
                ref address,
            } => write!(f, "ldr{}sh\t{}, {}", condition, rd, address),

            Instruction::Strh {
                condition,
                rd,
                ref address,
            } => write!(f, "str{}h\t{}, {}", condition, rd, address),

            Instruction::Ldrbt {
                condition,
                rd,
                ref address,
            } => write!(f, "ldr{}bt\t{}, {}", condition, rd, address),

            Instruction::Ldrt {
                condition,
                rd,
                ref address,
            } => write!(f, "ldr{}t\t{}, {}", condition, rd, address),

            Instruction::Ldrb {
                condition,
                rd,
                ref address,
            } => write!(f, "ldr{}b\t{}, {}", condition, rd, address),

            Instruction::Ldr {
                condition,
                rd,
                ref address,
            } => write!(f, "ldr{}\t{}, {}", condition, rd, address),

            Instruction::Strbt {
                condition,
                rd,
                ref address,
            } => write!(f, "str{}bt\t{}, {}", condition, rd, address),

            Instruction::Strt {
                condition,
                rd,
                ref address,
            } => write!(f, "str{}t\t{}, {}", condition, rd, address),

            Instruction::Strb {
                condition,
                rd,
                ref address,
            } => write!(f, "str{}b\t{}, {}", condition, rd, address),

            Instruction::Str {
                condition,
                rd,
                ref address,
            } => write!(f, "str{}\t{}, {}", condition, rd, address),

            Instruction::Ldm1 { condition, .. } => {
                write!(f, "ldm1{}", condition)
            }

            Instruction::Ldm2 { condition, .. } => {
                write!(f, "ldm2{}", condition)
            }

            Instruction::Ldm3 { condition, .. } => {
                write!(f, "ldm3{}", condition)
            }

            Instruction::Stm1 { condition, .. } => {
                write!(f, "stm1{}", condition)
            }

            Instruction::Stm2 { condition, .. } => {
                write!(f, "stm2{}", condition)
            }

            Instruction::Swpb { condition, .. } => {
                write!(f, "swpb{}", condition)
            }

            Instruction::Swp { condition, .. } => write!(f, "swp{}", condition),

            Instruction::Swi { condition, .. } => write!(f, "swi{}", condition),

            Instruction::Cdp { condition, .. } => write!(f, "cdp{}", condition),

            Instruction::Ldc { condition, .. } => write!(f, "ldc{}", condition),

            Instruction::Mcr { condition, .. } => write!(f, "mcr{}", condition),

            Instruction::Mrc { condition, .. } => write!(f, "mrc{}", condition),

            Instruction::Stc { condition, .. } => write!(f, "stc{}", condition),
        }
    }
}

impl fmt::Display for AddressMode1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            AddressMode1::Immediate { value, rotate } => {
                let immediate =
                    (value as u32).rotate_right((rotate as u32) * 2);
                write!(f, "#{:#x}", immediate)
            }

            AddressMode1::Shift {
                rm,
                ref shift,
                ref shift_imm,
            } => {
                let formatted_amount = match *shift_imm {
                    AddressingOffset::Immediate(value) => {
                        format!("{:#x}", if value == 0 { 32 } else { value })
                    }
                    AddressingOffset::Register(register) => {
                        format!("{}", register)
                    }
                    AddressingOffset::ScaledRegister { .. } => unreachable!(),
                };

                match *shift {
                    ShiftDirection::Asr => {
                        write!(f, "{}, asr {}", rm, formatted_amount)
                    }
                    ShiftDirection::Lsl => {
                        if *shift_imm == AddressingOffset::Immediate(0) {
                            write!(f, "{}", rm)
                        } else {
                            write!(f, "{}, lsl {}", rm, formatted_amount)
                        }
                    }
                    ShiftDirection::Lsr => {
                        write!(f, "{}, lsr {}", rm, formatted_amount)
                    }
                    ShiftDirection::Ror => {
                        write!(f, "{}, ror {}", rm, formatted_amount)
                    }
                    ShiftDirection::Rrx => write!(f, "{}, rrx", rm),
                }
            }
        }
    }
}

impl fmt::Display for AddressMode2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let AddressMode2 {
            rn,
            ref offset,
            ref addressing,
            u,
        } = *self;
        let formatted_offset = match *offset {
            AddressingOffset::Immediate(byte) => format!("{:#x}", byte),
            AddressingOffset::Register(register) => format!("{}", register),
            AddressingOffset::ScaledRegister {
                rm,
                ref shift,
                ref shift_imm,
            } => {
                // +/-<Rm>, <shift> #<shift_imm>

                let shift_imm = if *shift_imm == 0 { 32 } else { *shift_imm };
                let shift_imm = format!("{:#x}", shift_imm);

                match *shift {
                    ShiftDirection::Asr => format!("{}, asr {}", rm, shift_imm),
                    ShiftDirection::Lsl => format!("{}, lsl {}", rm, shift_imm),
                    ShiftDirection::Lsr => format!("{}, lsr {}", rm, shift_imm),
                    ShiftDirection::Ror => format!("{}, ror {}", rm, shift_imm),
                    ShiftDirection::Rrx => format!("{}, rrx", rm),
                }
            }
        };

        let signed_offset =
            format!("#{}{}", format_bool(!u, "-"), formatted_offset);

        match *addressing {
            AddressingMode::Offset => write!(f, "[{}, {}]", rn, signed_offset),
            AddressingMode::PreIndexed => {
                write!(f, "[{}, {}]!", rn, signed_offset)
            }
            AddressingMode::PostIndexed => {
                write!(f, "[{}], {}", rn, signed_offset)
            }
        }
    }
}

impl fmt::Display for AddressMode3 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let AddressMode3 {
            rn,
            ref offset,
            ref addressing,
            u,
        } = *self;
        let formatted_offset = match *offset {
            AddressingOffset::Immediate(byte) => format!("{:#x}", byte),
            AddressingOffset::Register(register) => format!("{}", register),
            AddressingOffset::ScaledRegister { .. } => unreachable!(),
        };

        let signed_offset =
            format!("#{}{}", format_bool(!u, "-"), formatted_offset);

        match *addressing {
            AddressingMode::Offset => write!(f, "[{}, {}]", rn, signed_offset),
            AddressingMode::PreIndexed => {
                write!(f, "[{}, {}]!", rn, signed_offset)
            }
            AddressingMode::PostIndexed => {
                write!(f, "[{}], {}", rn, signed_offset)
            }
        }
    }
}

impl fmt::Display for Condition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Condition::Eq => write!(f, "eq"),
            Condition::Ne => write!(f, "ne"),
            Condition::Cs => write!(f, "cs"),
            Condition::Cc => write!(f, "cc"),
            Condition::Mi => write!(f, "mi"),
            Condition::Pl => write!(f, "pl"),
            Condition::Vs => write!(f, "vs"),
            Condition::Vc => write!(f, "vc"),
            Condition::Hi => write!(f, "hi"),
            Condition::Ls => write!(f, "ls"),
            Condition::Ge => write!(f, "ge"),
            Condition::Lt => write!(f, "lt"),
            Condition::Gt => write!(f, "gt"),
            Condition::Le => write!(f, "le"),
            Condition::Al => write!(f, ""),
            Condition::Nv => write!(f, "nv"),
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            i @ 0...11 => write!(f, "r{}", i),
            12 => write!(f, "ip"),
            13 => write!(f, "sp"),
            14 => write!(f, "lr"),
            15 => write!(f, "pc"),
            _ => unreachable!(),
        }
    }
}

fn format_bool(value: bool, string: &str) -> &str {
    if value {
        string
    } else {
        ""
    }
}
