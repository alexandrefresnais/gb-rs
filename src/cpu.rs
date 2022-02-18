use crate::registers::Registers;
use crate::mmu::Mmu;
use crate::to_u8;
use crate::to_u16;

pub struct Cpu {
    reg: Registers,
    ime: bool
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: Registers::new(),
            ime: false,
        }
    }

    pub fn run_cycle(&mut self, mmu: &mut Mmu) -> u8 {
        // In this implementation, the Cpu will give the number of cycle
        // to run on other components
        let opcode = self.readb(mmu);

        match opcode {
            0x00 => 4, // NOP
            0x01 => { let w = self.readw(mmu); self.reg.set_bc(w); 12 }, // LD BC, n16
            0x02 => { mmu.writeb(self.reg.bc(), self.reg.a); 8 }, // LD (BC), A
            0x03 => { let bc = self.reg.bc(); self.reg.set_bc(bc + 1); 8 }, // INC BC
            0x04 => { self.reg.b = self.inc(self.reg.b); 4 }, // INC B
            0x05 => { self.reg.b = self.dec(self.reg.b); 4 }, // DEC B
            0x06 => { self.reg.b = self.readb(mmu); 8 }, // LD B, u8
            0x08 => { let addr = self.readw(mmu); mmu.writew(addr, self.reg.sp); 20 }, // LD (u16), SP
            0x0a => { let bc = self.reg.bc(); self.reg.a = mmu.readb(bc); 8 }, // LD A, (BC)
            0x0b => { let bc = self.reg.bc(); self.reg.set_bc(bc - 1); 8 }, // DEC BC
            0x0c => { self.reg.c = self.inc(self.reg.c); 4 }, // INC C
            0x0d => { self.reg.c = self.dec(self.reg.c); 4 }, // DEC C
            0x0e => { self.reg.c = self.readb(mmu); 8 }, // LD C, u8
            0x10 => 4, // STOP
            0x11 => { let w = self.readw(mmu); self.reg.set_de(w); 12 }, // LD DE, n16
            0x12 => { mmu.writeb(self.reg.de(), self.reg.a); 8 }, // LD (DE), A
            0x13 => { let de = self.reg.de(); self.reg.set_de(de + 1); 8 }, // INC DE
            0x14 => { self.reg.d = self.inc(self.reg.d); 4 }, // INC D
            0x15 => { self.reg.d = self.dec(self.reg.d); 4 }, // DEC D
            0x16 => { self.reg.d = self.readb(mmu); 8 }, // LD D, u8
            0x18 => { self.jr(mmu); 12 }, // JR i8
            0x1a => { let de = self.reg.de(); self.reg.a = mmu.readb(de); 8 }, // LD A, (DE)
            0x1b => { let de = self.reg.de(); self.reg.set_de(de - 1); 8 }, // DEC DE
            0x1c => { self.reg.e = self.inc(self.reg.e); 4 }, // INC E
            0x1d => { self.reg.e = self.dec(self.reg.e); 4 }, // DEC E
            0x1e => { self.reg.e = self.readb(mmu); 8 }, // LD E, u8
            0x20 => { if self.reg.get_z() { 8 } else { self.jr(mmu); 12 } } // JR NZ, i8
            0x21 => { let w = self.readw(mmu); self.reg.set_hl(w); 12 }, // LD HL, n16
            0x22 => { mmu.writeb(self.reg.hl(), self.reg.a); self.inc_hl(); 8 }, // LD (HL+), A
            0x23 => { self.inc_hl(); 8 }, // INC HL
            0x24 => { self.reg.h = self.inc(self.reg.h); 4 }, // INC H
            0x25 => { self.reg.h = self.dec(self.reg.h); 4 }, // DEC H
            0x26 => { self.reg.h = self.readb(mmu); 8 }, // LD H, u8
            0x28 => { if self.reg.get_z() { self.jr(mmu); 12 } else { 8 } } // JR Z, i8
            0x2a => { let hl = self.reg.hl(); self.inc_hl(); self.reg.a = mmu.readb(hl); 8 }, // LD A, (HL+)
            0x2b => { self.dec_hl(); 8 }, // DEC HL
            0x2c => { self.reg.l = self.inc(self.reg.l); 4 }, // INC L
            0x2d => { self.reg.l = self.dec(self.reg.l); 4 }, // DEC L
            0x2e => { self.reg.l = self.readb(mmu); 8 }, // LD L, u8
            0x2f => { self.cpl(); 4 }, // CPL
            0x30 => { if self.reg.get_c() { 8 } else { self.jr(mmu); 12 } } // JR NC, i8
            0x31 => { self.reg.sp = self.readw(mmu); 12 }, // LD SP, n16
            0x32 => { mmu.writeb(self.reg.hl(), self.reg.a); self.dec_hl(); 8 }, // LD (HL-), A
            0x33 => { self.reg.sp += 1; 8 }, // INC SP
            0x34 => { let hl = self.reg.hl(); self.inc_at(mmu, hl); 12 }, // INC (HL)
            0x35 => { let hl = self.reg.hl(); self.dec_at(mmu, hl); 12 }, // DEC (HL)
            0x36 => { mmu.writeb(self.reg.hl(), self.reg.a); 8 }, // LD (HL), u8
            0x37 => { self.scf(); 4 }, // SCF
            0x38 => { if self.reg.get_c() { self.jr(mmu); 12 } else { 8 } }, // JR C, i8
            0x3a => { let hl = self.reg.hl(); self.dec_hl(); self.reg.a = mmu.readb(hl); 8 }, // LD A, (HL-)
            0x3b => { self.reg.sp -= 1; 8 }, // DEC SP
            0x3c => { self.reg.a = self.inc(self.reg.a); 4 }, // INC A
            0x3d => { self.reg.a = self.dec(self.reg.a); 4 }, // DEC A

            0x3e => { self.reg.a = self.readb(mmu); 8 }, // LD A, u8
            0x3f => { self.reg.set_n(false); self.reg.set_h(false); let c = self.reg.get_c(); self.reg.set_c(!c); 4 }, // CCF
            0x40 => { 4 }, // LD B, B
            0x41 => { self.reg.b = self.reg.c; 4 }, // LD B, C
            0x42 => { self.reg.b = self.reg.d; 4 }, // LD B, D
            0x43 => { self.reg.b = self.reg.e; 4 }, // LD B, E
            0x44 => { self.reg.b = self.reg.h; 4 }, // LD B, H
            0x45 => { self.reg.b = self.reg.l; 4 }, // LD B, L
            0x46 => { self.reg.b = mmu.readb(self.reg.hl()); 8 }, // LD B, (HL)
            0x47 => { self.reg.b = self.reg.a; 4 }, // LD B, A
            0x48 => { self.reg.c = self.reg.b; 4 }, // LD C, B
            0x49 => { 4 }, // LD C, C
            0x4a => { self.reg.c = self.reg.d; 4 }, // LD C, D
            0x4b => { self.reg.c = self.reg.e; 4 }, // LD C, E
            0x4c => { self.reg.c = self.reg.h; 4 }, // LD C, H
            0x4d => { self.reg.c = self.reg.l; 4 }, // LD C, L
            0x4e => { self.reg.c = mmu.readb(self.reg.hl()); 8 }, // LD C, (HL)
            0x4f => { self.reg.c = self.reg.a; 4 }, // LD C, A
            0x50 => { self.reg.d = self.reg.b; 4 }, // LD D, B
            0x51 => { self.reg.d = self.reg.c; 4 }, // LD D, C
            0x52 => { 4 }, // LD D, D
            0x53 => { self.reg.d = self.reg.e; 4 }, // LD D, E
            0x54 => { self.reg.d = self.reg.h; 4 }, // LD D, H
            0x55 => { self.reg.d = self.reg.l; 4 }, // LD D, L
            0x56 => { self.reg.d = mmu.readb(self.reg.hl()); 8 }, // LD D, (HL)
            0x57 => { self.reg.d = self.reg.a; 4 }, // LD D, A
            0x58 => { self.reg.e = self.reg.b; 4 }, // LD E, B
            0x59 => { self.reg.e = self.reg.c; 4 }, // LD E, C
            0x5a => { self.reg.e = self.reg.d; 4 }, // LD E, D
            0x5b => { 4 }, // LD E, E
            0x5c => { self.reg.e = self.reg.h; 4 }, // LD E, H
            0x5d => { self.reg.e = self.reg.l; 4 }, // LD E, L
            0x5e => { self.reg.e = mmu.readb(self.reg.hl()); 8 }, // LD E, (HL)
            0x5f => { self.reg.e = self.reg.a; 4 }, // LD E, A
            0x60 => { self.reg.h = self.reg.b; 4 }, // LD H, B
            0x61 => { self.reg.h = self.reg.c; 4 }, // LD H, C
            0x62 => { self.reg.h = self.reg.d; 4 }, // LD H, D
            0x63 => { self.reg.h = self.reg.e; 4 }, // LD H, E
            0x64 => { 4 }, // LD H, H
            0x65 => { self.reg.h = self.reg.l; 4 }, // LD H, L
            0x66 => { self.reg.h = mmu.readb(self.reg.hl()); 8 }, // LD H, (HL)
            0x67 => { self.reg.h = self.reg.a; 4 }, // LD H, A
            0x68 => { self.reg.l = self.reg.b; 4 }, // LD L, B
            0x69 => { self.reg.l = self.reg.c; 4 }, // LD L, C
            0x6a => { self.reg.l = self.reg.d; 4 }, // LD L, D
            0x6b => { self.reg.l = self.reg.e; 4 }, // LD L, E
            0x6c => { self.reg.l = self.reg.h; 4 }, // LD L, H
            0x6d => { 4 }, // LD L, L
            0x6e => { self.reg.l = mmu.readb(self.reg.hl()); 8 }, // LD L, (HL)
            0x6f => { self.reg.l = self.reg.a; 4 }, // LD L, A
            0x70 => { mmu.writeb(self.reg.hl(), self.reg.b); 8 }, // LD (HL), B
            0x71 => { mmu.writeb(self.reg.hl(), self.reg.c); 8 }, // LD (HL), C
            0x72 => { mmu.writeb(self.reg.hl(), self.reg.d); 8 }, // LD (HL), D
            0x73 => { mmu.writeb(self.reg.hl(), self.reg.e); 8 }, // LD (HL), E
            0x74 => { mmu.writeb(self.reg.hl(), self.reg.h); 8 }, // LD (HL), H
            0x75 => { mmu.writeb(self.reg.hl(), self.reg.l); 8 }, // LD (HL), L
            0x76 => { 4 }, // HALT
            0x77 => { mmu.writeb(self.reg.hl(), self.reg.a); 8 },
            0x78 => { self.reg.a = self.reg.b; 4 }, // LD A, B
            0x79 => { self.reg.a = self.reg.c; 4 }, // LD A, C
            0x7a => { self.reg.a = self.reg.d; 4 }, // LD A, D
            0x7b => { self.reg.a = self.reg.e; 4 }, // LD A, E
            0x7c => { self.reg.a = self.reg.h; 4 }, // LD A, H
            0x7d => { self.reg.a = self.reg.l; 4 }, // LD A, L
            0x7e => { self.reg.a = mmu.readb(self.reg.hl()); 8 }, // LD A, (HL)
            0x7f => { 8 }, // LD A, A

            0x80 | 0x88 => { self.add(self.reg.b, opcode == 0x88); 4 }, // ADD A, B or ADC A, B
            0x81 | 0x89 => { self.add(self.reg.c, opcode == 0x89); 4 }, // ADD A, C or ADC A, C
            0x82 | 0x8a => { self.add(self.reg.d, opcode == 0x8a); 4 }, // ADD A, D or ADC A, D
            0x83 | 0x8b => { self.add(self.reg.e, opcode == 0x8b); 4 }, // ADD A, E or ADC A, E
            0x84 | 0x8c => { self.add(self.reg.h, opcode == 0x8c); 4 }, // ADD A, H or ADC A, H
            0x85 | 0x8d => { self.add(self.reg.l, opcode == 0x8d); 4 }, // ADD A, L or ADC A, L
            0x86 | 0x8e => { let val = mmu.readb(self.reg.hl()); self.add(val, opcode == 0x8e); 8 }, // ADD A, (HL) or ADC A, (HL)
            0x87 | 0x8f => { self.add(self.reg.a, opcode == 0x8f); 4 }, // ADD A, A or SBC A, A

            0x90 | 0x98 => { self.sub(self.reg.b, opcode == 0x98); 4 }, // SUB A, B or SBC A, B
            0x91 | 0x99 => { self.sub(self.reg.c, opcode == 0x99); 4 }, // SUB A, C or SBC A, C
            0x92 | 0x9a => { self.sub(self.reg.d, opcode == 0x9a); 4 }, // SUB A, D or SBC A, D
            0x93 | 0x9b => { self.sub(self.reg.e, opcode == 0x9b); 4 }, // SUB A, E or SBC A, E
            0x94 | 0x9c => { self.sub(self.reg.h, opcode == 0x9c); 4 }, // SUB A, H or SBC A, H
            0x95 | 0x9d => { self.sub(self.reg.l, opcode == 0x9d); 4 }, // SUB A, L or SBC A, L
            0x96 | 0x9e => { let val = mmu.readb(self.reg.hl()); self.sub(val, opcode == 0x9e); 8 }, // SUB A, (HL) or SBC A, (HL)
            0x97 | 0x9f => { self.sub(self.reg.a, opcode == 0x9f); 4 }, // SUB A, A or SBC A, A

            0xa0 => { self.and(self.reg.b); 4 }, // AND A, B
            0xa1 => { self.and(self.reg.c); 4 }, // AND A, C
            0xa2 => { self.and(self.reg.d); 4 }, // AND A, D
            0xa3 => { self.and(self.reg.e); 4 }, // AND A, E
            0xa4 => { self.and(self.reg.h); 4 }, // AND A, H
            0xa5 => { self.and(self.reg.l); 4 }, // AND A, L
            0xa6 => { let val = mmu.readb(self.reg.hl()); self.and(val); 8 }, // AND A, (HL)
            0xa7 => { 4 }, // AND A, A

            0xa8 => { self.xor(self.reg.b); 4 }, // XOR A, B
            0xa9 => { self.xor(self.reg.c); 4 }, // XOR A, C
            0xaa => { self.xor(self.reg.d); 4 }, // XOR A, D
            0xab => { self.xor(self.reg.e); 4 }, // XOR A, E
            0xac => { self.xor(self.reg.h); 4 }, // XOR A, H
            0xad => { self.xor(self.reg.l); 4 }, // XOR A, L
            0xae => { let val = mmu.readb(self.reg.hl()); self.xor(val); 8 }, // XOR A, (HL)
            0xaf => { self.xor(self.reg.a); 4 }, // XOR A, A

            0xb0 => { self.or(self.reg.b); 4 }, // OR A, B
            0xb1 => { self.or(self.reg.c); 4 }, // OR A, C
            0xb2 => { self.or(self.reg.d); 4 }, // OR A, D
            0xb3 => { self.or(self.reg.e); 4 }, // OR A, E
            0xb4 => { self.or(self.reg.h); 4 }, // OR A, H
            0xb5 => { self.or(self.reg.l); 4 }, // OR A, L
            0xb6 => { let val = mmu.readb(self.reg.hl()); self.or(val); 8 }, // OR A, (HL)
            0xb7 => { 4 }, // OR A, A

            0xb8 => { self.cp(self.reg.b); 4 }, // CP A, B
            0xb9 => { self.cp(self.reg.c); 4 }, // CP A, C
            0xba => { self.cp(self.reg.d); 4 }, // CP A, D
            0xbb => { self.cp(self.reg.e); 4 }, // CP A, E
            0xbc => { self.cp(self.reg.h); 4 }, // CP A, H
            0xbd => { self.cp(self.reg.l); 4 }, // CP A, L
            0xbe => { let val = mmu.readb(self.reg.hl()); self.cp(val); 8 }, // CP A, (HL)
            0xbf => { self.cp(self.reg.a); 4 }, // CP A, A

            0xc1 => { let bc = self.pop(mmu); self.reg.set_bc(bc); 12 }, // POP BC
            0xc2 => { if self.reg.get_z() { 12 } else { self.reg.pc = self.readw(mmu); 16 } }, // JP NZ, u16
            0xc3 => { self.reg.pc = self.readw(mmu); 16 }, // JP u16
            0xc5 => { self.push(mmu, self.reg.bc()); 16 }, // PUSH BC
            0xc6 | 0xce => { let val = self.readb(mmu); self.add(val, opcode == 0xce); 8 }, // ADD A, u8 or ADC A, u8
            0xc9 => { self.reg.pc = self.pop(mmu); 16 }, // RET
            0xca => { if self.reg.get_z() { self.reg.pc = self.readw(mmu); 16 } else { 12 } }, // JP Z, u16
            0xcd => { let addr = self.readw(mmu); self.call(mmu, addr); 24 }, // CALL u16

            0xd1 => { let de = self.pop(mmu); self.reg.set_de(de); 12 }, // POP DE

            0xd2 => { if self.reg.get_c() { 12 } else { self.reg.pc = self.readw(mmu); 16 } }, // JP NC, u16
            0xd5 => { self.push(mmu, self.reg.de()); 16 }, // PUSH DE
            0xd6 | 0xde => { let val = self.readb(mmu); self.sub(val, opcode == 0xde); 8 }, // SUB A, u8 or SBC A, u8
            0xda => { if self.reg.get_c() { self.reg.pc = self.readw(mmu); 16 } else { 12 } }, // JP C, u16

            0xe1 => { let hl = self.pop(mmu); self.reg.set_hl(hl); 12 }, // POP HL
            0xe5 => { self.push(mmu, self.reg.hl()); 16 }, // PUSH HL
            0xe6 => { let val = self.readb(mmu); self.and(val); 8 }, // AND A, u8
            0xee => { let val = self.readb(mmu); self.xor(val); 8 }, // XOR A, u8

            0xf1 => { let af = self.pop(mmu); self.reg.set_af(af); 12 }, // POP AF
            0xf3 => { self.ime = false; 4 }, // DI
            0xf5 => { self.push(mmu, self.reg.af()); 16 }, // PUSH AF
            0xf6 => { let val = self.readb(mmu); self.or(val); 8 }, // OR A, u8
            0xfb => { self.ime = true; 4 } // EI
            0xfe => { let val = self.readb(mmu); self.cp(val); 8 }, // CP A, u8
            _ => panic!("Unknown opcode {:#04x} at {:#04x}.", opcode, self.reg.pc - 1)
        }
    }

    fn add(&mut self, value: u8, add_carry: bool) {
        // Result stored in A
        // Add 1 if add_carry is true and C is set to one
        let c = if add_carry && self.reg.get_c() { 1 } else { 0 };

        let sum = self.reg.a.wrapping_add(value).wrapping_add(c);

        self.reg.set_z(sum == 0);
        self.reg.set_n(false);
        self.reg.set_h((self.reg.a & 0x0f) + (value & 0x0f) + c > 0x0f);
        self.reg.set_c((self.reg.a as u16) + (value as u16) + (c as u16) > 0xff);

        self.reg.a = sum;
    }

    fn sub(&mut self, value: u8, sub_carry: bool) {
        // Result stored in A
        // Add 1 if add_carry is true and C is set to one
        let c = if sub_carry && self.reg.get_c() { 1 } else { 0 };

        let sub = self.reg.a.wrapping_sub(value).wrapping_sub(c);
        self.reg.set_z(sub == 0);
        self.reg.set_h((self.reg.a & 0x0F) < (value & 0x0F) + c);
        self.reg.set_n(true);
        self.reg.set_c((self.reg.a as u16) < (value as u16) + (c as u16));

        self.reg.a = sub;
    }

    fn xor(&mut self, value: u8) {
        self.reg.a ^= value;

        self.reg.set_h(false);
        self.reg.set_c(false);
        self.reg.set_n(false);
        self.reg.set_z(self.reg.a == 0);
    }

    fn and(&mut self, value: u8) {
        self.reg.a &= value;

        self.reg.set_h(true);
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_c(false);
        self.reg.set_n(false);
    }

    fn or(&mut self, value: u8) {
        self.reg.a |= value;

        self.reg.set_h(false);
        self.reg.set_z(self.reg.a == 0);
        self.reg.set_c(false);
        self.reg.set_n(false);
    }

    fn cp(&mut self, value: u8)
    {
        // CP sets flag as a substraction would but does not stores the result
        let tmp = self.reg.a;
        self.sub(value, false); // Cheating is allowed ;)
        self.reg.a = tmp;
    }

    fn cpl(&mut self) {
        self.reg.a = !self.reg.a;
        self.reg.set_h(true);
        self.reg.set_n(true);
    }

    fn scf(&mut self) {
        self.reg.set_n(false);
        self.reg.set_h(false);
        self.reg.set_c(true);
    }

    fn jr(&mut self, mmu: &mut Mmu) {
        // Add i8 to PC
        let delta = self.readb(mmu) as i8;
        self.reg.pc = self.reg.pc.wrapping_add(delta as u16);
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

    fn inc_at(&mut self, mmu: &mut Mmu, address: u16) {
        // INC (address)
        // Sets Z, N, and H
        let value = mmu.readb(address);
        let res = value.wrapping_add(1);
        self.reg.set_z(res == 0);
        self.reg.set_n(false);
        self.reg.set_h((value & 0x0f) + 1 > 0x0f);
        mmu.writeb(address, res);
    }

    fn dec_at(&mut self, mmu: &mut Mmu, address: u16) {
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
        self.reg.set_hl(hl.wrapping_add(1));
    }

    fn dec_hl(&mut self) {
        let hl = self.reg.hl();
        self.reg.set_hl(hl.wrapping_sub(1));
    }

    fn push(&mut self, mmu: &mut Mmu, value: u16) {
        let (msb, lsb) = to_u8(value);

        // TODO: can we simplify with writew ?
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        mmu.writeb(self.reg.sp, msb as u8);
        self.reg.sp = self.reg.sp.wrapping_sub(1);
        mmu.writeb(self.reg.sp, lsb as u8);
    }

    fn pop(&mut self, mmu: &Mmu) -> u16 {
        // Returns poped 16bits value as (msb, lsb)
        let lsb = mmu.readb(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);
        let msb = mmu.readb(self.reg.sp);
        self.reg.sp = self.reg.sp.wrapping_add(1);

        to_u16(msb, lsb)
    }

    fn call(&mut self, mmu: &mut Mmu, addr: u16) {
        // CALL on addr
        // PUSH PC and JP

        self.push(mmu, self.reg.pc);
        self.reg.pc = addr;
    }

    fn readb(&mut self, mmu: &Mmu) -> u8 {
        let byte = mmu.readb(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(1);
        byte
    }

    fn readw(&mut self, mmu: &Mmu) -> u16 {
        let word = mmu.readw(self.reg.pc);
        self.reg.pc = self.reg.pc.wrapping_add(2);
        word
    }
}