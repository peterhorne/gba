use bus;

pub struct Memory(Box<[u8; 268_435_456]>);

impl Memory {
    pub fn new() -> Memory {
        Memory(box [0; 268_435_456])
    }
}

impl bus::Read for Memory {
    fn read_byte(&mut self, address: u32) -> u8 {
        println!("Memory#read {:x}", address);
        self.0[address as usize]
    }

    fn read_halfword(&mut self, address: u32) -> u16 {
        println!("Memory#read {:x}", address);
        (self.0[address as usize] as u16)
            + ((self.0[(address + 1) as usize] as u16) << 8)
    }

    fn read_word(&mut self, address: u32) -> u32 {
        println!("Memory#read {:x}", address);
        (self.0[address as usize] as u32)
            + ((self.0[(address + 1) as usize] as u32) << 8)
            + ((self.0[(address + 2) as usize] as u32) << 16)
            + ((self.0[(address + 3) as usize] as u32) << 24)
    }
}

impl bus::Write for Memory {
    fn write_byte(&mut self, address: u32, value: u8) {
        println!("Memory#write {:x} = {:x}", address, value);
        self.0[address as usize] = value;
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        println!("Memory#write {:x} = {:x}", address, value);
        self.0[address as usize] = value as u8;
        self.0[(address + 1) as usize] = (value >> 8) as u8;
    }

    fn write_word(&mut self, address: u32, value: u32) {
        println!("Memory#write {:x} = {:x}", address, value);
        self.0[address as usize] = value as u8;
        self.0[(address + 1) as usize] = (value >> 8) as u8;
        self.0[(address + 2) as usize] = (value >> 16) as u8;
        self.0[(address + 3) as usize] = (value >> 24) as u8;
    }
}
