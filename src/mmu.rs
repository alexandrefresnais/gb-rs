use crate::lcd;
use crate::cartridge::Cartridge;
use crate::to_u16;
use crate::to_u8;

pub struct Mmu<'a> {
    cartridge: &'a mut Cartridge,
    pub memory: [u8; 0x10000],
}

impl<'a> Mmu<'a> {
    pub fn new(cartridge: &'a mut Cartridge) -> Mmu<'a> {
        let mut mmu = Mmu {
            cartridge,
            memory: [0; 0x10000],
        };

        mmu.memory[0xFF05] = 0x00;
        mmu.memory[0xFF06] = 0x00;
        mmu.memory[0xFF07] = 0x00;
        mmu.memory[0xFF10] = 0x80;
        mmu.memory[0xFF11] = 0xBF;
        mmu.memory[0xFF12] = 0xF3;
        mmu.memory[0xFF14] = 0xBF;
        mmu.memory[0xFF16] = 0x3F;
        mmu.memory[0xFF17] = 0x00;
        mmu.memory[0xFF19] = 0xBF;
        mmu.memory[0xFF1A] = 0x7F;
        mmu.memory[0xFF1B] = 0xFF;
        mmu.memory[0xFF1C] = 0x9F;
        mmu.memory[0xFF1E] = 0xBF;
        mmu.memory[0xFF20] = 0xFF;
        mmu.memory[0xFF21] = 0x00;
        mmu.memory[0xFF22] = 0x00;
        mmu.memory[0xFF23] = 0xBF;
        mmu.memory[0xFF24] = 0x77;
        mmu.memory[0xFF25] = 0xF3;
        mmu.memory[0xFF26] = 0xF1;
        mmu.memory[0xFF40] = 0x91;
        mmu.memory[0xFF42] = 0x00;
        mmu.memory[0xFF43] = 0x00;
        mmu.memory[0xFF45] = 0x00;
        mmu.memory[0xFF47] = 0xFC;
        mmu.memory[0xFF48] = 0xFF;
        mmu.memory[0xFF49] = 0xFF;
        mmu.memory[0xFF4A] = 0x00;
        mmu.memory[0xFF4B] = 0x00;
        mmu.memory[0xFFFF] = 0x00;

        mmu
    }

    pub fn readb(&self, addr: u16) -> u8 {
        if addr < 0x8000 || (0xA000..=0xBFFF).contains(&addr) {
            self.cartridge.readb(addr)
        } else {
            self.memory[addr as usize]
        }
    }

    pub fn readw(&self, addr: u16) -> u16 {
        let lsb = self.readb(addr);
        let msb = self.readb(addr + 1);

        to_u16(msb, lsb)
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        match addr as usize {
            0..=0x7fff | 0xa000..=0xbfff => self.cartridge.writeb(addr, value),
            0xe000..=0xfdff => {
                self.memory[addr as usize] = value;
                self.writeb(addr - 0x2000, value)
            } // ECHO RAM
            0xfea0..=0xfeff => (), // Restricted
            lcd::SCANLINE_REGISTER => self.memory[lcd::SCANLINE_REGISTER] = 0, // Reset if write
            n => self.memory[n] = value,
        };
    }

    pub fn writew(&mut self, addr: u16, value: u16) {
        let (msb, lsb) = to_u8(value);

        self.writeb(addr, lsb);
        self.writeb(addr + 1, msb);
    }
}
