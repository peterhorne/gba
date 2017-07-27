use bit::{Bit, Bits};
use cpu::Register;
use instruction::{
    AddressMode1,
    AddressMode2,
    AddressMode3,
    Condition,
    Instruction,
    AddressingOffset,
    AddressingMode,
    ShiftDirection,
};

pub fn decode_arm(inst: u32) -> Instruction {
    let bits = (
        inst.bit(27) as u8, inst.bit(26) as u8, inst.bit(25) as u8,
        inst.bit(24) as u8, inst.bit(23) as u8, inst.bit(22) as u8,
        inst.bit(21) as u8, inst.bit(20) as u8, inst.bit(7) as u8,
        inst.bit(6) as u8, inst.bit(5) as u8, inst.bit(4) as u8,
    );

    match bits {
        (0,0,0,0,0,0,0,_, 1,0,0,1) => { mul(inst) },
        (0,0,0,0,0,0,1,_, 1,0,0,1) => { mla(inst) },
        (0,0,0,0,1,0,0,_, 1,0,0,1) => { umull(inst) },
        (0,0,0,0,1,0,1,_, 1,0,0,1) => { umlal(inst) },
        (0,0,0,0,1,1,0,_, 1,0,0,1) => { smull(inst) },
        (0,0,0,0,1,1,1,_, 1,0,0,1) => { smlal(inst) },
        (0,0,0,1,0,_,0,0, 0,0,0,0) => { mrs(inst) },
        (0,0,1,1,0,_,1,0, _,_,_,_) => { msr(inst) },
        (0,0,0,1,0,_,1,0, 0,0,0,0) => { msr(inst) },
        (0,0,0,1,0,0,1,0, 0,0,0,1) => { bx(inst) },
        (0,0,0,1,0,0,0,0, 1,0,0,1) => { swp(inst) },
        (0,0,0,1,0,1,0,0, 1,0,0,1) => { swpb(inst) },
        (0,0,0,_,_,_,_,1, 1,0,1,1) => { ldrh(inst) },
        (0,0,0,_,_,_,_,0, 1,0,1,1) => { strh(inst) },
        (0,0,0,_,_,_,_,1, 1,1,0,1) => { ldrsb(inst) },
        (0,0,0,_,_,_,_,1, 1,1,1,1) => { ldrsh(inst) },
        (0,0,_,0,0,0,0,_, _,_,_,_) => { and(inst) },
        (0,0,_,0,0,0,1,_, _,_,_,_) => { eor(inst) },
        (0,0,_,0,0,1,0,_, _,_,_,_) => { sub(inst) },
        (0,0,_,0,0,1,1,_, _,_,_,_) => { rsb(inst) },
        (0,0,_,0,1,0,0,_, _,_,_,_) => { add(inst) },
        (0,0,_,0,1,0,1,_, _,_,_,_) => { adc(inst) },
        (0,0,_,0,1,1,0,_, _,_,_,_) => { sbc(inst) },
        (0,0,_,0,1,1,1,_, _,_,_,_) => { rsc(inst) },
        (0,0,_,1,1,0,0,_, _,_,_,_) => { orr(inst) },
        (0,0,_,1,1,0,1,_, _,_,_,_) => { mov(inst) },
        (0,0,_,1,1,1,0,_, _,_,_,_) => { bic(inst) },
        (0,0,_,1,1,1,1,_, _,_,_,_) => { mvn(inst) },
        (0,0,_,1,0,0,0,1, _,_,_,_) => { tst(inst) },
        (0,0,_,1,0,0,1,1, _,_,_,_) => { teq(inst) },
        (0,0,_,1,0,1,0,1, _,_,_,_) => { cmp(inst) },
        (0,0,_,1,0,1,1,1, _,_,_,_) => { cmn(inst) },
        (0,1,_,_,_,0,_,1, _,_,_,_) => { ldr(inst) },
        (0,1,_,_,_,1,_,1, _,_,_,_) => { ldrb(inst) },
        (0,1,_,_,_,0,_,0, _,_,_,_) => { str(inst) },
        (0,1,_,_,_,1,_,0, _,_,_,_) => { strb(inst) },
        (0,1,_,0,_,1,1,1, _,_,_,_) => { ldrbt(inst) },
        (0,1,_,0,_,0,1,1, _,_,_,_) => { ldrt(inst) },
        (0,1,_,0,_,1,1,0, _,_,_,_) => { strbt(inst) },
        (0,1,_,0,_,0,1,0, _,_,_,_) => { strt(inst) },
        (1,0,0,_,_,0,_,1, _,_,_,_) => { ldm1(inst) },
        (1,0,0,_,_,1,_,1, _,_,_,_) => { ldm3(inst) },
        (1,0,0,_,_,0,_,0, _,_,_,_) => { stm1(inst) },
        (1,0,0,_,_,1,0,1, _,_,_,_) => { ldm2(inst) },
        (1,0,0,_,_,1,0,0, _,_,_,_) => { stm2(inst) },
        (1,0,1,_,_,_,_,_, _,_,_,_) => { b(inst) },
        (1,1,0,_,_,_,_,1, _,_,_,_) => { ldc(inst) },
        (1,1,0,_,_,_,_,0, _,_,_,_) => { stc(inst) },
        (1,1,1,0,_,_,_,1, _,_,_,0) => { cdp(inst) },
        (1,1,1,0,_,_,_,0, _,_,_,1) => { mcr(inst) },
        (1,1,1,0,_,_,_,1, _,_,_,1) => { mrc(inst) },
        (1,1,1,1,_,_,_,_, _,_,_,_) => { swi(inst) },
        _ => { panic!("Unrecognised instruction: {:x}", inst); },
    }
}

