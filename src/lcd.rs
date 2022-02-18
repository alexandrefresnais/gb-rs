use crate::mmu::Mmu;
use crate::test_bit;
use crate::cpu::V_BLANK_INTERUPT;

// Memory address storing the current scanline
pub const SCANLINE_REGISTER: u16 = 0xFF44;
const LCD_STATUS_REGISTER: usize = 0xFF41;
const LCD_CONTROL_REGISTER: u16 = 0xFF40;

// Number of cpu clock cycles it takes to draw on scanline
const SCANLINE_CYCLES: i64 = 456;

pub struct Lcd {
    scanlines_cycles: i64
}

impl Lcd {
    pub fn new() -> Self {
        Lcd {
            scanlines_cycles: SCANLINE_CYCLES
        }
    }

    fn is_lcd_enabled(&self, mmu: &mut Mmu) -> bool {
        test_bit(mmu.readb(LCD_CONTROL_REGISTER) as u16, 7)
    }

    pub fn update_graphics(&mut self, cycles: u8, mmu: &mut Mmu) {
        // cycles: elasped cycles given by Cpu
        // We access Mmu memory directly because mmu::writeb() protects SCANLINE_REGISTER

        if !self.is_lcd_enabled(mmu) {
            return;
        }

        self.scanlines_cycles -= cycles as i64;

        if self.scanlines_cycles <= 0 {
            // We have to move on to next scanline
            let mut current_line = mmu.readb(SCANLINE_REGISTER as u16);
            current_line = current_line.wrapping_add(1);
            mmu.memory[SCANLINE_REGISTER as usize] =  current_line;

            self.scanlines_cycles = 456;

            if current_line == 144 {
                mmu.request_interupt(V_BLANK_INTERUPT);
            }

            if current_line > 153 {
                mmu.memory[SCANLINE_REGISTER as usize] = 0;
            }
        }
    }
}
