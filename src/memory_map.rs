use bus;
use memory::Memory;
use std::cell::RefCell;
use std::fs::File;
use std::io::*;

pub struct MemoryMap {
    bios: RefCell<BufReader<File>>,
    rom: RefCell<BufReader<File>>,
    raw_memory: Memory,
}

impl MemoryMap {
    pub fn new(bios: BufReader<File>, rom: BufReader<File>) -> MemoryMap {
        MemoryMap {
            bios: RefCell::new(bios),
            rom: RefCell::new(rom),
            raw_memory: Memory::new(),
        }
    }

    fn map_read<'a, T>(&'a self, address: u32, read: fn(&'a bus::Read, u32) -> T) -> T {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x00000000...0x00003FFF => read(&self.bios, offset), // BIOS
            0x00004000...0x01FFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            0x02000000...0x0203FFFF => panic!("WRAM - On-board Work RAM"),
            0x02040000...0x02FFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            0x03000000...0x03007FFF => panic!("WRAM - On-chip Work RAM"),
            0x03008000...0x03FFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            0x04000000...0x040003FE => panic!("I/O Registers"), // I/O Registers
            0x04000400...0x04FFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            // Internal Display Memory
            0x05000000...0x050003FF => panic!("BG/OBJ Palette RAM"),
            0x05000400...0x05FFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            0x06000000...0x06017FFF => panic!("VRAM - Video RAM"),
            0x06018000...0x06FFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            0x07000000...0x070003FF => panic!("OAM - OBJ Attributes"),
            0x07000400...0x07FFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            // External Memory (Game Pak)
            0x07ffffff...0x09FFFFFF => read(&self.rom, offset), // ROM
            0x0A000000...0x0BFFFFFF => read(&self.rom, offset), // ROM
            0x0C000000...0x0DFFFFFF => read(&self.rom, offset), // ROM
            0x0E000000...0x0E00FFFF => panic!("Game Pak SRAM"),
            0x0E010000...0x0FFFFFFF => panic!("Memory address {:x} is unreadable", address), // Unused
            // Unused Memory Area
            _ /* 0x10000000...0xFFFFFFFF */ => panic!("Memory address {:x} is unreadable", address),
        }
    }

    fn map_write<'a, T>(&'a mut self, address: u32, value: T, write: fn(&'a mut bus::Write, u32, T)) {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x00000000...0x00003FFF => panic!("Memory address {:x} is unwritable", address), // BIOS
            0x00004000...0x01FFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            0x02000000...0x0203FFFF => panic!("WRAM - On-board Work RAM"),
            0x02040000...0x02FFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            0x03000000...0x03007FFF => panic!("WRAM - On-chip Work RAM"),
            0x03008000...0x03FFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            0x04000000...0x040003FE => panic!("I/O Registers"), // I/O Registers
            0x04000400...0x04FFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            // Internal Display Memory
            0x05000000...0x050003FF => panic!("BG/OBJ Palette RAM"),
            0x05000400...0x05FFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            0x06000000...0x06017FFF => panic!("VRAM - Video RAM"),
            0x06018000...0x06FFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            0x07000000...0x070003FF => panic!("OAM - OBJ Attributes"),
            0x07000400...0x07FFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            // External Memory (Game Pak)
            0x08000000...0x09FFFFFF => panic!("Memory address {:x} is unwritable", address), // ROM
            0x0A000000...0x0BFFFFFF => panic!("Memory address {:x} is unwritable", address), // ROM
            0x0C000000...0x0DFFFFFF => panic!("Memory address {:x} is unwritable", address), // ROM
            0x0E000000...0x0E00FFFF => panic!("Game Pak SRAM"),
            0x0E010000...0x0FFFFFFF => panic!("Memory address {:x} is unwritable", address), // Unused
            // Unused Memory Area
            _ /* 0x10000000...0xFFFFFFFF */ => panic!("Memory address {:x} is unwritable", address),
        }
    }
}

impl bus::Read for MemoryMap {
    fn read_byte(&self, address: u32) -> u8 {
        self.map_read(address, bus::Read::read_byte)
    }

    fn read_halfword(&self, address: u32) -> u16 {
        self.map_read(address, bus::Read::read_halfword)
    }

    fn read_word(&self, address: u32) -> u32 {
        self.map_read(address, bus::Read::read_word)
    }
}

impl bus::Write for MemoryMap {
    fn write_byte(&mut self, address: u32, value: u8) {
        self.map_write(address, value, bus::Write::write_byte)
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        self.map_write(address, value, bus::Write::write_halfword)
    }

    fn write_word(&mut self, address: u32, value: u32) {
        self.map_write(address, value, bus::Write::write_word)
    }
}