fn condition(inst: u32) -> Condition {
    match inst.bits(28..32) {
        0b0000 => { Condition::Eq },
        0b0001 => { Condition::Ne },
        0b0010 => { Condition::Cs },
        0b0011 => { Condition::Cc },
        0b0100 => { Condition::Mi },
        0b0101 => { Condition::Pl },
        0b0110 => { Condition::Vs },
        0b0111 => { Condition::Vc },
        0b1000 => { Condition::Hi },
        0b1001 => { Condition::Ls },
        0b1010 => { Condition::Ge },
        0b1011 => { Condition::Lt },
        0b1100 => { Condition::Gt },
        0b1101 => { Condition::Le },
        0b1110 => { Condition::Al },
        0b1111 => { Condition::Nv },
        _      => { unreachable!() },
    }
}

fn mul(inst: u32) -> Instruction {
    Instruction::Mul {
        condition: condition(inst),
        s: inst.bit(20),
        rd: Register(inst.bits(16..20)),
        rm: Register(inst.bits(0..4)),
        rs: Register(inst.bits(8..12)),
    }
}

fn mla(inst: u32) -> Instruction {
    Instruction::Mla {
        condition: condition(inst),
        s: inst.bit(20),
        rd: Register(inst.bits(16..20)),
        rn: Register(inst.bits(12..16)),
        rm: Register(inst.bits(0..4)),
        rs: Register(inst.bits(8..12)),
    }
}

fn umull(inst: u32) -> Instruction {
    Instruction::Umull {
        condition: condition(inst),
        s: inst.bit(20),
        rd_hi: Register(inst.bits(16..20)),
        rd_lo: Register(inst.bits(12..16)),
        rs: Register(inst.bits(8..12)),
        rm: Register(inst.bits(0..4)),
    }
}

fn umlal(inst: u32) -> Instruction {
    Instruction::Umlal {
        condition: condition(inst),
        s: inst.bit(20),
        rd_hi: Register(inst.bits(16..20)),
        rd_lo: Register(inst.bits(12..16)),
        rs: Register(inst.bits(8..12)),
        rm: Register(inst.bits(0..4)),
    }
}

fn smull(inst: u32) -> Instruction {
    Instruction::Smull {
        condition: condition(inst),
        s: inst.bit(20),
        rd_hi: Register(inst.bits(16..20)),
        rd_lo: Register(inst.bits(12..16)),
        rs: Register(inst.bits(8..12)),
        rm: Register(inst.bits(0..4)),
    }
}

fn smlal(inst: u32) -> Instruction {
    Instruction::Smlal {
        condition: condition(inst),
        s: inst.bit(20),
        rd_hi: Register(inst.bits(16..20)),
        rd_lo: Register(inst.bits(12..16)),
        rs: Register(inst.bits(8..12)),
        rm: Register(inst.bits(0..4)),
    }
}

fn mrs(inst: u32) -> Instruction {
    Instruction::Mrs {
        condition: condition(inst),
        r: inst.bit(22),
        rd: Register(inst.bits(12..16)),
    }
}

