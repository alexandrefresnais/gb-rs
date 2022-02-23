use crate::utils::Bits;

use crate::cpu::STAT_INTERUPT;
use crate::cpu::V_BLANK_INTERUPT;

pub const CONTROL_REGISTER: u16 = 0xFF40;
pub const STATUS_REGISTER: u16 = 0xFF41;

// Memory address storing the current scanline
pub const LY_REGISTER: u16 = 0xFF44;
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
const SCANLINE_CYCLES: u32 = 456;

pub const VRAM_START: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;

// Sprite attribute table
pub const OAM_START: u16 = 0xFE00;
pub const OAM_SIZE: usize = 0xA0;
pub const OAM_END: u16 = 0xFE9F;

const VRAM_SIZE: usize = 0x2000;
pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

#[derive(Copy, Clone, PartialEq)]
pub enum Color {
    White,
    DarkGrey,
    LightGrey,
    Black,
}

impl Color {
    pub fn rgb(&self) -> (u8, u8, u8) {
        match self {
            Color::White => (255, 255, 255),
            Color::LightGrey => (192, 192, 192),
            Color::DarkGrey => (96, 96, 96),
            Color::Black => (0, 0, 0),
        }
    }
}

struct LcdStatus {
    lyc_int_enable: bool,
    mode2_int_enable: bool,
    mode1_int_enable: bool,
    mode0_int_enable: bool,
    mode: u8,
    lyc: u8,
    curr_line: u8,
}

impl LcdStatus {
    fn new() -> Self {
        LcdStatus {
            lyc: 0,
            curr_line: 0,
            lyc_int_enable: false,
            mode2_int_enable: false,
            mode1_int_enable: false,
            mode0_int_enable: false,
            mode: 0,
        }
    }

    fn update_modes(&mut self, status: u8) {
        self.lyc_int_enable = status.is_set(6);
        self.mode2_int_enable = status.is_set(5);
        self.mode1_int_enable = status.is_set(4);
        self.mode0_int_enable = status.is_set(3);
    }

    fn to_u8(&self) -> u8 {
        (self.lyc_int_enable as u8) << 6
            | (self.mode2_int_enable as u8) << 5
            | (self.mode1_int_enable as u8) << 4
            | (self.mode0_int_enable as u8) << 3
            | ((self.lyc == self.curr_line) as u8) << 2
            | self.mode
    }
}

struct LcdControl {
    lcd_on: bool,
    window_tilemap: u16,
    window_enable: bool,
    bg_win_tile_data: u16,
    bg_tilemap: u16,
    obj_size: u32,
    obj_enable: bool,
    bg_win_enable: bool,
}

impl LcdControl {
    fn from_u8(value: u8) -> Self {
        LcdControl {
            lcd_on: value.is_set(7),
            window_tilemap: if value.is_set(6) { 0x9C00 } else { 0x9800 },
            window_enable: value.is_set(5),
            bg_win_tile_data: if value.is_set(4) { 0x8000 } else { 0x8800 },
            bg_tilemap: if value.is_set(3) { 0x9C00 } else { 0x9800 },
            obj_size: if value.is_set(2) { 16 } else { 8 },
            obj_enable: value.is_set(1),
            bg_win_enable: value.is_set(0),
        }
    }

    fn to_u8(&self) -> u8 {
        (self.lcd_on as u8) << 7
            | ((self.window_tilemap == 0x9C00) as u8) << 6
            | (self.window_enable as u8) << 5
            | ((self.bg_win_tile_data == 0x8000) as u8) << 4
            | ((self.bg_tilemap == 0x9C00) as u8) << 3
            | ((self.obj_size == 16) as u8) << 2
            | (self.obj_enable as u8) << 1
            | (self.bg_win_enable as u8)
    }
}

