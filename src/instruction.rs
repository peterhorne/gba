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
    Tst { condition: Condition, s: bool, rn: Register, operand2: AddressMode1 },
    Teq { condition: Condition, s: bool, rn: Register, operand2: AddressMode1 },
    Cmp { condition: Condition, s: bool, rn: Register, operand2: AddressMode1 },
    Cmn { condition: Condition, s: bool, rn: Register, operand2: AddressMode1 },
    Orr { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Mov { condition: Condition, s: bool, rd: Register, operand2: AddressMode1 },
    Bic { condition: Condition, s: bool, rd: Register, rn: Register, operand2: AddressMode1 },
    Mvn { condition: Condition, s: bool, rd: Register, operand2: AddressMode1 },

    // Multiply
    Mul { condition: Condition, s: bool, rd: Register, rm: Register, rs: Register },
    Mla { condition: Condition, s: bool, rd: Register, rn: Register, rm: Register, rs: Register },
    Umull { condition: Condition, s: bool, rd: Register, rn: Register, rm: Register, rs: Register },
    Umlal { condition: Condition, s: bool, rd: Register, rn: Register, rm: Register, rs: Register },
    Smull { condition: Condition, s: bool, rd: Register, rn: Register, rm: Register, rs: Register },
    Smlal { condition: Condition, s: bool, rd: Register, rn: Register, rm: Register, rs: Register },

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
            Instruction::B { condition, .. } => condition,
            Instruction::Bx { condition, .. } => condition,
            Instruction::And { condition, .. } => condition,
            Instruction::Eor { condition, .. } => condition,
            Instruction::Sub { condition, .. } => condition,
            Instruction::Rsb { condition, .. } => condition,
            Instruction::Add { condition, .. } => condition,
            Instruction::Adc { condition, .. } => condition,
            Instruction::Sbc { condition, .. } => condition,
            Instruction::Rsc { condition, .. } => condition,
            Instruction::Tst { condition, .. } => condition,
            Instruction::Teq { condition, .. } => condition,
            Instruction::Cmp { condition, .. } => condition,
            Instruction::Cmn { condition, .. } => condition,
            Instruction::Orr { condition, .. } => condition,
            Instruction::Mov { condition, .. } => condition,
            Instruction::Bic { condition, .. } => condition,
            Instruction::Mvn { condition, .. } => condition,
            Instruction::Mul { condition, .. } => condition,
            Instruction::Mla { condition, .. } => condition,
            Instruction::Umull { condition, .. } => condition,
            Instruction::Umlal { condition, .. } => condition,
            Instruction::Smull { condition, .. } => condition,
            Instruction::Smlal { condition, .. } => condition,
            Instruction::Msr { condition, .. } => condition,
            Instruction::Mrs { condition, .. } => condition,
            Instruction::Ldrh { condition, .. } => condition,
            Instruction::Ldrsb { condition, .. } => condition,
            Instruction::Ldrsh { condition, .. } => condition,
            Instruction::Strh { condition, .. } => condition,
            Instruction::Ldrbt { condition, .. } => condition,
            Instruction::Ldrt { condition, .. } => condition,
            Instruction::Ldrb { condition, .. } => condition,
            Instruction::Ldr { condition, .. } => condition,
            Instruction::Strbt { condition, .. } => condition,
            Instruction::Strt { condition, .. } => condition,
            Instruction::Strb { condition, .. } => condition,
            Instruction::Str { condition, .. } => condition,
            Instruction::Ldm1 { condition, .. } => condition,
            Instruction::Ldm2 { condition, .. } => condition,
            Instruction::Ldm3 { condition, .. } => condition,
            Instruction::Stm1 { condition, .. } => condition,
            Instruction::Stm2 { condition, .. } => condition,
            Instruction::Swpb { condition, .. } => condition,
            Instruction::Swp { condition, .. } => condition,
            Instruction::Swi { condition, .. } => condition,
            Instruction::Cdp { condition, .. } => condition,
            Instruction::Ldc { condition, .. } => condition,
            Instruction::Mcr { condition, .. } => condition,
            Instruction::Mrc { condition, .. } => condition,
            Instruction::Stc { condition, .. } => condition,
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

pub enum AddressMode1 {
    Immediate {
        value: u8,
        rotate: u8
    },
    Shift {
        rm: Register,
        direction: ShiftDirection,
        amount: ShiftAmount
    },
}

pub enum ShiftDirection {
    Asr, // Arithmetic shift right
    Lsl, // Logical shift left
    Lsr, // Logical shift right
    Ror, // Rotate right
    Rrx, // Rotate right with extend
}

pub enum ShiftAmount {
    Immediate(u8),
    Register(Register),
}

pub struct AddressMode2 {
    pub i: bool,
    pub p: bool,
    pub u: bool,
    pub w: bool,
    pub rn: Register,
    pub offset: u32,
}

pub struct AddressMode3 {
    pub p: bool,
    pub u: bool,
    pub i: bool,
    pub w: bool,
    pub rn: Register,
    pub offset_a: u32,
    pub offset_b: u32,
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
            Instruction::Eor { condition, .. } => {
                format!("eor{}",
                        format_condition(condition))
            },
            Instruction::Sub { condition, .. } => {
                format!("sub{}",
                        format_condition(condition))
            },
            Instruction::Rsb { condition, .. } => {
                format!("rsb{}",
                        format_condition(condition))
            },
            Instruction::Add { condition, .. } => {
                format!("add{}",
                        format_condition(condition))
            },
            Instruction::Adc { condition, .. } => {
                format!("adc{}",
                        format_condition(condition))
            },
            Instruction::Sbc { condition, .. } => {
                format!("sbc{}",
                        format_condition(condition))
            },
            Instruction::Rsc { condition, .. } => {
                format!("rsc{}",
                        format_condition(condition))
            },
            Instruction::Tst { condition, .. } => {
                format!("tst{}",
                        format_condition(condition))
            },
            Instruction::Teq { condition, .. } => {
                format!("teq{}",
                        format_condition(condition))
            },
            Instruction::Cmp { condition, .. } => {
                format!("cmp{}",
                        format_condition(condition))
            },
            Instruction::Cmn { condition, .. } => {
                format!("cmn{}",
                        format_condition(condition))
            },
            Instruction::Orr { condition, .. } => {
                format!("orr{}",
                        format_condition(condition))
            },
            Instruction::Mov { condition, .. } => {
                format!("mov{}",
                        format_condition(condition))
            },
            Instruction::Bic { condition, .. } => {
                format!("bic{}",
                        format_condition(condition))
            },
            Instruction::Mvn { condition, .. } => {
                format!("mvn{}",
                        format_condition(condition))
            },
            Instruction::Mul { condition, .. } => {
                format!("mul{}",
                        format_condition(condition))
            },
            Instruction::Mla { condition, .. } => {
                format!("mla{}",
                        format_condition(condition))
            },
            Instruction::Umull { condition, .. } => {
                format!("umull{}",
                        format_condition(condition))
            },
            Instruction::Umlal { condition, .. } => {
                format!("umlal{}",
                        format_condition(condition))
            },
            Instruction::Smull { condition, .. } => {
                format!("smull{}",
                        format_condition(condition))
            },
            Instruction::Smlal { condition, .. } => {
                format!("smlal{}",
                        format_condition(condition))
            },
            Instruction::Msr { condition, .. } => {
                format!("msr{}",
                        format_condition(condition))
            },
            Instruction::Mrs { condition, .. } => {
                format!("mrs{}",
                        format_condition(condition))
            },
            Instruction::Ldrh { condition, .. } => {
                format!("ldrh{}",
                        format_condition(condition))
            },
            Instruction::Ldrsb { condition, .. } => {
                format!("ldrsb{}",
                        format_condition(condition))
            },
            Instruction::Ldrsh { condition, .. } => {
                format!("ldrsh{}",
                        format_condition(condition))
            },
            Instruction::Strh { condition, .. } => {
                format!("strh{}",
                        format_condition(condition))
            },
            Instruction::Ldrbt { condition, .. } => {
                format!("ldrbt{}",
                        format_condition(condition))
            },
            Instruction::Ldrt { condition, .. } => {
                format!("ldrt{}",
                        format_condition(condition))
            },
            Instruction::Ldrb { condition, .. } => {
                format!("ldrb{}",
                        format_condition(condition))
            },
            Instruction::Ldr { condition, .. } => {
                format!("ldr{}",
                        format_condition(condition))
            },
            Instruction::Strbt { condition, .. } => {
                format!("strbt{}",
                        format_condition(condition))
            },
            Instruction::Strt { condition, .. } => {
                format!("strt{}",
                        format_condition(condition))
            },
            Instruction::Strb { condition, .. } => {
                format!("strb{}",
                        format_condition(condition))
            },
            Instruction::Str { condition, .. } => {
                format!("str{}",
                        format_condition(condition))
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

fn format_address_mode_1<'a>(value: &AddressMode1) -> &'a str {
    "todo"
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
