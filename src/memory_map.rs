use bit::SetBits;
use bus::{Read, Write};
use interrupt_controller::InterruptController;
use std::cell::RefCell;
use std::fs::File;
use std::io::*;
use std::rc::Rc;

pub struct MemoryMap {
    bios: RefCell<BufReader<File>>,
    rom: RefCell<BufReader<File>>,
    interrupts: Rc<RefCell<InterruptController>>,
    misc: MiscRegisters,
}

impl MemoryMap {
    pub fn new(
        bios: BufReader<File>,
        rom: BufReader<File>,
        interrupts: Rc<RefCell<InterruptController>>
    ) -> MemoryMap {
        MemoryMap {
            bios: RefCell::new(bios),
            rom: RefCell::new(rom),
            interrupts: interrupts,
            misc: MiscRegisters::new(),
        }
    }

    // fn map_read<'a, T>(&'a self, address: u32, read: fn(&'a Read, u32) -> T) -> T {
    fn map_read(&self, address: u32) -> (&Read, u32) {
        let offset = address & 0xFFFFFF;
        match address {
            // General Internal Memory
            0x0000000...0x0003FFF => (&self.bios, offset),
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
            0x4000200...0x4000209 => (&self.interrupts, offset),
            0x4000300...0x4000803 => (&self.misc, offset),
            // Internal Display Memory
            0x5000000...0x50003FF => panic!("BG/OBJ Palette RAM"),
            0x6000000...0x6017FFF => panic!("VRAM - Video RAM"),
            0x7000000...0x70003FF => panic!("OAM - OBJ Attributes"),
            // External Memory (Game Pak)
            0x7ffffff...0x9FFFFFF => (&self.rom, offset),
            0xA000000...0xBFFFFFF => (&self.rom, offset),
            0xC000000...0xDFFFFFF => (&self.rom, offset),
            0xE000000...0xE00FFFF => panic!("Game Pak SRAM"),
            _ => panic!("Memory address {:#x} is unreadable", address),
        }
    }

    fn map_write(&mut self, address: u32) -> (&mut Write, u32) {
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
            0x4000200...0x4000209 => (&mut self.interrupts, offset),
            0x4000300...0x4000803 => (&mut self.misc, offset),
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

impl Read for MemoryMap {
    fn read_byte(&self, address: u32) -> u8 {
        let (device, offset) = self.map_read(address);
        device.read_byte(offset)
    }

    fn read_halfword(&self, address: u32) -> u16 {
        let (device, offset) = self.map_read(address);
        device.read_halfword(offset)
    }

    fn read_word(&self, address: u32) -> u32 {
        let (device, offset) = self.map_read(address);
        device.read_word(offset)
    }
}

impl Write for MemoryMap {
    fn write_byte(&mut self, address: u32, value: u8) {
        let (device, offset) = self.map_write(address);
        device.write_byte(offset, value);
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        let (device, offset) = self.map_write(address);
        device.write_halfword(offset, value);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        let (device, offset) = self.map_write(address);
        device.write_word(offset, value);
    }
}

struct MiscRegisters {
    postflg: bool,
    haltcnt: bool,
    memcnt: u32,
}

impl MiscRegisters {
    fn new() -> MiscRegisters {
        MiscRegisters { postflg: false, haltcnt: false, memcnt: 0 }
    }
}

impl Read for MiscRegisters {
    fn read_byte(&self, address: u32) -> u8 {
        match address {
            0x300 => self.postflg as u8,
            0x301 => self.haltcnt as u8,
            0x800 => self.memcnt as u8,
            0x801 => (self.memcnt >> 8) as u8,
            0x802 => (self.memcnt >> 16) as u8,
            0x803 => (self.memcnt >> 24) as u8,
            _ => unreachable!(),
        }
    }

    fn read_halfword(&self, address: u32) -> u16 {
        match address {
            0x300 => self.postflg as u16,
            0x301 => self.haltcnt as u16,
            0x800 => self.memcnt as u16,
            0x802 => (self.memcnt >> 16) as u16,
            _ => unreachable!(),
        }
    }

    fn read_word(&self, address: u32) -> u32 {
        match address {
            0x300 => self.postflg as u32,
            0x301 => self.haltcnt as u32,
            0x800 => self.memcnt,
            _ => unreachable!(),
        }
    }
}

impl Write for MiscRegisters {
    fn write_byte(&mut self, address: u32, value: u8) {
        match address {
            0x300 if value == 0 => self.postflg = false,
            0x300 if value == 1 => self.postflg = true,
            0x300 => panic!("Attempted to write non-boolean value to POSTFLG"),
            0x301 if value == 0 => self.haltcnt = false,
            0x301 if value == 1 => self.haltcnt = true,
            0x301 => panic!("Attempted to write non-boolean value to HALTCNT"),
            0x800 => self.memcnt.set_bits(0..8, value as u32),
            0x801 => self.memcnt.set_bits(8..16, value as u32),
            0x802 => self.memcnt.set_bits(16..24, value as u32),
            0x803 => self.memcnt.set_bits(24..32, value as u32),
            _ => unreachable!(),
        }
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        match address {
            0x300 if value == 0 => self.postflg = false,
            0x300 if value == 1 => self.postflg = true,
            0x300 => panic!("Attempted to write non-boolean value to POSTFLG"),
            0x301 if value == 0 => self.haltcnt = false,
            0x301 if value == 1 => self.haltcnt = true,
            0x301 => panic!("Attempted to write non-boolean value to HALTCNT"),
            0x800 => self.memcnt.set_bits(0..16, value as u32),
            0x802 => self.memcnt.set_bits(16..32, value as u32),
            _ => unreachable!(),
        }
    }

    fn write_word(&mut self, address: u32, value: u32) {
        match address {
            0x300 if value == 0 => self.postflg = false,
            0x300 if value == 1 => self.postflg = true,
            0x300 => panic!("Attempted to write non-boolean value to POSTFLG"),
            0x301 if value == 0 => self.haltcnt = false,
            0x301 if value == 1 => self.haltcnt = true,
            0x301 => panic!("Attempted to write non-boolean value to HALTCNT"),
            0x800 => self.memcnt = value,
            _ => unreachable!(),
        }
    }
}
