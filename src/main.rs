#![feature(box_syntax)]

extern crate byteorder;
extern crate core;

mod bit;
mod bus;
mod cpu;
mod decode;
mod execute;
mod instruction;
mod memory;
mod mmu;

use cpu::Cpu;
use mmu::Mmu;
use std::fs::File;
use std::io::*;

fn main() {
    let rom = BufReader::new(File::open("./misc/super-mario.gba").unwrap());
    let bios = BufReader::new(File::open("./misc/bios.gba").unwrap());
    let memory = Mmu::new(bios, rom);
    let mut cpu = Cpu::new(memory);
    loop { cpu.tick(); }
}
