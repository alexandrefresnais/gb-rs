use std::env;

extern crate minifb;
use minifb::{Window, WindowOptions, ScaleMode, Scale};

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

use lcd::SCREEN_HEIGHT;
use lcd::SCREEN_WIDTH;


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
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cartridge = Cartridge::new(&args[1]);
    let mut mmu = Mmu::new(&mut cartridge);
    let mut cpu = Cpu::new();

    let mut win_opt = WindowOptions::default();
    win_opt.scale_mode = ScaleMode::AspectRatioStretch;
    win_opt.resize = true;
    win_opt.scale = Scale::X2;

    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut window = Window::new(
        "gb-rs",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        win_opt,
    ).unwrap();

      while window.is_open() {
        run_one_frame(&mut cpu, &mut mmu);
        for x in 0..SCREEN_WIDTH {
            for y in 0..SCREEN_HEIGHT {
                let (r, g, b) = mmu.lcd.screen_data[x][y].rgb();
                buffer[y * SCREEN_WIDTH + x] = 0xFF000000 | (r as u32) << 16 | (g as u32)  << 8 | (b as u32);
            }
        }
        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}