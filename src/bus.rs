use byteorder::{ReadBytesExt, LittleEndian};
use std::cell::RefCell;
use std::fs::File;
use std::io::*;
use std::io;
use std::rc::Rc;

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

impl<T: Read> Read for Rc<RefCell<T>> {
    fn read_byte(&self, address: u32) -> u8 {
        self.borrow().read_byte(address)
    }

    fn read_halfword(&self, address: u32) -> u16 {
        self.borrow().read_halfword(address)
    }

    fn read_word(&self, address: u32) -> u32 {
        self.borrow().read_word(address)
    }
}

impl<T: Write> Write for Rc<RefCell<T>> {
    fn write_byte(&mut self, address: u32, value: u8) {
        self.borrow_mut().write_byte(address, value);
    }

    fn write_halfword(&mut self, address: u32, value: u16) {
        self.borrow_mut().write_halfword(address, value);
    }

    fn write_word(&mut self, address: u32, value: u32) {
        self.borrow_mut().write_word(address, value);
    }
}


impl Read for RefCell<BufReader<File>> {
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
