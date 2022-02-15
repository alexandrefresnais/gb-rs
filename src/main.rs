use std::env;

mod cpu {
    struct Registers {
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        h: u8,
        l: u8,
        pc: u16,
        sp: u16,
    }

    impl Registers {
        fn new() -> Registers {
            Registers {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
                e: 0,
                f: 0,
                h: 0,
                l: 0,
                pc: 0x100,
                sp: 0,
            }
        }

        fn af(&self) -> u16 {
            ((self.a as u16) << 8) | self.f as u16
        }

        fn bc(&self) -> u16 {
            ((self.b as u16) << 8) | self.c as u16
        }

        fn de(&self) -> u16 {
            ((self.d as u16) << 8) | self.e as u16
        }

        fn hl(&self) -> u16 {
            ((self.h as u16) << 8) | self.l as u16
        }

        fn set_af(&mut self, value: u16) {
            self.a = ((value & 0xff00) >> 8) as u8;
            self.f = (value & 0xff) as u8;
        }

        fn set_bc(&mut self, value: u16) {
            self.b = ((value & 0xff00) >> 8) as u8;
            self.c = (value & 0xff) as u8;
        }

        fn set_de(&mut self, value: u16) {
            self.d = ((value & 0xff00) >> 8) as u8;
            self.e = (value & 0xff) as u8;
        }

        fn set_hl(&mut self, value: u16) {
            self.h = ((value & 0xff00) >> 8) as u8;
            self.l = (value & 0xff) as u8;
        }

        fn set_z(&mut self, value: bool) {
            if value {
                self.f = self.f | 0b1000000;
            } else {
                self.f = self.f & 0b0111111;
            }
        }

        fn set_n(&mut self, value: bool) {
            if value {
                self.f = self.f | 0b100000;
            } else {
                self.f = self.f & 0b011111;
            }
        }

        fn set_h(&mut self, value: bool) {
            if value {
                self.f = self.f | 0b10000;
            } else {
                self.f = self.f & 0b01111;
            }
        }

        fn set_c(&mut self, value: bool) {
            if value {
                self.f = self.f | 0b1000;
            } else {
                self.f = self.f & 0b0111;
            }
        }
    }

    pub struct CPU {
        reg: Registers,
    }

    impl CPU {
        pub fn new() -> CPU {
            CPU {
                reg: Registers::new(),
            }
        }

        fn load_at(&self, mmu: &super::mmu::MMU, address: u16, value: u8) {
            // LD (address), value
        }

        pub fn run_cycle(&mut self, mmu: &mut super::mmu::MMU) -> u8 {
            // In this implementation, the CPU will give the number of cycle
            // to run on other components
            let opcode = self.readb(mmu);

            match opcode {
                0x00 => 4, // NOP
                0x01 => { let w = self.readw(mmu); self.reg.set_bc(w); 12 }, // LD BC, n16
                0x02 => { let bc = self.reg.bc(); let a = self.reg.a; self.load_at(mmu, bc, a); 8 }, // LD (BC), A
                0x03 => { let bc = self.reg.bc(); self.reg.set_bc(bc + 1); 8 }, // INC BC
                0x04 => { self.reg.b = self.inc(self.reg.b); 4 }, // INC B
                0x05 => { self.reg.b = self.dec(self.reg.b); 4 }, // DEC B
                0x0b => { let bc = self.reg.bc(); self.reg.set_bc(bc - 1); 8 }, // DEC BC
                0x0c => { self.reg.c = self.inc(self.reg.c); 4 }, // INC C
                0x0d => { self.reg.c = self.dec(self.reg.c); 4 }, // DEC C
                0x11 => { let w = self.readw(mmu); self.reg.set_de(w); 12 }, // LD DE, n16
                0x12 => { let de = self.reg.de(); let a = self.reg.a; self.load_at(mmu, de, a); 8 }, // LD (DE), A
                0x13 => { let de = self.reg.de(); self.reg.set_de(de + 1); 8 }, // INC DE
                0x14 => { self.reg.d = self.inc(self.reg.d); 4 }, // INC D
                0x15 => { self.reg.d = self.dec(self.reg.d); 4 }, // DEC D
                0x1b => { let de = self.reg.de(); self.reg.set_de(de - 1); 8 }, // DEC DE
                0x1c => { self.reg.e = self.inc(self.reg.e); 4 }, // INC E
                0x1d => { self.reg.e = self.dec(self.reg.e); 4 }, // DEC E
                0x21 => { let w = self.readw(mmu); self.reg.set_hl(w); 12 }, // LD HL, n16
                0x22 => { let hl = self.reg.hl(); let a = self.reg.a; self.load_at(mmu, hl, a); self.inc_hl(); 8 }, // LD (HL+), A
                0x23 => { self.inc_hl(); 8 }, // INC HL
                0x24 => { self.reg.h = self.inc(self.reg.h); 4 }, // INC H
                0x25 => { self.reg.h = self.dec(self.reg.h); 4 }, // DEC H
                0x2b => { self.dec_hl(); 8 }, // DEC HL
                0x2c => { self.reg.l = self.inc(self.reg.l); 4 }, // INC L
                0x2d => { self.reg.l = self.dec(self.reg.l); 4 }, // DEC L
                0x31 => { self.reg.sp = self.readw(mmu); 12 }, // LD SP, n16
                0x32 => { let hl = self.reg.hl(); let a = self.reg.a; self.load_at(mmu, hl, a); self.dec_hl(); 8 }, // LD (HL-), A
                0x33 => { self.reg.sp += 1; 8 }, // INC SP
                0x34 => { let hl = self.reg.hl(); self.inc_at(mmu, hl); 12 }, // INC (HL)
                0x35 => { let hl = self.reg.hl(); self.dec_at(mmu, hl); 12 }, // DEC (HL)
                0x3b => { self.reg.sp -= 1; 8 }, // DEC SP
                0x3c => { self.reg.a = self.inc(self.reg.a); 4 }, // INC A
                0x3d => { self.reg.a = self.dec(self.reg.a); 4 }, // DEC A
                0xc3 => { self.reg.pc = self.readw(mmu); 16 }, // JP u16
                _ => panic!("Unknown opcode {:#04x} at {:#04x}.", opcode, self.reg.pc - 1)
            }
        }

