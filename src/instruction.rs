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
        // Condition not extracted into a parent struct because
        // Display trait impl accesses condition and arguments.
        use self::Instruction::*;
        match *self {
            B { condition, .. } => condition,
            Bx { condition, .. } => condition,
            And { condition, .. } => condition,
            Eor { condition, .. } => condition,
            Sub { condition, .. } => condition,
            Rsb { condition, .. } => condition,
            Add { condition, .. } => condition,
            Adc { condition, .. } => condition,
            Sbc { condition, .. } => condition,
            Rsc { condition, .. } => condition,
            Tst { condition, .. } => condition,
            Teq { condition, .. } => condition,
            Cmp { condition, .. } => condition,
            Cmn { condition, .. } => condition,
            Orr { condition, .. } => condition,
            Mov { condition, .. } => condition,
            Bic { condition, .. } => condition,
            Mvn { condition, .. } => condition,
            Mul { condition, .. } => condition,
            Mla { condition, .. } => condition,
            Umull { condition, .. } => condition,
            Umlal { condition, .. } => condition,
            Smull { condition, .. } => condition,
            Smlal { condition, .. } => condition,
            Msr { condition, .. } => condition,
            Mrs { condition, .. } => condition,
            Ldrh { condition, .. } => condition,
            Ldrsb { condition, .. } => condition,
            Ldrsh { condition, .. } => condition,
            Strh { condition, .. } => condition,
            Ldrbt { condition, .. } => condition,
            Ldrt { condition, .. } => condition,
            Ldrb { condition, .. } => condition,
            Ldr { condition, .. } => condition,
            Strbt { condition, .. } => condition,
            Strt { condition, .. } => condition,
            Strb { condition, .. } => condition,
            Str { condition, .. } => condition,
            Ldm1 { condition, .. } => condition,
            Ldm2 { condition, .. } => condition,
            Ldm3 { condition, .. } => condition,
            Stm1 { condition, .. } => condition,
            Stm2 { condition, .. } => condition,
            Swpb { condition, .. } => condition,
            Swp { condition, .. } => condition,
            Swi { condition, .. } => condition,
            Cdp { condition, .. } => condition,
            Ldc { condition, .. } => condition,
            Mcr { condition, .. } => condition,
            Mrc { condition, .. } => condition,
            Stc { condition, .. } => condition,
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

pub struct AddressMode1 {
    pub i: bool,
    pub operand: u32,
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
        use self::Instruction::*;
        let value = match *self {
            B { .. } => { "B" },
            Bx { .. } => { "Bx" },
            And { .. } => { "And" },
            Eor { .. } => { "Eor" },
            Sub { .. } => { "Sub" },
            Rsb { .. } => { "Rsb" },
            Add { .. } => { "Add" },
            Adc { .. } => { "Adc" },
            Sbc { .. } => { "Sbc" },
            Rsc { .. } => { "Rsc" },
            Tst { .. } => { "Tst" },
            Teq { .. } => { "Teq" },
            Cmp { .. } => { "Cmp" },
            Cmn { .. } => { "Cmn" },
            Orr { .. } => { "Orr" },
            Mov { .. } => { "Mov" },
            Bic { .. } => { "Bic" },
            Mvn { .. } => { "Mvn" },
            Mul { .. } => { "Mul" },
            Mla { .. } => { "Mla" },
            Umull { .. } => { "Umull" },
            Umlal { .. } => { "Umlal" },
            Smull { .. } => { "Smull" },
            Smlal { .. } => { "Smlal" },
            Msr { .. } => { "Msr" },
            Mrs { .. } => { "Mrs" },
            Ldrh { .. } => { "Ldrh" },
            Ldrsb { .. } => { "Ldrsb" },
            Ldrsh { .. } => { "Ldrsh" },
            Strh { .. } => { "Strh" },
            Ldrbt { .. } => { "Ldrbt" },
            Ldrt { .. } => { "Ldrt" },
            Ldrb { .. } => { "Ldrb" },
            Ldr { .. } => { "Ldr" },
            Strbt { .. } => { "Strbt" },
            Strt { .. } => { "Strt" },
            Strb { .. } => { "Strb" },
            Str { .. } => { "Str" },
            Ldm1 { .. } => { "Ldm1" },
            Ldm2 { .. } => { "Ldm2" },
            Ldm3 { .. } => { "Ldm3" },
            Stm1 { .. } => { "Stm1" },
            Stm2 { .. } => { "Stm2" },
            Swpb { .. } => { "Swpb" },
            Swp { .. } => { "Swp" },
            Swi { .. } => { "Swi" },
            Cdp { .. } => { "Cdp" },
            Ldc { .. } => { "Ldc" },
            Mcr { .. } => { "Mcr" },
            Mrc { .. } => { "Mrc" },
            Stc { .. } => { "Stc" },
        };

        write!(f, "{}", value)
    }
}
