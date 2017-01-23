use bit::{Bit, Bits};
use cpu::Register;
use instruction::*;

pub fn decode_arm(inst: u32) -> Instruction {
    let condition = match inst.bits(28..32) {
        0b0000 => { Condition::Eq }, // Equal
        0b0001 => { Condition::Ne }, // Not equal
        0b0010 => { Condition::Cs }, // Carry set/unsigned higher or same
        0b0011 => { Condition::Cc }, // Carry clear/unsigned lower
        0b0100 => { Condition::Mi }, // Minus/Negative
        0b0101 => { Condition::Pl }, // Plus/positive or zero
        0b0110 => { Condition::Vs }, // Overflow
        0b0111 => { Condition::Vc }, // No overflow
        0b1000 => { Condition::Hi }, // Unsigned higher
        0b1001 => { Condition::Ls }, // Unsigned lower or same
        0b1010 => { Condition::Ge }, // Signed greater than or equal
        0b1011 => { Condition::Lt }, // Signed less than
        0b1100 => { Condition::Gt }, // Signed greater than
        0b1101 => { Condition::Le }, // Signed less than or equal
        0b1110 => { Condition::Al }, // Always (unconditional)
        0b1111 => { Condition::Nv }, // Never
        _      => { unreachable!() },
    };

    // Extended instruction set
    let operation = if inst.bits(24..28) == 0b0000 && inst.bits(4..8) == 0b1001 {
        multiply(inst)
    }

    else if inst.bits(26..28) == 0b00 && inst.bits(23..25) == 0b10 && !inst.bit(20)
        && (inst.bit(25) || !inst.bit(7) || !inst.bit(4)) {

            let bit25   = inst.bit(25);
            let nibble2 = inst.bits(4..8);
            let op1     = inst.bits(21..23);

            if (!bit25 && nibble2 == 0b0000) || bit25 && (op1 == 0b01 || op1 == 0b11) {
                status_register(inst)
            } else if !bit25 && nibble2 == 0b0001 && op1 == 0b01 {
                branch_and_exchange(inst)
            } else {
                unreachable!();
            }
        }

    else if inst.bits(25..28) == 0b000 && inst.bit(7) && inst.bit(4)
        && (inst.bit(24) || inst.bits(5..7) != 0b00) {

            let bits20to25 = inst.bits(20..25);
            let op1        = inst.bits(5..7);

            if (bits20to25 == 0b10000 || bits20to25 == 0b10100) && op1 == 0b00 {
                semaphore(inst)
            } else {
                load_and_store_halfword_or_signed_byte(inst)
            }
        }

    // Standard instruction set
    else {
        match inst.bits(24..28) {
            0b0000...0b0011 => { data_processing(inst) },
            0b0100...0b0111 => { load_and_store_word_or_unsigned_byte(inst) },
            0b1000...0b1001 => { load_and_store_multiple(inst) },
            0b1010...0b1011 => { branch(inst) },
            0b1100...0b1110 => { coprocessor(inst) },
            0b1111          => { software_interrupt(inst) },
            _ => { unreachable!() }
        }
    };

    Instruction::new(condition, operation)
}

fn branch(inst: u32) -> Operation {
    Operation::Branch {
        l: inst.bit(24),
        signed_immed: inst.bits(0..24),
    }
}

fn branch_and_exchange(inst: u32) -> Operation {
    Operation::BranchAndExchange {
        rm: Register(inst.bits(0..4))
    }
}

fn coprocessor(inst: u32) -> Operation {
    let bit25 = inst.bit(25);
    let bit20 = inst.bit(20);
    let bit4  = inst.bit(4);

    use instruction::CoprocessorOperation::*;
    let operation =
        if       bit25 &&  bit20 &&  bit4 { Mrc }
        else if  bit25 &&  bit20 && !bit4 { Cdp }
        else if  bit25 && !bit20          { Mcr }
        else if !bit25 &&  bit20          { Ldc }
        else if !bit25 && !bit20          { Stc }
        else { unreachable!() };

    Operation::Coprocessor {
        operation:   operation,
        opcode1:     inst.bits(20..24),
        crn:         inst.bits(16..20),
        crd:         inst.bits(12..16),
        coprocessor: inst.bits(8..12),
        opcode2:     inst.bits(5..8),
        crm:         inst.bits(0..4),
    }
}

