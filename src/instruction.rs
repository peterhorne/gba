use cpu::Register;

pub struct Instruction {
    pub condition: Condition,
    pub operation: Operation,
}

impl Instruction {
    pub fn new(condition: Condition, operation: Operation) -> Instruction {
        Instruction { condition: condition, operation: operation }
    }
}

pub enum Condition {
    Eq, Ne, Cs, Cc, Mi, Pl, Vs, Vc, Hi, Ls, Ge, Lt, Gt, Le, Al, Nv }

pub enum Operation {
    Branch {
        l: bool,
        signed_immed: u32
    },

    BranchAndExchange {
        rm: Register
    },

    Coprocessor {
        operation: CoprocessorOperation,
        coprocessor: u32,
        opcode1: u32,
        opcode2: u32,
        crd: u32,
        crn: u32,
        crm: u32
    },

    DataProcessing {
        operation: DataProcessingOperation,
        s: bool,
        rn: Register,
        rd: Register,
        address: AddressingMode1,
    },

    LoadAndStoreHalfwordOrSignedByte {
        operation: LoadAndStoreHalfwordOrSignedByteOperation,
        rd: Register,
        address: AddressingMode3,
    },

    LoadAndStoreMultiple {
        operation: LoadAndStoreMultipleOperation,
    },

    LoadAndStoreWordOrUnsignedByte {
        operation: LoadAndStoreWordOrUnsignedByteOperation,
        rd: Register,
        address: AddressingMode2,
    },

    Multiply {
        operation: MultiplyOperation,
        s: bool,
        rd: Register,
        rn: Register,
        rm: Register,
        rs: Register,
    },

    Semaphore {
        b: bool,
        rd: Register,
        rm: Register,
        rn: Register,
    },

    SoftwareInterrupt {
        immediate: u32,
    },

    StatusRegister {
        operation: StatusRegisterOperation,
        c: bool,
        x: bool,
        s: bool,
        f: bool,
        r: bool,
        rd: Register,
        address: AddressingMode1,
    },
}

pub enum CoprocessorOperation {
    Cdp, Ldc, Mcr, Mrc, Stc }

pub enum DataProcessingOperation {
    And, Eor, Sub, Rsb, Add, Adc, Sbc, Rsc,
    Tst, Teq, Cmp, Cmn, Orr, Mov, Bic, Mvn }

pub enum LoadAndStoreHalfwordOrSignedByteOperation {
    Ldrh, Strh, Ldrsb, Ldrsh }

pub enum LoadAndStoreMultipleOperation {
    Ldm1, Ldm2, Ldm3, Stm1, Stm2 }

pub enum LoadAndStoreWordOrUnsignedByteOperation {
    Ldr, Ldrb, Ldrbt, Ldrt, Str, Strb, Strbt, Strt }

pub enum MultiplyOperation {
    Mul, Mla, Umull, Umlal, Smull, Smlal }

pub enum StatusRegisterOperation {
    Msr, Mrs }

pub struct AddressingMode1 {
    pub i: bool,
    pub operand: u32,
}

pub struct AddressingMode2 {
    pub i: bool,
    pub p: bool,
    pub u: bool,
    pub w: bool,
    pub rn: Register,
    pub offset: u32,
}

pub struct AddressingMode3 {
    pub p: bool,
    pub u: bool,
    pub i: bool,
    pub w: bool,
    pub rn: Register,
    pub offset_a: u32,
    pub offset_b: u32,
}
