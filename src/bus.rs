use std::io;
use byteorder::{ReadBytesExt, LittleEndian};

pub trait Read {
    fn read_byte(&mut self, address: u32) -> u8;
    fn read_halfword(&mut self, address: u32) -> u16;
    fn read_word(&mut self, address: u32) -> u32;
}

pub trait Write {
    fn write_byte(&mut self, address: u32, value: u8);
    fn write_halfword(&mut self, address: u32, value: u16);
    fn write_word(&mut self, address: u32, value: u32);
}

impl<T: io::Read + io::Seek> Read for T {
    fn read_byte(&mut self, address: u32) -> u8 {
        self.seek(io::SeekFrom::Start(address as u64)).unwrap();
        self.read_u8().unwrap()
    }

    fn read_halfword(&mut self, address: u32) -> u16 {
        self.seek(io::SeekFrom::Start(address as u64)).unwrap();
        self.read_u16::<LittleEndian>().unwrap()
    }

    fn read_word(&mut self, address: u32) -> u32 {
        self.seek(io::SeekFrom::Start(address as u64)).unwrap();
        self.read_u32::<LittleEndian>().unwrap()
    }
}