fn msr(inst: u32) -> Instruction {
    Instruction::Msr {
        condition: condition(inst),
        r: inst.bit(22),
        f: inst.bit(19),
        s: inst.bit(18),
        x: inst.bit(17),
        c: inst.bit(16),
        address: decode_address_mode_1(inst),
    }
}

fn bx(inst: u32) -> Instruction {
    Instruction::Bx {
        condition: condition(inst),
        rm: Register(inst.bits(0..4)),
    }
}

fn swp(inst: u32) -> Instruction {
    Instruction::Swp {
        condition: condition(inst),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        rm: Register(inst.bits(0..4)),
    }
}

fn swpb(inst: u32) -> Instruction {
    Instruction::Swpb {
        condition: condition(inst),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        rm: Register(inst.bits(0..4)),
    }
}

fn ldrh(inst: u32) -> Instruction {
    Instruction::Ldrh {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_3(inst),
    }
}

fn strh(inst: u32) -> Instruction {
    Instruction::Strh {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_3(inst),
    }
}

fn ldrsb(inst: u32) -> Instruction {
    Instruction::Ldrsb {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_3(inst),
    }
}

fn ldrsh(inst: u32) -> Instruction {
    Instruction::Ldrsh {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_3(inst),
    }
}

fn and(inst: u32) -> Instruction {
    Instruction::And {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn eor(inst: u32) -> Instruction {
    Instruction::Eor {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn sub(inst: u32) -> Instruction {
    Instruction::Sub {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn rsb(inst: u32) -> Instruction {
    Instruction::Rsb {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn add(inst: u32) -> Instruction {
    Instruction::Add {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn adc(inst: u32) -> Instruction {
    Instruction::Adc {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn sbc(inst: u32) -> Instruction {
    Instruction::Sbc {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn rsc(inst: u32) -> Instruction {
    Instruction::Rsc {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn orr(inst: u32) -> Instruction {
    Instruction::Orr {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn mov(inst: u32) -> Instruction {
    Instruction::Mov {
        condition: condition(inst),
        s: inst.bit(20),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn bic(inst: u32) -> Instruction {
    Instruction::Bic {
        condition: condition(inst),
        s: inst.bit(20),
        rn: Register(inst.bits(16..20)),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn mvn(inst: u32) -> Instruction {
    Instruction::Mvn {
        condition: condition(inst),
        s: inst.bit(20),
        rd: Register(inst.bits(12..16)),
        operand2: decode_address_mode_1(inst),
    }
}

fn tst(inst: u32) -> Instruction {
    Instruction::Tst {
        condition: condition(inst),
        rn: Register(inst.bits(16..20)),
        operand2: decode_address_mode_1(inst),
    }
}

fn teq(inst: u32) -> Instruction {
    Instruction::Teq {
        condition: condition(inst),
        rn: Register(inst.bits(16..20)),
        operand2: decode_address_mode_1(inst),
    }
}

fn cmp(inst: u32) -> Instruction {
    Instruction::Cmp {
        condition: condition(inst),
        rn: Register(inst.bits(16..20)),
        operand2: decode_address_mode_1(inst),
    }
}

fn cmn(inst: u32) -> Instruction {
    Instruction::Cmn {
        condition: condition(inst),
        rn: Register(inst.bits(16..20)),
        operand2: decode_address_mode_1(inst),
    }
}

fn ldr(inst: u32) -> Instruction {
    Instruction::Ldr {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn ldrb(inst: u32) -> Instruction {
    Instruction::Ldrb {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn str(inst: u32) -> Instruction {
    Instruction::Str {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn strb(inst: u32) -> Instruction {
    Instruction::Strb {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn ldrbt(inst: u32) -> Instruction {
    Instruction::Ldrbt {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn ldrt(inst: u32) -> Instruction {
    Instruction::Ldrt {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn strbt(inst: u32) -> Instruction {
    Instruction::Strbt {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn strt(inst: u32) -> Instruction {
    Instruction::Strt {
        condition: condition(inst),
        rd: Register(inst.bits(12..16)),
        address: decode_address_mode_2(inst),
    }
}

fn ldm1(inst: u32) -> Instruction {
    Instruction::Ldm1 {
        condition: condition(inst),
    }
}

fn ldm3(inst: u32) -> Instruction {
    Instruction::Ldm3 {
        condition: condition(inst),
    }
}

fn stm1(inst: u32) -> Instruction {
    Instruction::Stm1 {
        condition: condition(inst),
    }
}

fn ldm2(inst: u32) -> Instruction {
    Instruction::Ldm2 {
        condition: condition(inst),
    }
}

fn stm2(inst: u32) -> Instruction {
    Instruction::Stm2 {
        condition: condition(inst),
    }
}

fn b(inst: u32) -> Instruction {
    Instruction::B {
        condition: condition(inst),
        l: inst.bit(24),
        signed_immed: inst.bits(0..24),
    }
}

fn ldc(inst: u32) -> Instruction {
    Instruction::Ldc {
        condition: condition(inst),
    }
}

fn stc(inst: u32) -> Instruction {
    Instruction::Stc {
        condition: condition(inst),
    }
}

fn cdp(inst: u32) -> Instruction {
    Instruction::Cdp {
        condition: condition(inst),
    }
}

fn mcr(inst: u32) -> Instruction {
    Instruction::Mcr {
        condition: condition(inst),
    }
}

fn mrc(inst: u32) -> Instruction {
    Instruction::Mrc {
        condition: condition(inst),
    }
}

fn swi(inst: u32) -> Instruction {
    Instruction::Swi {
        condition: condition(inst),
        immediate: inst.bits(0..24),
    }
}

fn decode_address_mode_1(inst: u32) -> AddressMode1 {
    if inst.bit(25) {
        AddressMode1::Immediate {
            value: inst.bits(0..8) as u8,
            rotate: inst.bits(8..12) as u8,
        }
    } else {
        AddressMode1::Shift {
            rm: Register(inst.bits(0..4)),
            shift: match inst.bits(5..7) {
                0b00 => { ShiftDirection::Lsl },
                0b01 => { ShiftDirection::Lsr },
                0b10 => { ShiftDirection::Asr },
                0b11 if inst.bits(7..12) == 0 => { ShiftDirection::Rrx },
                0b11 => { ShiftDirection::Ror },
                _ => { unreachable!() },
            },
            shift_imm: if inst.bit(4) {
                AddressingOffset::Register(Register(inst.bits(8..12)))
            } else {
                AddressingOffset::Immediate(inst.bits(7..12) as u16)
            },
        }
    }
}

fn decode_address_mode_2(inst: u32) -> AddressMode2 {
    let i = inst.bit(25);
    let p = inst.bit(24);
    let u = inst.bit(23);
    let w = inst.bit(21);
    let rn = Register(inst.bits(16..20));
    let offset = inst.bits(0..12);
    let shift_imm = inst.bits(7..12);
    let shift = inst.bits(5..7);
    let rm = Register(inst.bits(0..4));

    let offset = if i {
        if inst.bits(4..12) == 0 {
            AddressingOffset::Register(rm)
        } else {
            AddressingOffset::ScaledRegister {
                rm: rm,
                shift: match shift {
                    0b00 => { ShiftDirection::Lsl },
                    0b01 => { ShiftDirection::Lsr },
                    0b10 => { ShiftDirection::Asr },
                    0b11 if shift_imm == 0 => { ShiftDirection::Rrx },
                    0b11 => { ShiftDirection::Ror },
                    _ => { unreachable!() },
                },
                shift_imm: shift_imm as u8,
            }
        }
    } else {
        AddressingOffset::Immediate(offset as u16)
    };

    let addressing =
        if p && w { AddressingMode::PreIndexed }
        else if p && !w { AddressingMode::Offset }
        else if !p && w { panic!("unpredictable") } // TODO: not unpredictable for address mode 2
        else /* !p && !w  */ { AddressingMode::PostIndexed };

    AddressMode2 {
        rn: rn,
        offset: offset,
        addressing: addressing,
        u: u,
    }
}

fn decode_address_mode_3(inst: u32) -> AddressMode3 {
    let p = inst.bit(24);
    let u = inst.bit(23);
    let i = inst.bit(22);
    let w = inst.bit(21);
    let rn = Register(inst.bits(16..20));
    let offset_a = inst.bits(8..12);
    let offset_b = inst.bits(0..4);

    let offset = if i {
        let byte = (offset_a << 4) | offset_b;
        AddressingOffset::Immediate(byte as u16)
    } else {
        let register = Register(offset_b);
        AddressingOffset::Register(register)
    };

    let addressing =
        if p && w { AddressingMode::PreIndexed }
        else if p && !w { AddressingMode::Offset }
        else if !p && w { panic!("unpredictable") }
        else /* !p && !w  */ { AddressingMode::PostIndexed };

    AddressMode3 {
        rn: rn,
        offset: offset,
        addressing: addressing,
        u: u,
    }
}
