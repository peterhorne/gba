#![feature(box_syntax)]

extern crate byteorder;
extern crate core;

mod bit;
mod bus;
mod cpu;
mod decode;
mod execute;
mod instruction;
mod memory_map;

use cpu::Cpu;
use memory_map::MemoryMap;
use std::cell::RefCell;
use std::fs::File;
use std::io::*;
use std::rc::Rc;

fn main() {
    let bios = BufReader::new(File::open("./misc/bios-dump.bin").unwrap());
    let rom = BufReader::new(File::open("./misc/super-mario.gba").unwrap());
    let memory = MemoryMap::new(bios, rom);
    let mut cpu = Cpu::new(memory);
    loop { cpu.tick(); }
}
