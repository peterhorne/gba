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
    postflg: [u8; 1],
    haltcnt: [u8; 1],
    memcnt: [u8; 4],
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
            postflg: [0; 1],
            haltcnt: [0; 1],
            memcnt: [0; 4],
        }
    }

    fn map_read<'a, T>(&'a self, address: u32, read: fn(&'a bus::Read, u32) -> T) -> T {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x0000000...0x0003FFF => read(&self.bios, offset),
            0x2000000...0x203FFFF => panic!("WRAM - On-board Work RAM"),
            0x3000000...0x3007FFF => panic!("WRAM - On-chip Work RAM"),
            // I/O Map
            0x4000000...0x4000056 => panic!("LCD I/O Registers"),
            0x4000060...0x40000A8 => panic!("Sound Registers"),
            0x40000B0...0x40000E0 => panic!("DMA Transfer Channels"),
            0x4000100...0x4000110 => panic!("Timer Registers"),
            0x4000120...0x400012C => panic!("Serial Communication (1)"),
            0x4000130...0x4000132 => panic!("Keypad Input"),
            0x4000134...0x400015A => panic!("Serial Communication (2)"),
            0x4000200...0x4000209 => read(&*self.irc, offset),
            0x4000300...0x4000300 => read(&self.postflg, 0),
            0x4000301...0x4000301 => read(&self.haltcnt, 0),
            0x4000800...0x4000803 => read(&self.memcnt, address & 0x4000800),
            // Internal Display Memory
            0x5000000...0x50003FF => panic!("BG/OBJ Palette RAM"),
            0x6000000...0x6017FFF => panic!("VRAM - Video RAM"),
            0x7000000...0x70003FF => panic!("OAM - OBJ Attributes"),
            // External Memory (Game Pak)
            0x7ffffff...0x9FFFFFF => read(&self.rom, offset),
            0xA000000...0xBFFFFFF => read(&self.rom, offset),
            0xC000000...0xDFFFFFF => read(&self.rom, offset),
            0xE000000...0xE00FFFF => panic!("Game Pak SRAM"),
            _ => panic!("Memory address {:#x} is unreadable", address),
        }
    }

    fn map_write<'a, T>(&'a mut self, address: u32, value: T, write: fn(&'a mut bus::Write, u32, T)) {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x2000000...0x203FFFF => panic!("WRAM - On-board Work RAM"),
            0x3000000...0x3007FFF => panic!("WRAM - On-chip Work RAM"),
            // I/O Map
            0x4000000...0x4000056 => panic!("LCD I/O Registers"),
            0x4000060...0x40000A8 => panic!("Sound Registers"),
            0x40000B0...0x40000E0 => panic!("DMA Transfer Channels"),
            0x4000100...0x4000110 => panic!("Timer Registers"),
            0x4000120...0x400012C => panic!("Serial Communication (1)"),
            0x4000130...0x4000132 => panic!("Keypad Input"),
            0x4000134...0x400015A => panic!("Serial Communication (2)"),
            0x4000200...0x4000209 => write(&*self.irc, offset, value),
            0x4000300...0x4000300 => write(self.postflg, 0, value),
            0x4000301...0x4000301 => write(&self.haltcnt, 0, value),
            0x4000800...0x4000803 => write(&self.memcnt, address & 0x4000800, value),
            // Internal Display Memory
            0x5000000...0x50003FF => panic!("BG/OBJ Palette RAM"),
            0x6000000...0x6017FFF => panic!("VRAM - Video RAM"),
            0x7000000...0x70003FF => panic!("OAM - OBJ Attributes"),
            // External Memory (Game Pak)
            0xE000000...0xE00FFFF => panic!("Game Pak SRAM"),
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

