use crate::cartridge::Mbc;
use crate::cartridge::RAM_BANK_SIZE;
use crate::cartridge::ROM_BANK_SIZE;

pub struct Mbc3 {
    rom: Vec<u8>,
    is_ram_enabled: bool,
    current_rom_bank: u8,
    current_ram_bank: u8,
    ram_banks: [u8; 0x10000],
}

impl Mbc3 {
    pub fn new(rom: Vec<u8>) -> Box<Self> {
        Box::new(Mbc3 {
            rom,
            is_ram_enabled: false,
            current_rom_bank: 1,
            current_ram_bank: 0,
            ram_banks: [0; 0x10000], // Max 64KB of RAM
        })
    }

    fn toggle_ram_banking(&mut self, value: u8) {
        // RAM banking is only enabled if four first bits of value written is 0xA
        self.is_ram_enabled = (value & 0xf) == 0xA;
    }

    fn change_rom_bank(&mut self, value: u8) {
        // Replace 7 lowest bits with value's ones

        self.current_rom_bank = match value & 0x7f {
            0 => 1,
            n => n,
        };
    }
}

// Timer less implementation of MBC3
impl Mbc for Mbc3 {
    fn readb(&self, addr: u16) -> u8 {
        if (0xA000..=0xBFFF).contains(&addr) {
            if self.current_ram_bank <= 3 {
                return self.ram_banks
                    [((addr - 0xA000) + (self.current_ram_bank as u16) * RAM_BANK_SIZE) as usize];
            } else {
                return 0;
            }
        }

        let real_addr = match addr {
            0x4000..=0x7FFF => {
                (addr - ROM_BANK_SIZE) as u32
                    + ((self.current_rom_bank as u32) * ROM_BANK_SIZE as u32) as u32
            }
            _ => addr as u32,
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
            self.change_rom_bank(value);
        } else if addr < 0x6000 {
            self.current_ram_bank = value;
        }
    }
}
