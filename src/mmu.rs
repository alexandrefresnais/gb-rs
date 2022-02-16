pub struct MMU {
    rom: Vec<u8>,
    pub memory: [u8; 0x10000]
}

use super::lcd;

impl MMU {
    pub fn new(rom: Vec<u8>) -> MMU {
        MMU { rom: rom, memory:  [0; 0x10000]}
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