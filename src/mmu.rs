use crate::cartridge::Cartridge;
use crate::joypad::Joypad;
use crate::joypad::JOYPAD_REGISTER;
use crate::lcd::Lcd;
use crate::lcd::CONTROL_REGISTER;
use crate::lcd::WINDOW_X_REGISTER;
use crate::lcd::OAM_START;
use crate::lcd::OAM_END;
use crate::lcd::VRAM_START;
use crate::lcd::VRAM_END;
use crate::timer::Timer;
use crate::timer::DIVIDER_REGISTER;
use crate::timer::TIMA;
use crate::timer::TMA;
use crate::timer::TMC;
use crate::utils::to_u16;
use crate::utils::to_u8;

const INT_REQUEST_REGISTER: u16 = 0xFF0F; // Interupt Request Register
const INT_ENABLED_REGISTER: u16 = 0xFFFF; // Interupt Enabled Register

// TODO: make an interupt register object
pub struct Mmu<'a> {
    cartridge: &'a mut Cartridge,
    pub memory: [u8; 0x10000],
    pub joypad: Joypad,
    pub lcd: Lcd,
    pub timer: Timer,
    pub int_request: u8, // Interupt Request Register
    pub int_enabled: u8,
}

impl<'a> Mmu<'a> {
    pub fn new(cartridge: &'a mut Cartridge) -> Mmu<'a> {
        let mut mmu = Mmu {
            cartridge,
            memory: [0; 0x10000],
            joypad: Joypad::new(),
            lcd: Lcd::new(),
            timer: Timer::default(),
            int_request: 0,
            int_enabled: 0,
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
        match addr {
            0..=0x7fff | 0xA000..=0xBFFF => self.cartridge.readb(addr),
            DIVIDER_REGISTER | TIMA | TMA | TMC => self.timer.readb(addr),
            VRAM_START..=VRAM_END => self.lcd.readb(addr),
            OAM_START..=OAM_END => self.lcd.readb(addr),
            CONTROL_REGISTER..=WINDOW_X_REGISTER => self.lcd.readb(addr),
            JOYPAD_REGISTER => self.joypad.readb(addr),
            INT_REQUEST_REGISTER => self.int_request,
            INT_ENABLED_REGISTER => self.int_enabled,
            _ => self.memory[addr as usize],
        }
    }

    pub fn readw(&self, addr: u16) -> u16 {
        let lsb = self.readb(addr);
        let msb = self.readb(addr + 1);

        to_u16(msb, lsb)
    }

    pub fn writeb(&mut self, addr: u16, value: u8) {
        match addr {
            0..=0x7fff | 0xa000..=0xbfff => self.cartridge.writeb(addr, value),
            0xe000..=0xfdff => {
                self.memory[addr as usize] = value;
                self.writeb(addr - 0x2000, value)
            } // ECHO RAM
            DIVIDER_REGISTER | TIMA | TMA | TMC => self.timer.writeb(addr, value),
            0xfea0..=0xfeff => (), // Restricted
            0xff46 => self.do_dma(value),
            VRAM_START..=VRAM_END => self.lcd.writeb(addr, value),
            OAM_START..=OAM_END => self.lcd.writeb(addr, value),
            CONTROL_REGISTER..=WINDOW_X_REGISTER => self.lcd.writeb(addr, value),
            JOYPAD_REGISTER => self.joypad.writeb(addr, value),
            INT_REQUEST_REGISTER => self.int_request = value,
            INT_ENABLED_REGISTER => self.int_enabled = value,
            n => self.memory[n as usize] = value,
        };
    }

    pub fn writew(&mut self, addr: u16, value: u16) {
        let (msb, lsb) = to_u8(value);

        self.writeb(addr, lsb);
        self.writeb(addr + 1, msb);
    }

    fn do_dma(&mut self, value: u8) {
        let base = (value as u16) << 8;
        for i in 0..0xA0 {
            let b = self.readb(base + i);
            self.writeb(0xFE00 + i, b);
        }
    }

    pub fn update(&mut self, cycles: u32) {
        self.timer.update(cycles);
        self.int_request |= self.timer.int_request;
        self.timer.int_request = 0;
        self.lcd.update_graphics(cycles);
        self.int_request |= self.lcd.int_request;
        self.lcd.int_request = 0;
        self.int_request |= self.joypad.int_request;
        self.joypad.int_request = 0;
    }
}
