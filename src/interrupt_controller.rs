use bus;
use bit::SetBit;

pub struct InterruptController {
    enabled: bool,
    asserted: bool,
    lines_enabled: u16,
    lines_asserted: u16,
}

impl InterruptController {
    pub fn new() -> InterruptController {
        InterruptController {
            enabled: false,
            asserted: false,
            lines_enabled: 0,
            lines_asserted: 0,
        }
    }

    pub fn is_asserted(&self) -> bool {
        self.asserted
    }

    pub fn assert(&mut self, line: u8) {
        self.asserted = true;
        self.lines_asserted.set_bit(line, true);
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
            0x200 => self.lines_enabled,
            0x202 => self.lines_asserted,
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
            0x200 => self.lines_enabled = value,
            0x202 => self.lines_asserted = value,
            0x208 => {
                if value == 0 {
                    self.enabled = false;
                } else if value == 1 {
                    self.enabled = true;
                } else {
                    panic!("Attempted to write non-boolean value to IME");
                }
            },
            _ => unreachable!(),
        }
    }

    fn write_word(&mut self, _address: u32, _value: u32) {
        unimplemented!();
    }
}

