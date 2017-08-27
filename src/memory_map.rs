use bus;
use interrupt_controller::InterruptController;
use std::cell::RefCell;
use std::fs::File;
use std::io::*;
use std::rc::Rc;

pub struct MemoryMap {
    bios: RefCell<BufReader<File>>,
    rom: RefCell<BufReader<File>>,
    irc: Rc<RefCell<InterruptController>>,
}

impl MemoryMap {
    pub fn new(
        bios: BufReader<File>,
        rom: BufReader<File>,
        irc: Rc<RefCell<InterruptController>>
    ) -> MemoryMap {
        MemoryMap {
            bios: RefCell::new(bios),
            rom: RefCell::new(rom),
            irc: irc,
        }
    }

    fn map_read<'a, T>(&'a self, address: u32, read: fn(&'a bus::Read, u32) -> T) -> T {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x00000000...0x00003FFF => read(&self.bios, offset),
            0x02000000...0x0203FFFF => panic!("WRAM - On-board Work RAM"),
            0x03000000...0x03007FFF => panic!("WRAM - On-chip Work RAM"),
            // I/O Map
            0x04000000...0x04000056 => panic!("LCD I/O Registers"),
            0x04000060...0x040000A8 => panic!("Sound Registers"),
            0x040000B0...0x040000E0 => panic!("DMA Transfer Channels"),
            0x04000100...0x04000110 => panic!("Timer Registers"),
            0x04000120...0x0400012C => panic!("Serial Communication (1)"),
            0x04000130...0x04000132 => panic!("Keypad Input"),
            0x04000134...0x0400015A => panic!("Serial Communication (2)"),
            0x04000200...0x04000209 => read(&*self.irc, offset),
            0x04000210...0x040003FE => panic!("Interrupt, Waitstate, and Power-Down Control"),
            // Internal Display Memory
            0x05000000...0x050003FF => panic!("BG/OBJ Palette RAM"),
            0x06000000...0x06017FFF => panic!("VRAM - Video RAM"),
            0x07000000...0x070003FF => panic!("OAM - OBJ Attributes"),
            // External Memory (Game Pak)
            0x07ffffff...0x09FFFFFF => read(&self.rom, offset),
            0x0A000000...0x0BFFFFFF => read(&self.rom, offset),
            0x0C000000...0x0DFFFFFF => read(&self.rom, offset),
            0x0E000000...0x0E00FFFF => panic!("Game Pak SRAM"),
            _ => panic!("Memory address {:#x} is unreadable", address),
        }
    }

    fn map_write<'a, T>(&'a mut self, address: u32, value: T, write: fn(&'a mut bus::Write, u32, T)) {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x02000000...0x0203FFFF => panic!("WRAM - On-board Work RAM"),
            0x03000000...0x03007FFF => panic!("WRAM - On-chip Work RAM"),
            // I/O Map
            0x04000000...0x04000056 => panic!("LCD I/O Registers"),
            0x04000060...0x040000A8 => panic!("Sound Registers"),
            0x040000B0...0x040000E0 => panic!("DMA Transfer Channels"),
            0x04000100...0x04000110 => panic!("Timer Registers"),
            0x04000120...0x0400012C => panic!("Serial Communication (1)"),
            0x04000130...0x04000132 => panic!("Keypad Input"),
            0x04000134...0x0400015A => panic!("Serial Communication (2)"),
            0x04000200...0x040003FE => panic!("Interrupt, Waitstate, and Power-Down Control"),
            // Internal Display Memory
            0x05000000...0x050003FF => panic!("BG/OBJ Palette RAM"),
            0x06000000...0x06017FFF => panic!("VRAM - Video RAM"),
            0x07000000...0x070003FF => panic!("OAM - OBJ Attributes"),
            // External Memory (Game Pak)
            0x0E000000...0x0E00FFFF => panic!("Game Pak SRAM"),
            _ => panic!("Memory address {:#x} is unwritable", address),
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

