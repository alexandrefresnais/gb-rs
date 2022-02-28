const MBC_REGISTER: u16 = 0x147;
const ROM_BANK_SIZE: u16 = 0x4000;
const RAM_BANK_SIZE: u16 = 0x2000;
const NB_RAM_BANK: u16 = 4;
const RAM_SIZE: usize = (NB_RAM_BANK * RAM_BANK_SIZE) as usize;

pub struct Cartridge {
    rom: Vec<u8>,
    ram_banks: [u8; RAM_SIZE],
}

impl Cartridge {
    pub fn new(filename: &str) -> Self {
        let cartridge = Cartridge {
            rom: std::fs::read(filename).expect("Cannot read rom"),
            ram_banks: [0; RAM_SIZE],
        };

        cartridge
    }

    pub fn readb(&self, addr: u16) -> u8 {
        *self.rom.get(addr as usize).expect(&format!(
            "Cannot access cartridge memory at {:#08x}",
            addr
        )[..])
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        if (0xA000..0xC000).contains(&addr) {
            // Cartridge RAM bank write
            self.ram_banks[(addr - 0xA000) as usize] = value;
        }
    }
}
