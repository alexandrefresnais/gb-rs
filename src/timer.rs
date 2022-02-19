use crate::mmu::Mmu;
use crate::utils::Bits;
use crate::cpu::TIMER_INTERUPT;

pub const DIVIDER_REGISTER: u16 = 0xFF04;
pub const TIMA: u16 = 0xFF05; // Timer
pub const TMA: u16 = 0xFF06; // Timer modulator

// Timer controller register
// 3 bits register
// 00: 4096 Hz
// 01: 262144 Hz
// 10: 65536 Hz
// 11: 16384 Hz
// Bit 2: Enabled/Disabled state
pub const TMC: u16 = 0xFF07;

// Interupts at each timer modulation

const CLOCK_SPEED: u32 = 4194304;

pub struct Timer{
    timer_controller: u8, // TMC
    timer: u32, // TIMA
    timer_modulo: u8, // TMA
    timer_cycles: u32, // elasped cycles since last timer inc
    timer_frequency: u32, // Cycles quantity to increment timer
    divider_cycles: u32,
    divider: u8 //DIV
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            timer_controller: 0,
            timer: 0,
            timer_modulo: 0,
            timer_cycles: 0,
            timer_frequency: 256,
            divider_cycles: 0,
            divider: 0
        }
    }

    pub fn update(&mut self, cycles: u32, mmu: &mut Mmu) {
        // cycles: how many CPU cycles have run

        self.divider_cycles += cycles;
        if self.divider_cycles > 255 {
            self.divider += (self.divider_cycles / 256) as u8;
            self.divider_cycles %= 256;
        }

        if !self.is_clock_enabled(mmu) {
            return
        }

        self.timer_cycles += cycles;
        if self.timer_cycles >= self.timer_frequency {
            // Enough cycle have run to increase timer
            if self.timer == 255 {
                // About to overflow, set it to TMA
                self.timer = self.timer_modulo as u32;

                mmu.request_interupt(TIMER_INTERUPT as u8);
            }
            else {
                self.timer += 1;
            }
            self.timer_cycles -= self.timer_frequency;
        }
    }

    fn is_clock_enabled(&self, mmu: &Mmu) -> bool {
        // TODO: Why not use self.time_controller
        mmu.readb(TMC).is_set(2)
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match addr {
            DIVIDER_REGISTER => self.divider,
            TIMA => self.timer as u8,
            TMA => self.timer_modulo,
            TMC => self.timer_controller,
            _ => panic!("Should not happen!")
        }
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        if addr == DIVIDER_REGISTER {
            // Forbidden so we reset divider
            self.divider = 0;
        }
        else if addr == TMC {
            // Frequency change
            self.set_clock_freq(value);
        }
    }

    fn set_clock_freq(&mut self, value: u8) {
        // On writing on TMC
        self.timer_controller = value;
        self.timer_frequency = CLOCK_SPEED / self.get_clock_freq();
    }

    fn get_clock_freq(&self) -> u32 {
        match self.timer_controller & 0b11 {
            0b00 => 4096,
            0b01 => 262144,
            0b10 => 65536,
            0b11 => 16384,
            _ => panic!("Should not happend!")
        }
    }
}