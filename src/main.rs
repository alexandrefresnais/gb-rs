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

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = std::fs::read(&args[1]);
    let mut mmu = mmu::MMU::new(rom.unwrap());
    let mut cpu = cpu::CPU::new();

    loop {
        cpu.run_cycle(&mut mmu);
    }
}
