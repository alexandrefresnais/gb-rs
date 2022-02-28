const MBC_REGISTER: u16 = 0x147;
const ROM_BANK_SIZE: u16 = 0x4000;
const RAM_BANK_SIZE: u16 = 0x2000;

trait Mbc {
    fn readb(&self, addr: u16) -> u8;
    fn writeb(&mut self, addr: u16, value: u8);
}

struct NoMbc {
    rom: Vec<u8>,
}

impl NoMbc {
    fn new(rom: Vec<u8>) -> Box<Self> {
        Box::new(NoMbc { rom: rom })
    }
}

impl Mbc for NoMbc {
    fn readb(&self, addr: u16) -> u8 {
        *self
            .rom
            .get(addr as usize)
            .expect(&format!("Cannot access cartridge memory at {:#08x}", addr)[..])
    }

    fn writeb(&mut self, _addr: u16, _value: u8) {
        // No write is supposed to happen.
    }
}

pub struct Cartridge {
    mbc: Box<dyn Mbc>,
}

impl Cartridge {
    pub fn new(filename: &str) -> Self {
        let rom = std::fs::read(filename).expect("Cannot read rom");

        let mbc_type = rom
            .get(MBC_REGISTER as usize)
            .expect("Could not read MBC register");

        let mbc = match mbc_type {
            0 => NoMbc::new(rom),
            _ => panic!("Unsupported MBC"),
        };

        let cartridge = Cartridge {
            mbc,
        };

        cartridge
    }

    pub fn readb(&self, addr: u16) -> u8 {
        self.mbc.readb(addr)
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        self.mbc.writeb(addr, value);
    }
}
