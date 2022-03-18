use crate::utils::Bits;

pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: u8,
    pub h: u8,
    pub l: u8,
    pub pc: u16,
    pub sp: u16,
}

impl Default for Registers {
    fn default() -> Self {
        let mut registers = Registers {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: 0,
            h: 0,
            l: 0,
            pc: 0x100,
            sp: 0xfffe,
        };
        registers.set_af(0x01b0);
        registers.set_bc(0x13);
        registers.set_de(0xd8);
        registers.set_hl(0x14d);
        registers
    }
}

impl Registers {
    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | self.f as u16
    }

    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | self.c as u16
    }

    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | self.e as u16
    }

    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | self.l as u16
    }

    pub fn set_af(&mut self, value: u16) {
        self.a = ((value & 0xff00) >> 8) as u8;
        self.f = (value & 0xf0) as u8;
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = ((value & 0xff00) >> 8) as u8;
        self.c = (value & 0xff) as u8;
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = ((value & 0xff00) >> 8) as u8;
        self.e = (value & 0xff) as u8;
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = ((value & 0xff00) >> 8) as u8;
        self.l = (value & 0xff) as u8;
    }

    pub fn set_z(&mut self, value: bool) {
        if value {
            self.f = self.f.set_bit(7);
        } else {
            self.f = self.f.unset_bit(7);
        }
    }

    pub fn set_n(&mut self, value: bool) {
        if value {
            self.f = self.f.set_bit(6);
        } else {
            self.f = self.f.unset_bit(6);
        }
    }

    pub fn set_h(&mut self, value: bool) {
        if value {
            self.f = self.f.set_bit(5);
        } else {
            self.f = self.f.unset_bit(5);
        }
    }

    pub fn set_c(&mut self, value: bool) {
        if value {
            self.f = self.f.set_bit(4);
        } else {
            self.f = self.f.unset_bit(4);
        }
    }

    pub fn get_z(&self) -> bool {
        self.f.is_set(7)
    }

    pub fn get_n(&self) -> bool {
        self.f.is_set(6)
    }

    pub fn get_h(&self) -> bool {
        self.f.is_set(5)
    }

    pub fn get_c(&self) -> bool {
        self.f.is_set(4)
    }
}