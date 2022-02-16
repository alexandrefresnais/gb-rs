// Memory address storing the current scanline
pub const SCANLINE_REGISTER: usize = 0xFF44;
const LCD_STATUS_REGISTER: usize = 0xFF41;

// Number of cpu clock cycles it takes to draw on scanline
const SCANLINE_CYCLES: i64 = 456;

use super::mmu::MMU;

pub struct LCD {
    scanlines_cycles: i64
}

impl LCD {
    pub fn new() -> LCD {
        LCD {
            scanlines_cycles: SCANLINE_CYCLES
        }
    }

    pub fn update_graphics(&mut self, cycles: u8, mmu: &mut MMU) {
        // cycles: elasped cycles given by CPU
        // We access MMU memory directly because mmu::writeb() protects SCANLINE_REGISTER

        self.scanlines_cycles -= cycles as i64;

        if self.scanlines_cycles <= 0 {
            // We have to move on to next scanline
            let mut current_line = mmu.readb(SCANLINE_REGISTER as u16);
            current_line = current_line.wrapping_add(1);
            mmu.memory[SCANLINE_REGISTER] =  current_line;

            self.scanlines_cycles = 456;

            if current_line > 153 {
                mmu.memory[SCANLINE_REGISTER] = 0;
            }
        }
    }
}