fn data_processing(inst: u32) -> Operation {
    use instruction::DataProcessingOperation::*;
    let operation = match inst.bits(21..25) {
        0b0000 => { And },
        0b0001 => { Eor },
        0b0010 => { Sub },
        0b0011 => { Rsb },
        0b0100 => { Add },
        0b0101 => { Adc },
        0b0110 => { Sbc },
        0b0111 => { Rsc },
        0b1000 => { Tst },
        0b1001 => { Teq },
        0b1010 => { Cmp },
        0b1011 => { Cmn },
        0b1100 => { Orr },
        0b1101 => { Mov },
        0b1110 => { Bic },
        0b1111 => { Mvn },
        _      => { unreachable!() },
    };

    Operation::DataProcessing {
        operation: operation,
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        address: AddressingMode1 {
            i: inst.bit(25),
            operand: inst.bits(0..12),
        }
    }
}

fn load_and_store_halfword_or_signed_byte(inst: u32) -> Operation {
    let l = inst.bit(20);

    use instruction::LoadAndStoreHalfwordOrSignedByteOperation::*;
    let operation = match inst.bits(4..8) {
        0b1011 if l  => { Ldrh },
        0b1011 if !l => { Strh },
        0b1101       => { Ldrsb },
        0b1111       => { Ldrsh },
        _ => unreachable!(),
    };

    Operation::LoadAndStoreHalfwordOrSignedByte {
        operation: operation,
        rd: Register(inst.bits(12..16)),
        address: AddressingMode3 {
            p: inst.bit(24),
            u: inst.bit(23),
            i: inst.bit(22),
            w: inst.bit(21),
            rn: Register(inst.bits(16..20)),
            offset_a: inst.bits(8..12),
            offset_b: inst.bits(0..4),
        }
    }
}

fn load_and_store_multiple(inst: u32) -> Operation {
    let bit22 = inst.bit(22);
    let bit20 = inst.bit(20);
    let bit15 = inst.bit(15);

    use instruction::LoadAndStoreMultipleOperation::*;
    let operation =
        if       bit20 &&  bit22 &&  bit15 { Ldm3 }
        else if  bit20 &&  bit22 && !bit15 { Ldm2 }
        else if  bit20 && !bit22           { Ldm1 }
        else if !bit20 &&  bit22           { Stm2 }
        else if !bit20 && !bit22           { Stm1 }
        else { unreachable!(); };

    Operation::LoadAndStoreMultiple {
        operation: operation,
    }
}

fn load_and_store_word_or_unsigned_byte(inst: u32) -> Operation {
    let i = inst.bit(25);
    let p = inst.bit(24);
    let u = inst.bit(23);
    let b = inst.bit(22);
    let w = inst.bit(21);
    let l = inst.bit(20);
    let rn = Register(inst.bits(16..20));
    let rd = Register(inst.bits(12..16));
    let offset = inst.bits(0..12);

    use instruction::LoadAndStoreWordOrUnsignedByteOperation::*;
    let t = !p && w;
    let operation =
        if      l  && t  && b  { Ldrbt }
        else if l  && t  && !b { Ldrt }
        else if l  && !t && b  { Ldrb }
        else if l  && !t && !b { Ldr }
        else if !l && t  && b  { Strbt }
        else if !l && t  && !b { Strt }
        else if !l && !t && b  { Strb }
        else if !l && !t && !b { Str }
        else { unreachable!(); };

    Operation::LoadAndStoreWordOrUnsignedByte {
        operation: operation,
        rd: rd,
        address: AddressingMode2 {
            i: i,
            p: p,
            u: u,
            w: w,
            rn: rn,
            offset: offset,
        }
    }
}

fn multiply(inst: u32) -> Operation {
    use instruction::MultiplyOperation::*;
    Operation::Multiply {
        operation: match inst.bits(21..24) {
            0b000 | 0b010 => { Mul },
            0b001 | 0b011 => { Mla },
            0b100 => { Umull },
            0b101 => { Umlal },
            0b110 => { Smull },
            0b111 => { Smlal },
            _     => { unreachable!() }
        },
        s: inst.bit(20),
        rd: Register(inst.bits(16..20)),
        rn: Register(inst.bits(12..16)),
        rm: Register(inst.bits(0..4)),
        rs: Register(inst.bits(8..12)),
    }
}

fn semaphore(inst: u32) -> Operation {
    Operation::Semaphore {
        b: inst.bit(22),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        rm: Register(inst.bits(0..4)),
    }

}
fn software_interrupt(inst: u32) -> Operation {
    Operation::SoftwareInterrupt {
        immediate: inst.bits(0..24),
    }
}

fn status_register(inst: u32) -> Operation {
    use instruction::StatusRegisterOperation::*;
    let operation = if inst.bit(21) { Msr } else { Mrs };

    Operation::StatusRegister {
        operation: operation,
        r: inst.bit(22),
        f: inst.bit(19),
        s: inst.bit(18),
        x: inst.bit(17),
        c: inst.bit(16),
        rd: Register(inst.bits(12..16)),
        address: AddressingMode1 {
            i: inst.bit(25),
            operand: inst.bits(0..12),
        }
    }
}
