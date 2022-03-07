use crate::cartridge::Mbc;
use crate::cartridge::RAM_BANK_SIZE;
use crate::cartridge::ROM_BANK_SIZE;

pub struct Mbc1 {
    rom: Vec<u8>,
    is_ram_enabled: bool,
    // If true 16Mb ROM/8KB RAM else 4Mb ROM/32KB RAM
    is_rom_banking: bool,
    current_rom_bank: u8,
    current_ram_bank: u8,
    ram_banks: [u8; 0x8000],
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Box<Self> {
        Box::new(Mbc1 {
            rom,
            is_ram_enabled: false,
            is_rom_banking: true,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_banks: [0; 0x8000], // Max 32KB of RAM
        })
    }

    fn toggle_ram_banking(&mut self, value: u8) {
        // RAM banking is only enabled if four first bits of value written is 0xA
        self.is_ram_enabled = (value & 0xf) == 0xA;
    }

    fn change_low_rom_bank(&mut self, value: u8) {
        // Replace 5 lowest bits with value's ones
        let lower5 = value & 0b11111;
        self.current_rom_bank &= 0b11100000;
        self.current_rom_bank |= lower5;

        self.current_rom_bank = match self.current_rom_bank {
            0 => 1,
            0x20 => 0x21,
            0x40 => 0x41,
            0x60 => 0x61,
            _ => self.current_rom_bank,
        };
    }

    fn change_hi_rom_bank(&mut self, data: u8) {
        // Change the highest 3 bits of self.current_rom_bank
        self.current_rom_bank &= 0b11111;
        let bits = data & 0b11100000;
        self.current_rom_bank |= bits;

        if self.current_rom_bank == 0 {
            self.current_rom_bank = 1;
        }
    }

    fn change_ram_bank(&mut self, data: u8) {
        self.current_ram_bank = data & 0x3;
    }

    fn change_rom_ram_mode(&mut self, data: u8) {
        self.is_rom_banking = data & 0x1 == 0;
        if self.is_rom_banking {
            // Gameboy can only use bank 0 in this mode
            self.current_ram_bank = 0;
        }
    }
}

impl Mbc for Mbc1 {
    fn readb(&self, addr: u16) -> u8 {
        if (0xA000..=0xBFFF).contains(&addr) {
            return self.ram_banks
                [((addr - 0xA000) + (self.current_ram_bank as u16) * RAM_BANK_SIZE) as usize];
        }

        let real_addr = match addr {
            0x4000..=0x7FFF => {
                (addr - ROM_BANK_SIZE) + ((self.current_rom_bank as u16) * ROM_BANK_SIZE) as u16
            }
            _ => addr,
        } as usize;

        *self
            .rom
            .get(real_addr)
            .expect(&format!("Cannot access cartridge memory at {:#08x}", real_addr)[..])
    }

    fn writeb(&mut self, addr: u16, value: u8) {
        if (0xA000..0xC000).contains(&addr) {
            // Cartridge RAM bank write
            self.ram_banks[(addr - 0xA000) as usize] = value;
        } else if addr < 0x2000 {
            self.toggle_ram_banking(value);
        } else if addr < 0x4000 {
            // ROM bank change
            self.change_low_rom_bank(value);
        } else if addr < 0x6000 {
            if self.is_rom_banking {
                self.change_hi_rom_bank(value);
            } else {
                self.change_ram_bank(value);
            }
        } else if addr < 0x8000 {
            self.change_rom_ram_mode(value);
        }
    }
}
