use bus;
use memory::Memory;
use std::fs::File;
use std::io::*;

pub struct Mmu {
    bios: BufReader<File>,
    rom: BufReader<File>,
    raw_memory: Memory,
}

impl Mmu {
    pub fn new(bios: BufReader<File>, rom: BufReader<File>) -> Mmu {
        Mmu {
            bios: bios,
            rom: rom,
            raw_memory: Memory::new(),
        }
    }

    fn map_read(&mut self, address: u32) -> Option<(&mut bus::Read, u32)> {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x00000000...0x00003FFF => { Some((&mut self.bios, offset)) }, // BIOS
            0x00004000...0x01FFFFFF => { None }, // Unused
            0x02000000...0x0203FFFF => { panic!("WRAM - On-board Work RAM") },
            0x02040000...0x02FFFFFF => { None }, // Unused
            0x03000000...0x03007FFF => { panic!("WRAM - On-chip Work RAM") },
            0x03008000...0x03FFFFFF => { None }, // Unused
            0x04000000...0x040003FE => { panic!("I/O Registers") }, // I/O Registers
            0x04000400...0x04FFFFFF => { None }, // Unused
            // Internal Display Memory
            0x05000000...0x050003FF => { panic!("BG/OBJ Palette RAM") },
            0x05000400...0x05FFFFFF => { None }, // Unused
            0x06000000...0x06017FFF => { panic!("VRAM - Video RAM") },
            0x06018000...0x06FFFFFF => { None }, // Unused
            0x07000000...0x070003FF => { panic!("OAM - OBJ Attributes") },
            0x07000400...0x07FFFFFF => { None }, // Unused
            // External Memory (Game Pak)
            0x08000000...0x09FFFFFF => { Some((&mut self.rom, offset)) }, // ROM
            0x0A000000...0x0BFFFFFF => { Some((&mut self.rom, offset)) }, // ROM
            0x0C000000...0x0DFFFFFF => { Some((&mut self.rom, offset)) }, // ROM
            0x0E000000...0x0E00FFFF => { panic!("Game Pak SRAM") },
            0x0E010000...0x0FFFFFFF => { None }, // Unused
            // Unused Memory Area
            _ /* 0x10000000...0xFFFFFFFF */ => { None },
        }
    }

    fn map_write(&mut self, address: u32) -> Option<(&mut bus::Write, u32)> {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x00000000...0x00003FFF => { None }, // BIOS
            0x00004000...0x01FFFFFF => { None }, // Unused
            0x02000000...0x0203FFFF => { panic!("WRAM - On-board Work RAM") },
            0x02040000...0x02FFFFFF => { None }, // Unused
            0x03000000...0x03007FFF => { panic!("WRAM - On-chip Work RAM") },
            0x03008000...0x03FFFFFF => { None }, // Unused
            0x04000000...0x040003FE => { panic!("I/O Registers") }, // I/O Registers
            0x04000400...0x04FFFFFF => { None }, // Unused
            // Internal Display Memory
            0x05000000...0x050003FF => { panic!("BG/OBJ Palette RAM") },
            0x05000400...0x05FFFFFF => { None }, // Unused
            0x06000000...0x06017FFF => { panic!("VRAM - Video RAM") },
            0x06018000...0x06FFFFFF => { None }, // Unused
            0x07000000...0x070003FF => { panic!("OAM - OBJ Attributes") },
            0x07000400...0x07FFFFFF => { None }, // Unused
            // External Memory (Game Pak)
            0x08000000...0x09FFFFFF => { None }, // ROM
            0x0A000000...0x0BFFFFFF => { None }, // ROM
            0x0C000000...0x0DFFFFFF => { None }, // ROM
            0x0E000000...0x0E00FFFF => { panic!("Game Pak SRAM") },
            0x0E010000...0x0FFFFFFF => { None }, // Unused
            // Unused Memory Area
            _ /* 0x10000000...0xFFFFFFFF */ => { None },
        }
    }
}

impl bus::Read for Mmu {
    fn read_byte(&mut self, address: u32) -> u8 {
        match self.map_read(address) {
            Some((bus, offset)) => { bus.read_byte(offset) },
            None => { panic!("Memory address '{:x}' unreadadable", address) },
        }
    }

    fn read_halfword(&mut self, address: u32) -> u16 {
        match self.map_read(address) {
            Some((bus, offset)) => { bus.read_halfword(offset) },
            None => { panic!("Memory address '{:x}' unreadadable", address) },
        }
    }

    fn read_word(&mut self, address: u32) -> u32 {
        match self.map_read(address) {
            Some((bus, offset)) => { bus.read_word(offset) },
            None => { panic!("Memory address '{:x}' unreadadable", address) },
        }
    }
}

impl bus::Write for Mmu {
    fn write_byte(&mut self, address: u32, value: u8) {
        match self.map_write(address) {
            Some((bus, offset)) => { bus.write_byte(offset, value) },
            None => { panic!("Memory address '{:x}' unwritable", address) },
        }
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        match self.map_write(address) {
            Some((bus, offset)) => { bus.write_halfword(offset, value) },
            None => { panic!("Memory address '{:x}' unwritable", address) },
        }
    }

    fn write_word(&mut self, address: u32, value: u32) {
        match self.map_write(address) {
            Some((bus, offset)) => { bus.write_word(offset, value) },
            None => { panic!("Memory address '{:x}' unwritable", address) },
        }
    }
}

