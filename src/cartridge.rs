use crate::mbc0::NoMbc;
use crate::mbc1::Mbc1;
use crate::mbc3::Mbc3;

const MBC_REGISTER: u16 = 0x147;
pub const ROM_BANK_SIZE: u16 = 0x4000;
pub const RAM_BANK_SIZE: u16 = 0x2000;

pub trait Mbc {
    fn readb(&self, addr: u16) -> u8;
    fn writeb(&mut self, addr: u16, value: u8);
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

        let mbc: Box<dyn Mbc> = match mbc_type {
            0 => NoMbc::new(rom),
            1 | 2 | 3 => Mbc1::new(rom),
            0x13 => Mbc3::new(rom),
            _ => panic!("Unsupported MBC"),
        };

        Cartridge { mbc }
    }

    pub fn readb(&self, addr: u16) -> u8 {
        self.mbc.readb(addr)
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        self.mbc.writeb(addr, value);
    }
}
