use bus;
use bit::{Bit, SetBit};

pub struct InterruptController {
    enabled: bool,
    asserted: bool,
    mask: u16,
    flags: u16,
}

#[derive(Clone)]
pub enum Input {
    VBlank = 0,
    HBlank = 1,
    VCounter = 2,
    Timer0 = 3,
    Timer1 = 4,
    Timer2 = 5,
    Timer3 = 6,
    Serial = 7,
    Dma0 = 8,
    Dma1 = 9,
    Dma2 = 10,
    Dma3 = 11,
    Key = 12,
    GamePak = 13,
}

impl InterruptController {
    pub fn new() -> InterruptController {
        InterruptController {
            enabled: false,
            asserted: false,
            mask: 0,
            flags: 0,
        }
    }

    pub fn is_asserted(&self) -> bool {
        self.asserted
    }

    pub fn assert(&mut self, input: Input) {
        if self.mask.bit(input.clone() as u8) {
            self.asserted = true;
            self.flags.set_bit(input as u8, true);
        }
    }

    pub fn reset(&mut self) {
        self.asserted = false;
    }
}

impl bus::Read for InterruptController {
    fn read_byte(&self, _address: u32) -> u8 {
        unimplemented!();
    }

    fn read_halfword(&self, address: u32) -> u16 {
        match address {
            0x200 => self.mask,
            0x202 => self.flags,
            0x208 => self.enabled as u16,
            _ => unreachable!(),
        }
    }

    fn read_word(&self, _address: u32) -> u32 {
        unimplemented!();
    }
}

impl bus::Write for InterruptController {
    fn write_byte(&mut self, _address: u32, _value: u8) {
        unimplemented!();
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        match address {
            0x200 => self.mask = value & 0x3fff,
            0x202 => self.flags ^= self.flags & value & 0x3fff,
            0x208 if value == 0 => self.enabled = false,
            0x208 if value == 1 => self.enabled = true,
            0x208 => panic!("Attempted to write non-boolean value to IME"),
            _ => unreachable!(),
        }
    }

    fn write_word(&mut self, _address: u32, _value: u32) {
        unimplemented!();
    }
}

