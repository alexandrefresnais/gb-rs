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

fn test_bit(data: u16, index: u16) -> bool {
    // Returns true if nth bit of data is set
    data & (1 << index) != 0
}

mod cpu;
mod mmu;
mod registers;
mod lcd;
mod cartridge;
mod timer;

use cpu::Cpu;
use mmu::Mmu;
use cartridge::Cartridge;

fn run_one_frame(cpu: &mut Cpu, mmu: &mut Mmu) {
    // GameBoy can execute 4194304 cycles per second
    // We want 60 frames per second
    // So we run 69905 each frame
    const FRAME_CYLES: u64 = 69905;

    let mut cycles: u64 = 0;
    while cycles < FRAME_CYLES {
        let cpu_cycles = cpu.run_cycle(mmu);

        // TODO update timer

        cpu.check_interupts(mmu);
        cycles += cpu_cycles as u64;
    }

    // Render the screen
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cartridge = Cartridge::new(&args[1]);
    let mut mmu = Mmu::new(&mut cartridge);
    let mut cpu = Cpu::new();

    loop {
        run_one_frame(&mut cpu, &mut mmu);
    }
}
