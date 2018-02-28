#![feature(box_syntax)]

extern crate byteorder;
extern crate core;
#[macro_use]
extern crate structopt;

mod bit;
mod bus;
mod cpu;
mod decode;
mod execute;
mod instruction;
mod interrupt_controller;
mod memory_map;

use cpu::Cpu;
use interrupt_controller::InterruptController;
use memory_map::MemoryMap;
use std::cell::RefCell;
use std::fs::File;
use std::io::*;
use std::path::PathBuf;
use std::process;
use std::rc::Rc;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "GBA Emulator")]
struct Opt {
    /// Path to ROM
    #[structopt(short = "r", long = "rom", parse(from_os_str))]
    rom: PathBuf,

    /// Path to BIOS
    #[structopt(short = "b", long = "bios", parse(from_os_str))]
    bios: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    match run(opt) {
        Ok(_) => process::exit(0),
        Err(err) => {
            eprintln!("{}", err);
            process::exit(1);
        }
    };
}

fn run(opt: Opt) -> std::result::Result<(), String> {
    let interrupts = Rc::new(RefCell::new(InterruptController::new()));
    let bios = BufReader::new(File::open(opt.bios)
        .map_err(|err| format!("Error reading BIOS:\n  {}", err))?);
    let rom = BufReader::new(File::open(opt.rom)
        .map_err(|err| format!("Error reading ROM:\n  {}", err))?);
    let memory = MemoryMap::new(bios, rom, Rc::clone(&interrupts));
    let mut cpu = Cpu::new(memory, Rc::clone(&interrupts));
    loop {
        cpu.tick();
    }
}
