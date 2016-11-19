#![feature(box_syntax)]

extern crate core;

mod bit;
mod cpu;

use cpu::Cpu;
use std::fs::File;
use std::io::*;

fn main() {
    let file = File::open("./misc/super-mario.gba").unwrap();
    let mut rom = BufReader::new(file);
    let mut buffer = [0; 4];

    let mut cpu = Cpu::new();

    loop {
        rom.seek(SeekFrom::Start(cpu.pc() as u64)).unwrap();
        rom.read_exact(&mut buffer);
        let instruction = buffer[0] as u32
                        + ((buffer[1] as u32) << 8)
                        + ((buffer[2] as u32) << 16)
                        + ((buffer[3] as u32) << 24);
        println!("{:b}", instruction);
        cpu.execute(instruction);
    }
}
