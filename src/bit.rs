use std::ops::{ BitAnd, BitAndAssign, BitOrAssign, Not, Shl, Shr, Sub };

/// Get the bit at a given offset.
///
/// `offset` is 0-indexed from the least significant (right most) bits.
///
/// # Examples:
///
/// ```
/// let binary = 0b11001;
/// assert_eq!(binary.bit_at(0), true);
/// assert_eq!(binary.bit_at(1), false);
/// ```
pub trait BitAt
    where Self: BitAnd<Output=Self> + One + PartialEq + Shl<u8, Output=Self>
                + Sized + Zero {
    fn bit_at(self, offset: u8) -> bool {
        self & (Self::one() << offset) != Self::zero()
    }
}

impl BitAt for u32 { }
impl BitAt for u16 { }
impl BitAt for u8 { }

/// Get the bits at a given offset.
///
/// `offset` is 0-indexed from the least significant (right most) bits.
/// `length` is the number of bits to read.
///
/// # Examples:
///
/// ```
/// let binary = 0b11001;
/// assert_eq!(binary.bits_at(0, 2), 0b01);
/// assert_eq!(binary.bit_at(2, 3), 0b110);
/// ```
pub trait BitsAt
    where Self: BitAnd<Output=Self> + One + Shl<u8, Output=Self>
                + Shr<u8, Output=Self> + Sized + Sub<Output=Self> {
    fn bits_at(self, offset: u8, length: u8) -> Self {
        let mask = (Self::one() << length) - Self::one();
        (self & (mask << offset)) >> offset
    }
}

impl BitsAt for u32 { }
impl BitsAt for u16 { }
impl BitsAt for u8 { }

/// Sets the bit at a given offset.
///
/// `offset` is 0-indexed from the least significant (right most) bits.
/// `value` is a boolean where `true` sets the bit, `false` removes the bit.
///
/// # Examples:
///
/// ```
/// let mut binary = 0b11001;
/// binary.set_bit(0, false);
/// binary.set_bit(1, true);
/// assert_eq!(binary, 0b11010);
/// ```
pub trait SetBit
    where Self: BitAndAssign + BitOrAssign + Not<Output=Self> + One
                + Shl<u8, Output=Self> + Sized {
    fn set_bit(mut self, offset: u8, value: bool) {
        if value {
            self |= Self::one() << offset;
        } else {
            self &= !(Self::one() << offset);
        }
    }
}

impl SetBit for u32 { }
impl SetBit for u16 { }
impl SetBit for u8 { }

/// Sets the bits at a given offset.
///
/// `offset` is 0-indexed from the least significant (right most) bits.
/// `length` is the number of bits to replace.
/// `value` are the bits to be written.
///
/// # Examples:
///
/// ```
/// let mut binary = 0b11001;
/// binary.set_bits(0, 2, 0b11);
/// binary.set_bits(2, 3, 0b010);
/// assert_eq!(binary, 0b01011);
/// ```
pub trait SetBits
    where Self: BitAndAssign + BitOrAssign + Not<Output=Self> + One
                + Shl<u8, Output=Self> + Sized + Sub<Output=Self> {
    fn set_bits(mut self, offset: u8, length: u8, value: Self) {
        let mask = (Self::one() << length) - Self::one();
        self &= !mask;
        self |= value << offset;
    }
}

impl SetBits for u32 { }
impl SetBits for u16 { }
impl SetBits for u8 { }

// Bit functions are implemented as default methods on traits so the type of
// the underlying value is unknown. In order to use the integers 0 and 1 we
// define and implement the following two traits:

pub trait One where Self: Sized {
    fn one() -> Self;
}

pub trait Zero where Self: Sized {
    fn zero() -> Self;
}

impl One for u32 { fn one() -> u32 { 1 } }
impl One for u16 { fn one() -> u16 { 1 } }
impl One for u8 { fn one() -> u8 { 1 } }
impl Zero for u32 { fn zero() -> u32 { 0 } }
impl Zero for u16 { fn zero() -> u16 { 0 } }
impl Zero for u8 { fn zero() -> u8 { 0 } }
