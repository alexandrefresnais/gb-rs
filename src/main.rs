use std::env;

extern crate minifb;
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};

mod cartridge;
mod cpu;
mod joypad;
mod lcd;
mod mmu;
mod registers;
mod timer;
mod utils;

use cartridge::Cartridge;
use cpu::Cpu;
use mmu::Mmu;

use joypad::*;

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

    let mut buffer: Vec<u32> = vec![0; SCREEN_WIDTH * SCREEN_HEIGHT];
    let mut window = Window::new(
        "gb-rs",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            resize: true,
            scale_mode: ScaleMode::AspectRatioStretch,
            scale: Scale::X2,
            ..WindowOptions::default()
        },
    )
    .unwrap();

    // Make sure that at least 4 ms has passed since the last event poll
    window.limit_update_rate(Some(std::time::Duration::from_millis(16)));

    while window.is_open() {
        run_one_frame(&mut cpu, &mut mmu);
        for x in 0..SCREEN_WIDTH {
            for y in 0..SCREEN_HEIGHT {
                let (r, g, b) = mmu.lcd.screen_data[x][y].rgb();
                buffer[y * SCREEN_WIDTH + x] =
                    0xFF000000 | (r as u32) << 16 | (g as u32) << 8 | (b as u32);
            }
        }
        window
            .update_with_buffer(&buffer, SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();

        window
            .get_keys_pressed(KeyRepeat::No)
            .iter()
            .for_each(|key| match key {
                Key::Left => mmu.joypad.on_key_pressed(JoypadInput::LEFT),
                Key::Right => mmu.joypad.on_key_pressed(JoypadInput::RIGHT),
                Key::Up => mmu.joypad.on_key_pressed(JoypadInput::UP),
                Key::Down => mmu.joypad.on_key_pressed(JoypadInput::DOWN),
                Key::A => mmu.joypad.on_key_pressed(JoypadInput::A),
                Key::S => mmu.joypad.on_key_pressed(JoypadInput::B),
                Key::Enter => mmu.joypad.on_key_pressed(JoypadInput::START),
                Key::Space => mmu.joypad.on_key_pressed(JoypadInput::SELECT),
                _ => (),
            });

        window.get_keys_released().iter().for_each(|key| match key {
            Key::Left => mmu.joypad.on_key_released(JoypadInput::LEFT),
            Key::Right => mmu.joypad.on_key_released(JoypadInput::RIGHT),
            Key::Up => mmu.joypad.on_key_released(JoypadInput::UP),
            Key::Down => mmu.joypad.on_key_released(JoypadInput::DOWN),
            Key::A => mmu.joypad.on_key_released(JoypadInput::A),
            Key::S => mmu.joypad.on_key_released(JoypadInput::B),
            Key::Enter => mmu.joypad.on_key_released(JoypadInput::START),
            Key::Space => mmu.joypad.on_key_released(JoypadInput::SELECT),
            _ => (),
        });
    }
}
