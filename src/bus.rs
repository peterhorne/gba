use byteorder::{ReadBytesExt, LittleEndian};
use std::cell::RefCell;
use std::io;

pub trait Read {
    fn read_byte(&self, address: u32) -> u8;
    fn read_halfword(&self, address: u32) -> u16;
    fn read_word(&self, address: u32) -> u32;
}

pub trait Write {
    fn write_byte(&mut self, address: u32, value: u8);
    fn write_halfword(&mut self, address: u32, value: u16);
    fn write_word(&mut self, address: u32, value: u32);
}

impl<T: io::Read + io::Seek> Read for RefCell<T> {
    fn read_byte(&self, address: u32) -> u8 {
        let mut inner = self.borrow_mut();
        inner.seek(io::SeekFrom::Start(address as u64)).unwrap();
        inner.read_u8().unwrap()
    }

    fn read_halfword(&self, address: u32) -> u16 {
        let mut inner = self.borrow_mut();
        inner.seek(io::SeekFrom::Start(address as u64)).unwrap();
        inner.read_u16::<LittleEndian>().unwrap()
    }

    fn read_word(&self, address: u32) -> u32 {
        let mut inner = self.borrow_mut();
        inner.seek(io::SeekFrom::Start(address as u64)).unwrap();
        inner.read_u32::<LittleEndian>().unwrap()
    }
}
