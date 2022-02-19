use crate::cpu::LCD_INTERUPT;
use crate::cpu::V_BLANK_INTERUPT;
use crate::utils::Bits;

pub const CONTROL_REGISTER: u16 = 0xFF40;
pub const STATUS_REGISTER: u16 = 0xFF41;

// Memory address storing the current scanline
pub const SCANLINE_REGISTER: u16 = 0xFF44;

pub const LYC_REGISTER: u16 = 0xFF45;

// Palette to fetch colors from color id
pub const BG_PALETTE: u16 = 0xFF47;
pub const OBJ_PALETTE_0: u16 = 0xFF48;
pub const OBJ_PALETTE_1: u16 = 0xFF49;

// Background is 256x256 but view is 160x144
pub const SCROLL_Y_REGISTER: u16 = 0xFF42; // Y Position of the bg to start drawing the viewing area from
pub const SCROLL_X_REGISTER: u16 = 0xFF43; // X Position of the bg to start drawing the viewing area from
pub const WINDOW_Y_REGISTER: u16 = 0xFF4A; // Y Position of the viewing aera to start drawing the window from
pub const WINDOW_X_REGISTER: u16 = 0xFF4B; // X Position -7 of the viewing aera to start drawing the window from

// Number of cpu clock cycles it takes to draw on scanline
const SCANLINE_CYCLES: i64 = 456;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_SIZE: u16 = 0x2000;
pub const VRAM_END: u16 = VRAM_START + VRAM_SIZE - 1;

// Sprite attribute table
pub const OAM_START: u16 = 0xFE00;
pub const OAM_SIZE: u16 = 0x100;
pub const OAM_END: u16 = OAM_START + OAM_SIZE - 1;

#[derive(Copy, Clone)]
enum Color {
    White,
    DarkGrey,
    LightGrey,
    Black
}

impl Color {
    fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255),
            Color::DarkGrey => (0xcc, 0xcc, 0xcc),
            Color::LightGrey => (0x77, 0x77, 0x77),
            Color::Black => (0, 0, 0),
        }
    }
}

pub struct Lcd {
    vram: [u8; VRAM_SIZE as usize],
    oam: [u8; OAM_SIZE as usize],
    scanlines_cycles: i64,
    curr_line: u8, // LY
    lyc: u8,
    status: u8,
    control: u8,
    screen_data: [[Color; 144]; 160],
    scroll_x: u8,
    scroll_y: u8,
    window_x: u8,
    window_y: u8,
    bg_palette: u8,
    obj0_palette: u8,
    obj1_palette: u8,
    pub int_request: u8,
}

impl Lcd {
    pub fn new() -> Self {
        Lcd {
            vram: [0; VRAM_SIZE as usize],
            oam: [0; OAM_SIZE as usize],
            scanlines_cycles: SCANLINE_CYCLES,
            curr_line: 0,
            lyc: 0,
            status: 0,
            control: 0x91,
            screen_data: [[Color::White; 144]; 160],
            scroll_x: 0,
            scroll_y: 0,
            window_x: 0,
            window_y: 0,
            bg_palette: 0,
            obj0_palette: 0,
            obj1_palette: 1,
            int_request: 0,
        }
    }

