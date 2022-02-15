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
                pc: 0,
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

        pub fn run_cycle(&mut self, mmu: super::mmu::MMU) -> u8 {
            // In this implementation, the CPU will give the number of cycle
            // to run on other components
            let opcode = mmu.readb(self.reg.pc);
            self.reg.pc += 1;
            4
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
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = std::fs::read(&args[1]);
    let mut mmu = mmu::MMU::new(rom.unwrap());
    let mut cpu = cpu::CPU::new();
    cpu.run_cycle(mmu);
    println!("Hello, world!");
}