        fn inc(&mut self, value: u8) -> u8 {
            // Returns value + 1
            // Sets Z, N, and H
            let res = value.wrapping_add(1);
            self.reg.set_z(res == 0);
            self.reg.set_n(false);
            self.reg.set_h((value & 0x0f) + 1 > 0x0f);
            res
        }

        fn dec(&mut self, value: u8) -> u8 {
            // Returns value - 1
            // Sets Z, N, and H
            let res = value.wrapping_sub(1);
            self.reg.set_z(res == 0);
            self.reg.set_n(true);
            self.reg.set_h((value & 0x0f) == 0);
            res
        }

        fn inc_at(&mut self, mmu: &mut super::mmu::MMU, address: u16) {
            // INC (address)
            // Sets Z, N, and H
            let value = mmu.readb(address);
            let res = value.wrapping_add(1);
            self.reg.set_z(res == 0);
            self.reg.set_n(false);
            self.reg.set_h((value & 0x0f) + 1 > 0x0f);
            mmu.writeb(address, res);
        }

        fn dec_at(&mut self, mmu: &mut super::mmu::MMU, address: u16) {
            // DEC (address)
            // Sets Z, N, and H
            let value = mmu.readb(address);
            let res = value.wrapping_sub(1);
            self.reg.set_z(res == 0);
            self.reg.set_n(true);
            self.reg.set_h((value & 0x0f) == 0);
            mmu.writeb(address, res);
        }

        fn inc_hl(&mut self) {
            let hl = self.reg.hl();
            self.reg.set_hl(hl + 1);
        }

        fn dec_hl(&mut self) {
            let hl = self.reg.hl();
            self.reg.set_hl(hl - 1);
        }

        fn readb(&mut self, mmu: &super::mmu::MMU) -> u8 {
            let byte = mmu.readb(self.reg.pc);
            self.reg.pc += 1;
            byte
        }

        fn readw(&mut self, mmu: &super::mmu::MMU) -> u16 {
            let word = mmu.readw(self.reg.pc);
            self.reg.pc += 2;
            word
        }
    }
}

mod mmu {
    pub struct MMU {
        rom: Vec<u8>,
    }

    impl MMU {
        pub fn new(rom: Vec<u8>) -> MMU {
            MMU { rom: rom }
        }

        pub fn readb(&self, addr: u16) -> u8 {
            if addr < 0x8000 {
                *self.rom.get(addr as usize).unwrap()
            } else {
                0
            }
        }

        pub fn readw(&self, addr: u16) -> u16 {
            if addr < 0x8000 {
                let lsb = *self.rom.get(addr as usize).unwrap();
                let msb = *self.rom.get((addr + 1) as usize).unwrap();
                ((msb as u16) << 8) | lsb as u16
            } else {
                0
            }
        }

        pub fn writeb(&mut self, addr: u16, value: u8) {
            if addr < 0x8000 {
                panic!("Attempting write access to read-only memory!");
            }
        }

        pub fn writew(&mut self, addr: u16, value: u16) {
            if addr < 0x8000 {
                panic!("Attempting write access to read-only memory!");
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = std::fs::read(&args[1]);
    let mut mmu = mmu::MMU::new(rom.unwrap());
    let mut cpu = cpu::CPU::new();

    loop {
        cpu.run_cycle(&mut mmu);
    }
}