    fn is_lcd_enabled(&self) -> bool {
        self.control.is_set(7)
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize],
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize],
            SCANLINE_REGISTER => self.curr_line,
            STATUS_REGISTER => self.status,
            CONTROL_REGISTER => self.control,
            BG_PALETTE => self.bg_palette,
            OBJ_PALETTE_0 => self.obj0_palette,
            OBJ_PALETTE_1 => self.obj1_palette,
            SCROLL_X_REGISTER => self.scroll_x,
            SCROLL_Y_REGISTER => self.scroll_y,
            WINDOW_X_REGISTER => self.window_x,
            WINDOW_Y_REGISTER => self.window_y,
            LYC_REGISTER => self.lyc,
            _ => panic!("Should not happen!"),
        }
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize] = value,
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize] = value,
            SCANLINE_REGISTER => self.curr_line = 0, // Writing resets it
            STATUS_REGISTER => self.status = value,
            CONTROL_REGISTER => self.control = value,
            BG_PALETTE => self.bg_palette = value,
            OBJ_PALETTE_0 => self.obj0_palette = value,
            OBJ_PALETTE_1 => self.obj1_palette = value,
            SCROLL_X_REGISTER => self.scroll_x = value,
            SCROLL_Y_REGISTER => self.scroll_y = value,
            WINDOW_X_REGISTER => self.window_x = value,
            WINDOW_Y_REGISTER => self.window_y = value,
            LYC_REGISTER => self.lyc = value,
            _ => panic!("Should not happen!"),
        }
    }

    pub fn update_graphics(&mut self, cycles: u32) {
        // cycles: elasped cycles given by Cpu
        self.set_lcd_status();

        if !self.is_lcd_enabled() {
            return;
        }

        self.scanlines_cycles -= cycles as i64;

        if self.scanlines_cycles <= 0 {
            // We have to move on to next scanline
            self.curr_line = self.curr_line.wrapping_add(1);

            self.scanlines_cycles = SCANLINE_CYCLES;

            if self.curr_line == 144 {
                self.int_request |= 1 << V_BLANK_INTERUPT;
            }

            if self.curr_line > 153 {
                self.curr_line = 0;
            } else if self.curr_line < 144 {
                self.draw_scanline();
            }
        }
    }

    fn draw_scanline(&mut self) {
        if self.control.is_set(0) {
            self.render_tiles();
        } else if self.control.is_set(1) {
            self.render_sprites();
        }
    }

    fn render_tiles(&mut self) {
        // Tiles form the background and are not interactive.
        // Size: 8x8

        let window_y = self.window_y - 7;

        // Check if window is set and current scanline has window
        let draw_window: bool = self.control.is_set(5) && window_y <= self.curr_line;

        let mut tile_data: u16 = 0x8000;
        let mut unsig_op = true;

        if !self.control.is_set(4) {
            tile_data = 0x8800;
            unsig_op = false;
        }

        // TODO: opti
        let background_memory: u16 = match draw_window {
            true => if self.control.is_set(6) { 0x9c00 } else { 0x9800 },
            false => if self.control.is_set(3) { 0x9c00 } else { 0x9800 },
        };

        // Which of the 32 Y tiles we are drawing
        let y_tile: u8 = match draw_window {
            true => self.curr_line - window_y,
            false => self.scroll_y + self.curr_line,
        };

        // Which of 8 pixel of tile are we drawinf
        let tile_row = (y_tile / 8) * 32;

        // Draw the 160 horizontal pixels
        for pixel in 0..160 {
            let mut x_pos = pixel + self.scroll_x;
            if draw_window && pixel >= self.window_x {
                x_pos = pixel - self.window_x;
            }

            // which of the 32 horizontal tiles
            let tile_col: u16 = (x_pos / 8) as u16;

            let tile_addr: u16 = background_memory + tile_row as u16 + tile_col;
            let tile_num: u8 = self.readb(tile_addr);

            let tile_location: u16 = match unsig_op {
                true => (tile_data + (tile_num as u16 * 16)) as u16,
                false => tile_data + (((tile_num as i8 as i16) + 128) * 16) as u16,
            };

            // Each 8 pixels line is encode on 2 bytes
            let line: u8 = (y_tile % 8) * 2;
            let color_data_1 = self.readb(tile_location + line as u16);
            let color_data_2 = self.readb(tile_location + line as u16 + 1);

            // The ith pixel color is the combination of 7-ith bit of color_data_1
            // and 7-ith bit of color_data_2
            let color_bit = 7 - (x_pos % 8);

            let color_id = color_data_2.get_bit(color_bit) << 1 | color_data_1.get_bit(color_bit);
            let color: Color = self.get_color(color_id, self.bg_palette);

            // Safety check
            if self.curr_line > 143 || pixel > 159 {
                continue;
            }

            self.screen_data[pixel as usize][self.curr_line as usize] = color;
        }
    }

    fn get_color(&self, palette_id: u8, palette: u8) -> Color {

        let color_id = match palette_id & 0b11 {
            0b00 => palette & 0b11,
            0b01 => palette & 0b1100 >> 2,
            0b10 => palette & 0b110000 >> 4,
            0b11 => palette & 0b11000000 >> 6,
            _ => panic! ("Should not happend!")
        };

        match color_id {
            0b00 => Color::White,
            0b01 => Color::LightGrey,
            0b10 => Color::DarkGrey,
            0b11 => Color::Black,
            _ => panic! ("Should not happend!")
        }
    }

    fn render_sprites(&mut self) {
        // Interactive graphics

        let use8x16 = self.control.is_set(2);

        // 40 tiles located in memory region 0x8000-0x8FFF
        for sprite in 0..40 {
            // sprite are 4 bytes wide
            let index: u8 = sprite * 4;

            let y_pos = self.readb(0xFE00 + index as u16) - 16;
            let x_pos = self.readb(0xFE00 + index as u16 + 1) - 8;
            let tile_location = self.readb(0xFE00 + index as u16 + 2);
            let attributes = self.readb(0xFE00 + index as u16 + 3);

            let y_flip = attributes.is_set(6);
            let x_flip = attributes.is_set(5);

            let y_size = match use8x16 {
                true => 16,
                false => 8
            };

            if (y_pos..(y_pos + y_size)).contains(&self.curr_line) {
                let mut line = self.curr_line - y_pos;

                if y_flip {
                    line = y_size - line;
                }

                // Each 8 pixels line is encode on 2 bytes
                line *= 2;
                let data_addr: u16 = (0x8000 + (tile_location as u16 * 16)) + line as u16;
                let color_data_1 = self.readb(data_addr);
                let color_data_2 = self.readb(data_addr + 1);

                // Doing in reverse because color or stored reversed
                for tile_pixel in (0..8).rev() {
                    let mut color_bit = tile_pixel;
                    if x_flip {
                        color_bit = 7 - color_bit;
                    }

                    let color_id = color_data_2.get_bit(color_bit) << 1 | color_data_1.get_bit(color_bit);

                    let palette = if attributes.is_set(4) { self.obj1_palette } else { self.obj0_palette };
                    let color = self.get_color(color_id, palette);

                    // transparent
                    if let Color::White = color {
                        continue;
                    }

                    // reverse back
                    let pixel = x_pos + (7 - tile_pixel);
                    // Safety check
                    if self.curr_line > 143 || pixel > 159 {
                        continue;
                    }

                    self.screen_data[pixel as usize][self.curr_line as usize] = color;
                }
            }
        }
    }

    fn set_lcd_status(&mut self) {
        if !self.is_lcd_enabled() {
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
        let mut req_int = false;

        if self.curr_line >= 144 {
            mode = 1;
            status |= 1;
            status &= 0b11111101;
            req_int = status.is_set(4);
        } else {
            let mode2bounds = 456 - 80;
            let mode3bounds = mode2bounds - 172;

            // mode 2
            if self.scanlines_cycles >= mode2bounds {
                mode = 2;
                status |= 0b10;
                status &= 0b11111110;
                req_int = status.is_set(5);
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
                req_int = status.is_set(3);
            }
        }

        // Interupt requested and switch mode
        if req_int && (mode != curr_mode) {
            self.int_request |= 1 << LCD_INTERUPT;
        }

        if self.curr_line == self.lyc {
            status |= 1 << 2;
            if status.is_set(6) {
                self.int_request |= 1 << LCD_INTERUPT;
            }
        } else {
            status &= !(1 << 2);
        }

        self.status = status;
    }
}
