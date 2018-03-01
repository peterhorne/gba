pub use self::decode_arm::decode_arm;
pub use self::decode_thumb::decode_thumb;
use instruction::{EncodedInstruction, Instruction};

mod decode_arm;
mod decode_thumb;

pub fn decode(instruction: EncodedInstruction) -> Instruction {
    match instruction {
        EncodedInstruction::Thumb(bits) => decode_thumb(bits),
        EncodedInstruction::Arm(bits) => decode_arm(bits),
    }
}
