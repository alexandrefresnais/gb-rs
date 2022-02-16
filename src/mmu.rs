pub struct MMU {
    rom: Vec<u8>,
    pub memory: [u8; 0x10000]
}

use super::lcd;

impl MMU {
    pub fn new(rom: Vec<u8>) -> MMU {
        let mut mmu = MMU { rom: rom, memory:  [0; 0x10000]};
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
        if addr < 0x8000 {
            *self.rom.get(addr as usize).unwrap()
        } else {
            self.memory[addr as usize]
        }
    }

    pub fn readw(&self, addr: u16) -> u16 {
        let lsb = if addr < 0x8000 {
            *self.rom.get(addr as usize).unwrap()
        } else {
            self.memory[addr as usize]
        };

        let msb = if addr < 0x8000 {
            *self.rom.get((addr + 1) as usize).unwrap()
        } else {
            self.memory[(addr + 1) as usize]
        };

        super::to_u16(msb, lsb)
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        if addr < 0x8000 {
            self.rom[addr as usize] = value;
        } else if addr == lcd::SCANLINE_REGISTER as u16 {
            // Reset the current scanline index if the game tries to write to it
            self.memory[addr as usize] = 0 ;
        } else {
            self.memory[addr as usize] = value;
        }
    }

    pub fn writew(&mut self, addr: u16, value: u16) {
        let (msb, lsb) = super::to_u8(value);

        if addr < 0x8000 {
            self.rom[addr as usize] = lsb as u8;
            self.rom[(addr + 1) as usize] = msb as u8;
        }
        else {
            self.memory[addr as usize] = lsb as u8;
            self.memory[(addr + 1) as usize] = msb as u8;
        }
    }
}