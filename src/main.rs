use std::env;

mod cpu;
mod mmu;
mod registers;
mod lcd;
mod cartridge;
mod timer;
mod utils;

use cpu::Cpu;
use mmu::Mmu;
use cartridge::Cartridge;

fn read_blargg(mmu: &mut Mmu) {
    let has_out = mmu.readb(0xff02);
    if has_out == 0x81 {
        let chr = mmu.readb(0xff01) as char;
        print!("{}", chr);
        mmu.writeb(0xff02, 0);
    }
}

fn run_one_frame(cpu: &mut Cpu, mmu: &mut Mmu) {
    // GameBoy can execute 4194304 cycles per second
    // We want 60 frames per second
    // So we run 69905 each frame
    const FRAME_CYLES: u32 = 69905;

    let mut cycles: u32 = 0;
    while cycles < FRAME_CYLES {
        let cpu_cycles = cpu.run_cycle(mmu);

        mmu.update(cpu_cycles);
        cpu.check_interupts(mmu);
        cycles += cpu_cycles;

        // read_blargg(mmu);
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