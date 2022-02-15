use std::env;

mod cpu
{
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
        sp: u16
    }

    impl Registers {
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
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let rom = std::fs::read(&args[1]);
    println!("Hello, world!");
}
