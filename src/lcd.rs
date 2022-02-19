use crate::cpu::LCD_INTERUPT;
use crate::cpu::V_BLANK_INTERUPT;
use crate::mmu::Mmu;
use crate::utils::Bits;

// Memory address storing the current scanline
pub const SCANLINE_REGISTER: u16 = 0xFF44;
pub const STATUS_REGISTER: u16 = 0xFF41;
pub const LCD_CONTROL_REGISTER: u16 = 0xFF40;

// Number of cpu clock cycles it takes to draw on scanline
const SCANLINE_CYCLES: i64 = 456;

pub struct Lcd {
    scanlines_cycles: i64,
    curr_line: u8,
    status: u8,
}

impl Lcd {
    pub fn new() -> Self {
        Lcd {
            scanlines_cycles: SCANLINE_CYCLES,
            curr_line: 0,
            status: 0,
        }
    }

    fn is_lcd_enabled(&self, mmu: &mut Mmu) -> bool {
        mmu.readb(LCD_CONTROL_REGISTER).is_set(7)
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match addr {
            SCANLINE_REGISTER => self.curr_line,
            STATUS_REGISTER => self.status,
            _ => panic!("Should not happen!"),
        }
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        match addr {
            SCANLINE_REGISTER => self.curr_line = 0, // Writing resets it
            STATUS_REGISTER => self.status = value,
            _ => panic!("Should not happen!"),
        }
    }

    pub fn update_graphics(&mut self, cycles: u8, mmu: &mut Mmu) {
        // cycles: elasped cycles given by Cpu
        // We access Mmu memory directly because mmu::writeb() protects SCANLINE_REGISTER

        self.set_lcd_status(mmu);

        if !self.is_lcd_enabled(mmu) {
            return;
        }

        self.scanlines_cycles -= cycles as i64;

        if self.scanlines_cycles <= 0 {
            // We have to move on to next scanline
            self.curr_line = self.curr_line.wrapping_add(1);

            self.scanlines_cycles = SCANLINE_CYCLES;

            if self.curr_line == 144 {
                mmu.request_interupt(V_BLANK_INTERUPT);
            }

            if self.curr_line > 153 {
                self.curr_line = 0;
            } else if self.curr_line < 144 {
                self.draw_scanline(mmu);
            }
        }
    }

    fn draw_scanline(&mut self, mmu: &mut Mmu) {
        let control = mmu.readb(LCD_CONTROL_REGISTER);
        if control.is_set(0) {
            self.render_tiles(mmu);
        } else if control.is_set(1) {
            self.render_sprites(mmu);
        }
    }

    fn render_tiles(&mut self, mmu: &mut Mmu) {}

    fn render_sprites(&mut self, mmu: &mut Mmu) {}

    fn set_lcd_status(&mut self, mmu: &mut Mmu) {
        if !self.is_lcd_enabled(mmu) {
            // Set mode to 1 and reset scanline
            self.scanlines_cycles = SCANLINE_CYCLES;
            self.curr_line = 0;
            self.status &= 0b11111100;
            self.status |= 0b1;
            return;
        }

        let curr_mode = self.status & 0x3;
        let mut mode = 0;
        let mut status = self.status;
        let mut reqInt = false;

        if (self.curr_line >= 144) {
            mode = 1;
            status |= 1;
            status &= 0b11111101;
            reqInt = status.is_set(4);
        } else {
            let mode2bounds = 456 - 80;
            let mode3bounds = mode2bounds - 172;

            // mode 2
            if self.scanlines_cycles >= mode2bounds {
                mode = 2;
                status |= 0b10;
                status &= 0b11111110;
                reqInt = status.is_set(5);
            }
            // mode 3
            else if self.scanlines_cycles >= mode3bounds {
                mode = 3;
                status |= 0b11;
            }
            // mode 0
            else {
                mode = 0;
                status &= 0b11111100;
                reqInt = status.is_set(3);
            }
        }

        // Interupt requested and switch mode
        if reqInt && (mode != curr_mode) {
            mmu.request_interupt(LCD_INTERUPT);
        }

        if self.curr_line == mmu.readb(0xFF45) {
            status |= 1 << 2;
            if status.is_set(6) {
                mmu.request_interupt(LCD_INTERUPT);
            }
        } else {
            status &= !(1 << 2);
        }

        self.status = status;
    }
}