pub struct Lcd {
    lcd_control: LcdControl,
    lcd_status: LcdStatus,
    scanlines_cycles: u32,
    scroll_y: u8,
    scroll_x: u8,
    window_x: u8,
    window_y: u8,
    bg_palette: u8,
    obj0_palette: u8,
    obj1_palette: u8,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    pub screen_data: [[Color; SCREEN_HEIGHT]; SCREEN_WIDTH],
    pub int_request: u8,
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            lcd_control: LcdControl::from_u8(88),
            lcd_status: LcdStatus::new(),
            scanlines_cycles: 0,
            scroll_y: 0,
            scroll_x: 0,
            window_x: 0,
            window_y: 0,
            bg_palette: 0,
            obj0_palette: 0,
            obj1_palette: 1,
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            screen_data: [[Color::White; SCREEN_HEIGHT]; SCREEN_WIDTH],
            int_request: 0,
        }
    }

    pub fn update_graphics(&mut self, cycles: u32) {
        if !self.lcd_control.lcd_on {
            return;
        }

        let mut cycles = cycles;

        while cycles > 0 {
            self.scanlines_cycles += 1;
            cycles -= 1;

            // Finished line
            if self.scanlines_cycles >= SCANLINE_CYCLES {
                self.scanlines_cycles = 0;
                self.lcd_status.curr_line = (self.lcd_status.curr_line + 1) % 154;
                if self.lcd_status.lyc_int_enable
                    && self.lcd_status.curr_line == self.lcd_status.lyc
                {
                    self.int_request |= 1 << STAT_INTERUPT;
                }

                // This is a VBlank line
                if self.lcd_status.curr_line >= 144 && self.lcd_status.mode != 1 {
                    self.set_mode(1);
                }
            }

            // This is not a VBLANK line
            if self.lcd_status.curr_line < 144 {
                if self.scanlines_cycles <= 80 {
                    if self.lcd_status.mode != 2 {
                        self.set_mode(2);
                    }
                } else if self.scanlines_cycles <= 252 {
                    if self.lcd_status.mode != 3 {
                        self.set_mode(3);
                    }
                } else {
                    if self.lcd_status.mode != 0 {
                        self.set_mode(0);
                    }
                }
            }
        }
    }

    fn set_mode(&mut self, mode: u8) {
        self.lcd_status.mode = mode;
        if mode == 0 {
            self.draw_scanline();
            if self.lcd_status.mode0_int_enable {
                self.int_request |= 1 << STAT_INTERUPT;
            }
        } else if mode == 1 {
            self.int_request |= 1 << V_BLANK_INTERUPT;
            if self.lcd_status.mode1_int_enable {
                self.int_request |= 1 << STAT_INTERUPT;
            }
        } else if mode == 2 {
            if self.lcd_status.mode2_int_enable {
                self.int_request |= 1 << STAT_INTERUPT
            }
        }
    }

    fn turn_off_lcd(&mut self) {
        self.clear_screen();
        self.scanlines_cycles = 0;
        self.lcd_status.curr_line = 0;
        self.lcd_status.mode = 0;
    }

    fn clear_screen(&mut self) {
        for x in 0..SCREEN_WIDTH {
            for y in 0..SCREEN_HEIGHT {
                self.screen_data[x][y] = Color::White;
            }
        }
    }

    pub fn readb(&self, addr: u16) -> u8 {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize],
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize],
            CONTROL_REGISTER => self.lcd_control.to_u8(),
            STATUS_REGISTER => self.lcd_status.to_u8(),
            SCROLL_Y_REGISTER => self.scroll_y,
            SCROLL_X_REGISTER => self.scroll_x,
            LY_REGISTER => self.lcd_status.curr_line,
            LYC_REGISTER => self.lcd_status.lyc,
            BG_PALETTE => self.bg_palette,
            OBJ_PALETTE_0 => self.obj0_palette,
            OBJ_PALETTE_1 => self.obj1_palette,
            WINDOW_Y_REGISTER => self.window_y,
            WINDOW_X_REGISTER => self.window_x,
            _ => panic!("Unexpected read for PPU at {:x}", addr),
        }
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        match addr {
            VRAM_START..=VRAM_END => self.vram[(addr - VRAM_START) as usize] = value,
            OAM_START..=OAM_END => self.oam[(addr - OAM_START) as usize] = value,
            CONTROL_REGISTER => {
                self.lcd_control = LcdControl::from_u8(value);
                if !self.lcd_control.lcd_on {
                    self.turn_off_lcd();
                }
            }
            STATUS_REGISTER => self.lcd_status.update_modes(value),
            SCROLL_Y_REGISTER => self.scroll_y = value,
            SCROLL_X_REGISTER => self.scroll_x = value,
            LY_REGISTER => {} // Read-only
            LYC_REGISTER => self.lcd_status.lyc = value,
            BG_PALETTE => self.bg_palette = value,
            OBJ_PALETTE_0 => self.obj0_palette = value,
            OBJ_PALETTE_1 => self.obj1_palette = value,
            WINDOW_Y_REGISTER => self.window_y = value,
            WINDOW_X_REGISTER => self.window_x = value,
            _ => panic!("Unexpected write for PPU at {:x}", addr),
        }
    }

    fn get_color(&self, palette_id: u8, palette: u8) -> Color {
        let color_id = match palette_id & 0b11 {
            0b00 => (palette & 0b11),
            0b01 => (palette & 0b1100) >> 2,
            0b10 => (palette & 0b110000) >> 4,
            0b11 => (palette & 0b11000000) >> 6,
            _ => panic!("Should not happend!"),
        };

        match color_id {
            0b00 => Color::White,
            0b01 => Color::LightGrey,
            0b10 => Color::DarkGrey,
            0b11 => Color::Black,
            _ => panic!("Should not happend!"),
        }
    }

    fn draw_scanline(&mut self) {
        if self.lcd_control.bg_win_enable {
            self.draw_tiles();
        }
        if self.lcd_control.obj_enable {
            self.draw_sprites();
        }
    }

    fn draw_tiles(&mut self) {
        let window_y = match self.lcd_control.window_enable {
            true => self.lcd_status.curr_line as i32 - self.window_y as i32,
            false => -1,
        };

        let y_pos = self.scroll_y.wrapping_add(self.lcd_status.curr_line);

        for x in 0..SCREEN_WIDTH {
            let window_x = (x as i32) - ((self.window_x as i32) - 7);
            let x_pox = self.scroll_x as u32 + x as u32;

            let draw_window = self.lcd_control.window_enable && window_y >= 0 && window_x >= 0;

            let base_memory = match draw_window {
                true => self.lcd_control.window_tilemap,
                false => self.lcd_control.bg_tilemap
            };

            let (tile_row, tile_col) = match draw_window {
                true => ((window_y as u16 / 8), (window_x as u16 / 8)),
                false => ((y_pos as u16 / 8) & 31, (x_pox as u16 / 8) & 31),
            };

            let (y_tile, x_tile) = match draw_window {
                true => (window_y as u16 & 0x07, window_x as u8 & 0x07),
                false => (y_pos as u16 & 0x07, x_pox as u8 & 0x07),
            };

            let tile_num: u8 = self.readb(base_memory + tile_row * 32 + tile_col);

            let tile_location: u16 = match self.lcd_control.bg_win_tile_data {
                0x8000 => self.lcd_control.bg_win_tile_data + (tile_num as u16) * 16,
                _ => self.lcd_control.bg_win_tile_data + ((tile_num as i8 as i16 + 128) as u16) * 16
            };

            // Each 8 pixels line is encode on 2 bytes
            let line = y_tile * 2;
            let color_data_1 = self.readb(tile_location + line);
            let color_data_2 = self.readb(tile_location + line + 1);

            // The ith pixel color is the combination of 7-ith bit of color_data_1
            // and 7-ith bit of color_data_2
            let color_bit = 7 - (x_tile % 8);

            let color_id = color_data_2.get_bit(color_bit) << 1 | color_data_1.get_bit(color_bit);
            let color = self.get_color(color_id, self.bg_palette);
            self.screen_data[x as usize][self.lcd_status.curr_line as usize] = color;
        }
    }

    fn draw_sprites(&mut self) {
        // Interactive graphics

        // 40 tiles located in memory region 0x8000-0x8FFF
        for sprite in 0..40 {
            // sprite are 4 bytes wide
            let index = ((39 - sprite) as u16) * 4;

            let y_pos = self.readb(0xFE00 + index + 0) as u16 as i32 - 16;
            let y_size = self.lcd_control.obj_size as i32;

            let line = self.lcd_status.curr_line as i32;
            if !(y_pos..(y_pos + y_size)).contains(&line) {
                continue;
            }

            let x_pos = self.readb(0xFE00 + index + 1) as u16 as i32 - 8;
            let tile_location = match self.lcd_control.obj_size {
                16 => self.readb(0xFE00 + index + 2) & 0xFE,
                8 => self.readb(0xFE00 + index + 2) & 0xFF,
                _ => panic!("Unexpected OBJ size."),
            } as u16;

            let attributes = self.readb(0xFE00 + index + 3);

            let y_flip = attributes.is_set(6);
            let x_flip = attributes.is_set(5);

            let behind_bg: bool = attributes.is_set(7);

            if x_pos < -7 || x_pos >= (SCREEN_WIDTH as i32) {
                continue;
            }

            let line: u16 = match y_flip {
                true => (y_pos + y_size - line - 1) as u16,
                false => (line - y_pos) as u16,
            };

            let data_addr = 0x8000u16 + tile_location * 16 + line * 2;
            let color_data_1 = self.readb(data_addr);
            let color_data_2 = self.readb(data_addr + 1);

            for tile_pixel in 0..8 {
                let x = x_pos + tile_pixel;
                if x < 0 || x >= (SCREEN_WIDTH as i32) {
                    continue;
                }

                // Pixel are already stored in reverse
                // So we actually do not flip is x_flip is set
                let color_bit = match x_flip {
                    true => tile_pixel,
                    false => 7 - tile_pixel,
                };

                let color_id = color_data_2.get_bit(color_bit as u8) << 1
                    | color_data_1.get_bit(color_bit as u8);
                // Transparent
                if color_id == 0 {
                    continue;
                }

                if behind_bg
                    && self.screen_data[x as usize][self.lcd_status.curr_line as usize]
                        != Color::White
                {
                    continue;
                }
                let palette = match attributes.is_set(4) {
                    true =>self.obj1_palette,
                    false => self.obj0_palette
                };

                let color = self.get_color(color_id, palette);
                self.screen_data[x as usize][self.lcd_status.curr_line as usize] = color;
            }
        }
    }
}
