use std::env;

extern crate minifb;
use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};

mod cartridge;
mod cpu;
mod joypad;
mod lcd;
mod mbc0;
mod mbc1;
mod mbc3;
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

fn run_one_frame(cpu: &mut Cpu, mmu: &mut Mmu) {
    // Game Boy can execute 4194304 cycles per second
    // We want 60 frames per second
    // So we run 69905 each frame
    const FRAME_CYLES: u32 = 69905;

    let mut cycles: u32 = 0;
    while cycles < FRAME_CYLES {
        let cpu_cycles = cpu.run_cycle(mmu);

        mmu.update(cpu_cycles);
        cpu.check_interupts(mmu);
        cycles += cpu_cycles;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut cartridge = Cartridge::new(&args[1]);
    let mut mmu = Mmu::new(&mut cartridge);
    let mut cpu = Cpu::default();

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
                Key::Left => mmu.joypad.on_key_pressed(JoypadInput::Left),
                Key::Right => mmu.joypad.on_key_pressed(JoypadInput::Right),
                Key::Up => mmu.joypad.on_key_pressed(JoypadInput::Up),
                Key::Down => mmu.joypad.on_key_pressed(JoypadInput::Down),
                Key::A => mmu.joypad.on_key_pressed(JoypadInput::A),
                Key::S => mmu.joypad.on_key_pressed(JoypadInput::B),
                Key::Enter => mmu.joypad.on_key_pressed(JoypadInput::Start),
                Key::Space => mmu.joypad.on_key_pressed(JoypadInput::Select),
                _ => (),
            });

        window.get_keys_released().iter().for_each(|key| match key {
            Key::Left => mmu.joypad.on_key_released(JoypadInput::Left),
            Key::Right => mmu.joypad.on_key_released(JoypadInput::Right),
            Key::Up => mmu.joypad.on_key_released(JoypadInput::Up),
            Key::Down => mmu.joypad.on_key_released(JoypadInput::Down),
            Key::A => mmu.joypad.on_key_released(JoypadInput::A),
            Key::S => mmu.joypad.on_key_released(JoypadInput::B),
            Key::Enter => mmu.joypad.on_key_released(JoypadInput::Start),
            Key::Space => mmu.joypad.on_key_released(JoypadInput::Select),
            _ => (),
        });
    }
}
