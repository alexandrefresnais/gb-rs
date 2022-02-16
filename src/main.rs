use std::env;

fn to_u8(value: u16) -> (u8, u8) {
    // Converts u16 into (msb: u8, lsb: u8)
    let lsb = value & 0xff;
    let msb = (value & 0xff00) >> 8;
    (msb as u8, lsb as u8)
}

fn to_u16(msb: u8, lsb: u8) -> u16 {
    ((msb as u16) << 8) | lsb as u16
}

mod cpu;
mod mmu;
mod registers;
mod lcd;

use cpu::CPU;
use mmu::MMU;
use lcd::LCD;

fn run_one_frame(cpu: &mut CPU, mmu: &mut MMU) {
    // GameBoy can execute 4194304 cycles per second
    // We want 60 frames per second
    // So we run 69905 each frame
    const FRAME_CYLES: u64 = 69905;

    let mut cycles: u64 = 0;
    while cycles < FRAME_CYLES {
        let cpu_cycles = cpu.run_cycle(mmu);

        cycles += cpu_cycles as u64;
    }

    // Render the screen
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = std::fs::read(&args[1]);
    let mut mmu = mmu::MMU::new(rom.unwrap());
    let mut cpu = cpu::CPU::new();
    let mut lcd = lcd::LCD::new();

    loop {
        run_one_frame(&mut cpu, &mut mmu);
    }
}
