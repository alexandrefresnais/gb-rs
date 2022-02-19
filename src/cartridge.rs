const MBC_REGISTER: u16 = 0x147;
const ROM_BANK_SIZE: u16 = 0x4000;
const RAM_BANK_SIZE: u16 = 0x2000;
const NB_RAM_BANK: u16 = 4;
const RAM_SIZE: usize = (NB_RAM_BANK * RAM_BANK_SIZE) as usize;

use crate::utils::Bits;

// A000-BFFF RAM bank from cartridge if any (switchable)

// MBC is the mode for ROM banking
enum MBC {
    MBC1,
    MBC2,
}

pub struct Cartridge {
    rom: Vec<u8>,
    mbc: Option<MBC>,
    current_rom_bank: u8,
    ram_banks: [u8; RAM_SIZE],
    current_ram_bank: u8,
    is_ram_enabled: bool,
    is_rom_banking: bool,
}

impl Cartridge {
    pub fn new(filename: &str) -> Self {
        let mut cartridge = Cartridge {
            rom: std::fs::read(filename).expect("Cannot read rom"),
            mbc: None,
            current_rom_bank: 1,
            ram_banks: [0; RAM_SIZE],
            current_ram_bank: 0,
            is_ram_enabled: false,
            is_rom_banking: true,
        };

        cartridge.mbc = match cartridge
            .rom
            .get(MBC_REGISTER as usize)
            .expect("Could not read MBC register")
        {
            1..=3 => Some(MBC::MBC1),
            4 | 5 => Some(MBC::MBC2),
            _ => None
        };

        cartridge
    }

    pub fn readb(&self, addr: u16) -> u8 {
        let real_addr = match addr {
            0x4000..=0x7FFF => {
                (addr - ROM_BANK_SIZE) + ((self.current_rom_bank as u16) * ROM_BANK_SIZE) as u16
            }
            0xA000..=0xBFFF => (addr - 0xa000) + (self.current_ram_bank as u16) * RAM_BANK_SIZE,
            _ => addr,
        } as usize;

        *self.rom.get(real_addr).expect(&format!(
            "Cannot access cartridge memory at {:#08x}",
            real_addr
        )[..])
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        if (0xA000..0xC000).contains(&addr) {
            // Cartridge RAM bank write
            if self.is_ram_enabled {
                let real_address = (addr - 0xa000) + (self.current_ram_bank as u16) * RAM_BANK_SIZE;
                self.ram_banks[real_address as usize] = value;
            }
        } else {
            self.handle_banking(addr, value);
        }
    }

    fn handle_banking(&mut self, addr: u16, value: u8) {
        // No MBC means no banking
        if self.mbc.is_none() {
            return;
        }

        if addr < 0x2000 {
            // RAM enabling
            self.enable_ram_banking(addr, value);
        } else if addr < 0x4000 {
            // ROM bank change
            self.change_low_rom_bank(value);
        } else if addr < 0x6000 {
            // ROM or RAM bank change (if MBC1)
            if let Some(MBC::MBC1) = self.mbc {
                if self.is_rom_banking {
                    self.change_hi_rom_bank(value);
                } else {
                    self.change_ram_bank(value);
                }
            }
        } else if addr < 0x8000 {
            if let Some(MBC::MBC1) = self.mbc {
                self.change_rom_ram_mode(value);
            }
        }
    }

    fn enable_ram_banking(&mut self, addr: u16, data: u8) {
        // Writing to specific memory addresses may lead to
        // enabling ram banking
        // addr is the address the game was about to write
        // and data the value which should have been written

        if let Some(MBC::MBC2) = self.mbc {
            // In case of MBC2, bit 4 must not be set
            if addr.is_set(4) {
                return;
            }
        }

        let test_data = data & 0xf;
        if test_data == 0xa {
            self.is_ram_enabled = true;
        } else if test_data == 0 {
            self.is_ram_enabled = false;
        }
    }

    fn change_low_rom_bank(&mut self, data: u8) {
        // Change the 5 lowest bits of self.current_rom_bank
        if let Some(MBC::MBC2) = self.mbc {
            self.current_rom_bank = match data & 0xF {
                0 => 1,
                n => n,
            };
            return;
        }

        // Replace 5 lowest bits with data's ones
        let lower5 = data & 0b11111;
        self.current_rom_bank &= 0b11100000;
        self.current_rom_bank |= lower5;

        if self.current_rom_bank == 0 {
            self.current_rom_bank = 1;
        }
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
