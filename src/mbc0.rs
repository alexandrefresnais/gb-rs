use crate::cartridge::Mbc;

pub struct NoMbc {
    rom: Vec<u8>,
}

impl NoMbc {
    pub fn new(rom: Vec<u8>) -> Box<Self> {
        Box::new(NoMbc { rom })
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
