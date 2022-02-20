use crate::registers::Registers;
use crate::mmu::Mmu;
use crate::utils::to_u8;
use crate::utils::to_u16;
use crate::utils::Bits;

// Address of interupts routines
const V_BLANK_ROUTINE: u16 = 0x40;
const LCD_ROUTINE: u16 = 0x48;
const TIMER_ROUTINE: u16 = 0x50;
const JOYPAD_ROUTINE: u16 = 0x60;

// Interupt bit
pub const V_BLANK_INTERUPT: u8 = 0;
pub const LCD_INTERUPT: u8 = 1;
pub const TIMER_INTERUPT: u8 = 2;
pub const JOYPAD_INTERUPT: u8 = 4;


pub struct Cpu {
    reg: Registers,
    ime: bool,
    halted: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            reg: Registers::new(),
            ime: true,
            halted: false,
        }
    }

    // TODO: better jr
    pub fn run_cycle(&mut self, mmu: &mut Mmu) -> u32 {
        // In this implementation, the Cpu will give the number of cycle
        // to run on other components

        if self.halted {
            return 4;
        }

        let opcode = self.readb(mmu);

        match opcode {
            0x00 => 4, // NOP
            0x01 => { let w = self.readw(mmu); self.reg.set_bc(w); 12 }, // LD BC, n16
            0x02 => { mmu.writeb(self.reg.bc(), self.reg.a); 8 }, // LD (BC), A
            0x03 => { let bc = self.reg.bc(); self.reg.set_bc(bc.wrapping_add(1)); 8 }, // INC BC
            0x04 => { self.reg.b = self.inc(self.reg.b); 4 }, // INC B
            0x05 => { self.reg.b = self.dec(self.reg.b); 4 }, // DEC B
            0x06 => { self.reg.b = self.readb(mmu); 8 }, // LD B, u8
            0x07 => { self.reg.a = self.rlc(self.reg.a); self.reg.set_z(false); 4 }, // RLCA
            0x08 => { let addr = self.readw(mmu); mmu.writew(addr, self.reg.sp); 20 }, // LD (u16), SP
            0x09 => { self.add16(self.reg.bc()); 8 }, // ADD HL, BC
            0x0a => { let bc = self.reg.bc(); self.reg.a = mmu.readb(bc); 8 }, // LD A, (BC)
            0x0b => { let bc = self.reg.bc(); self.reg.set_bc(bc.wrapping_sub(1)); 8 }, // DEC BC
            0x0c => { self.reg.c = self.inc(self.reg.c); 4 }, // INC C
            0x0d => { self.reg.c = self.dec(self.reg.c); 4 }, // DEC C
            0x0e => { self.reg.c = self.readb(mmu); 8 }, // LD C, u8
            0x0f => { self.reg.a = self.rrc(self.reg.a); self.reg.set_z(false); 4 }, // RRCA
            0x10 => 4, // STOP
            0x11 => { let w = self.readw(mmu); self.reg.set_de(w); 12 }, // LD DE, n16
            0x12 => { mmu.writeb(self.reg.de(), self.reg.a); 8 }, // LD (DE), A
            0x13 => { let de = self.reg.de(); self.reg.set_de(de.wrapping_add(1)); 8 }, // INC DE
            0x14 => { self.reg.d = self.inc(self.reg.d); 4 }, // INC D
            0x15 => { self.reg.d = self.dec(self.reg.d); 4 }, // DEC D
            0x16 => { self.reg.d = self.readb(mmu); 8 }, // LD D, u8
            0x17 => { self.reg.a = self.rl(self.reg.a); self.reg.set_z(false); 4 }, // RLA
            0x18 => { let delta = self.readb(mmu); self.jr(delta as i8); 12 }, // JR i8
            0x19 => { self.add16(self.reg.de()); 8 }, // ADD HL, DE
            0x1a => { let de = self.reg.de(); self.reg.a = mmu.readb(de); 8 }, // LD A, (DE)
            0x1b => { let de = self.reg.de(); self.reg.set_de(de.wrapping_sub(1)); 8 }, // DEC DE
            0x1c => { self.reg.e = self.inc(self.reg.e); 4 }, // INC E
            0x1d => { self.reg.e = self.dec(self.reg.e); 4 }, // DEC E
            0x1e => { self.reg.e = self.readb(mmu); 8 }, // LD E, u8
            0x1f => { self.reg.a = self.rr(self.reg.a); self.reg.set_z(false); 4 }, // RRA
            0x20 => { let delta = self.readb(mmu); if self.reg.get_z() { 8 } else { self.jr(delta as i8); 12 } } // JR NZ, i8
            0x21 => { let w = self.readw(mmu); self.reg.set_hl(w); 12 }, // LD HL, n16
            0x22 => { mmu.writeb(self.reg.hl(), self.reg.a); self.inc_hl(); 8 }, // LD (HL+), A
            0x23 => { self.inc_hl(); 8 }, // INC HL
            0x24 => { self.reg.h = self.inc(self.reg.h); 4 }, // INC H
            0x25 => { self.reg.h = self.dec(self.reg.h); 4 }, // DEC H
            0x26 => { self.reg.h = self.readb(mmu); 8 }, // LD H, u8
            0x27 => { self.daa(); 4 },
            0x28 => { let delta = self.readb(mmu); if self.reg.get_z() { self.jr(delta as i8); 12 } else { 8 } } // JR Z, i8
            0x29 => { self.add16(self.reg.hl()); 8 }, // ADD HL, HL
            0x2a => { let hl = self.reg.hl(); self.inc_hl(); self.reg.a = mmu.readb(hl); 8 }, // LD A, (HL+)
            0x2b => { self.dec_hl(); 8 }, // DEC HL
            0x2c => { self.reg.l = self.inc(self.reg.l); 4 }, // INC L
            0x2d => { self.reg.l = self.dec(self.reg.l); 4 }, // DEC L
            0x2e => { self.reg.l = self.readb(mmu); 8 }, // LD L, u8
            0x2f => { self.cpl(); 4 }, // CPL
            0x30 => { let delta = self.readb(mmu); if self.reg.get_c() { 8 } else { self.jr(delta as i8); 12 } } // JR NC, i8
            0x31 => { self.reg.sp = self.readw(mmu); 12 }, // LD SP, n16
            0x32 => { mmu.writeb(self.reg.hl(), self.reg.a); self.dec_hl(); 8 }, // LD (HL-), A
            0x33 => { self.reg.sp = self.reg.sp.wrapping_add(1); 8 }, // INC SP
            0x34 => { let hl = self.reg.hl(); self.inc_at(mmu, hl); 12 }, // INC (HL)
            0x35 => { let hl = self.reg.hl(); self.dec_at(mmu, hl); 12 }, // DEC (HL)
            0x36 => { let val = self.readb(mmu); mmu.writeb(self.reg.hl(), val); 8 }, // LD (HL), u8
            0x37 => { self.scf(); 4 }, // SCF
            0x38 => { let delta = self.readb(mmu); if self.reg.get_c() { self.jr(delta as i8); 12 } else { 8 } }, // JR C, i8
            0x39 => { self.add16(self.reg.sp); 8 }, // ADD HL, SP
            0x3a => { let hl = self.reg.hl(); self.dec_hl(); self.reg.a = mmu.readb(hl); 8 }, // LD A, (HL-)
            0x3b => { self.reg.sp = self.reg.sp.wrapping_sub(1); 8 }, // DEC SP
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
            0x76 => { self.halted = true; 4 }, // HALT
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
            0xa7 => { self.and(self.reg.a); 4 }, // AND A, A

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
            0xb7 => { self.or(self.reg.a); 4 }, // OR A, A

            0xb8 => { self.cp(self.reg.b); 4 }, // CP A, B
            0xb9 => { self.cp(self.reg.c); 4 }, // CP A, C
            0xba => { self.cp(self.reg.d); 4 }, // CP A, D
            0xbb => { self.cp(self.reg.e); 4 }, // CP A, E
            0xbc => { self.cp(self.reg.h); 4 }, // CP A, H
            0xbd => { self.cp(self.reg.l); 4 }, // CP A, L
            0xbe => { let val = mmu.readb(self.reg.hl()); self.cp(val); 8 }, // CP A, (HL)
            0xbf => { self.cp(self.reg.a); 4 }, // CP A, A

            0xc0 => { if self.reg.get_z() { 8 } else { self.reg.pc = self.pop(mmu); 20 }}, // RET NZ
            0xc1 => { let bc = self.pop(mmu); self.reg.set_bc(bc); 12 }, // POP BC
            0xc2 => { let addr = self.readw(mmu); if self.reg.get_z() { 12 } else { self.reg.pc = addr; 16 } }, // JP NZ, u16
            0xc3 => { self.reg.pc = self.readw(mmu); 16 }, // JP u16
            0xc4 => { let addr = self.readw(mmu); if self.reg.get_z() { 12 } else { self.call(mmu, addr); 24 }}, // CALL NZ, u16
            0xc5 => { self.push(mmu, self.reg.bc()); 16 }, // PUSH BC
            0xc6 | 0xce => { let val = self.readb(mmu); self.add(val, opcode == 0xce); 8 }, // ADD A, u8 or ADC A, u8
            0xc7 => { self.call(mmu, 0); 16 }, // RST 00
            0xc8 => { if self.reg.get_z() { self.reg.pc = self.pop(mmu); 20 } else { 8 }}, // RET Z
            0xc9 => { self.reg.pc = self.pop(mmu); 16 }, // RET
            0xca => { let addr = self.readw(mmu); if self.reg.get_z() { self.reg.pc = addr; 16 } else { 12 } }, // JP Z, u16
            0xcb => { self.run_prefixed(mmu) }, // PREFIX
            0xcc => { let addr = self.readw(mmu); if self.reg.get_z() { self.call(mmu, addr); 24 } else { 12 }}, // CALL Z, u16
            0xcd => { let addr = self.readw(mmu); self.call(mmu, addr); 24 }, // CALL u16
            0xcf => { self.call(mmu, 0x08); 16 }, // RST 08

            0xd0 => { if self.reg.get_c() { 8 } else { self.reg.pc = self.pop(mmu); 20 }}, // RET NC
            0xd1 => { let de = self.pop(mmu); self.reg.set_de(de); 12 }, // POP DE

            0xd2 => { let addr = self.readw(mmu); if self.reg.get_c() { 12 } else { self.reg.pc = addr; 16 } }, // JP NC, u16
            0xd4 => { let addr = self.readw(mmu); if self.reg.get_c() { 12 } else { self.call(mmu, addr); 24 }}, // CALL NC, u16
            0xd5 => { self.push(mmu, self.reg.de()); 16 }, // PUSH DE
            0xd6 | 0xde => { let val = self.readb(mmu); self.sub(val, opcode == 0xde); 8 }, // SUB A, u8 or SBC A, u8
            0xd7 => { self.call(mmu, 0x10); 16 }, // RST 10
            0xd8 => { if self.reg.get_c() { self.reg.pc = self.pop(mmu); 20 } else { 8 }}, // RET C
            0xd9 => { self.ime = true; self.reg.pc = self.pop(mmu); 16 },
            0xda => { let addr = self.readw(mmu); if self.reg.get_c() { self.reg.pc = addr; 16 } else { 12 } }, // JP C, u16
            0xdc => { let addr = self.readw(mmu); if self.reg.get_c() { self.call(mmu, addr); 24 } else { 12 }}, // CALL C, u16
            0xdf => { self.call(mmu, 0x18); 16 }, // RST 18

            0xe0 => { let addr = 0xff00 | self.readb(mmu) as u16; mmu.writeb(addr, self.reg.a); 12 }, // LD (FF00+u8), A
            0xe1 => { let hl = self.pop(mmu); self.reg.set_hl(hl); 12 }, // POP HL
            0xe2 => { let addr = 0xff00 | self.reg.c as u16; mmu.writeb(addr, self.reg.a); 12 }, // LD (FF00+C), A
            0xe5 => { self.push(mmu, self.reg.hl()); 16 }, // PUSH HL
            0xe6 => { let val = self.readb(mmu); self.and(val); 8 }, // AND A, u8
            0xe7 => { self.call(mmu, 0x20); 16 }, // RST 20
            0xe8 => { let delta = self.readb(mmu); self.reg.sp = self.add_sp(delta); 16 }, // ADD SP, i8
            0xe9 => { self.reg.pc = self.reg.hl(); 4 }, // JP HL
            0xea => { let addr = self.readw(mmu); mmu.writeb(addr, self.reg.a); 16 }, // LD (u16), A
            0xee => { let val = self.readb(mmu); self.xor(val); 8 }, // XOR A, u8
            0xef => { self.call(mmu, 0x28); 16 }, // RST 28

            0xf0 => { let addr = 0xff00 | self.readb(mmu) as u16; self.reg.a = mmu.readb(addr); 12 }, // LD A, (FF00+u8)
            0xf1 => { let af = self.pop(mmu); self.reg.set_af(af); 12 }, // POP AF
            0xf2 => { let addr = 0xff00 | self.reg.c as u16; self.reg.a = mmu.readb(addr); 12 }, // LD A, (FF00+C)
            0xf3 => { self.ime = false; 4 }, // DI
            0xf5 => { self.push(mmu, self.reg.af()); 16 }, // PUSH AF
            0xf6 => { let val = self.readb(mmu); self.or(val); 8 }, // OR A, u8
            0xf7 => { self.call(mmu, 0x30); 16 }, // RST 30
            0xf8 => { let delta = self.readb(mmu); let res = self.add_sp(delta); self.reg.set_hl(res); 12 }, // LD HL, SP+i8
            0xf9 => { self.reg.sp = self.reg.hl(); 8 }, // LD SP, HL
            0xfa => { let addr = self.readw(mmu); self.reg.a = mmu.readb(addr); 16 }, // LD A, (u16)
            0xfb => { self.ime = true; 4 }, // EI
            0xfe => { let val = self.readb(mmu); self.cp(val); 8 }, // CP A, u8
            0xff => { self.call(mmu, 0x38); 16 }, // RST 38
            _ => panic!("Unknown opcode {:#04x} at {:#04x}.", opcode, self.reg.pc - 1)
        }
    }

    fn run_prefixed(&mut self, mmu: &mut Mmu) -> u32 {
        let opcode = self.readb(mmu);

        match opcode {
            0x00 => { self.reg.b = self.rlc(self.reg.b); 8 }, // RLC B
            0x01 => { self.reg.c = self.rlc(self.reg.c); 8 }, // RLC C
            0x02 => { self.reg.d = self.rlc(self.reg.d); 8 }, // RLC D
            0x03 => { self.reg.e = self.rlc(self.reg.e); 8 }, // RLC E
            0x04 => { self.reg.h = self.rlc(self.reg.h); 8 }, // RLC H
            0x05 => { self.reg.l = self.rlc(self.reg.l); 8 }, // RLC L
            0x06 => { let res = self.rlc(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // RLC (HL)
            0x07 => { self.reg.a = self.rlc(self.reg.a); 8 }, // RLC A
            0x08 => { self.reg.b = self.rrc(self.reg.b); 8 }, // RRC B
            0x09 => { self.reg.c = self.rrc(self.reg.c); 8 }, // RRC C
            0x0a => { self.reg.d = self.rrc(self.reg.d); 8 }, // RRC D
            0x0b => { self.reg.e = self.rrc(self.reg.e); 8 }, // RRC E
            0x0c => { self.reg.h = self.rrc(self.reg.h); 8 }, // RRC H
            0x0d => { self.reg.l = self.rrc(self.reg.l); 8 }, // RRC L
            0x0e => { let res = self.rrc(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // RRC (HL)
            0x0f => { self.reg.a = self.rrc(self.reg.a); 8 }, // RRC A
            0x10 => { self.reg.b = self.rl(self.reg.b); 8 }, // RL B
            0x11 => { self.reg.c = self.rl(self.reg.c); 8 }, // RL C
            0x12 => { self.reg.d = self.rl(self.reg.d); 8 }, // RL D
            0x13 => { self.reg.e = self.rl(self.reg.e); 8 }, // RL E
            0x14 => { self.reg.h = self.rl(self.reg.h); 8 }, // RL H
            0x15 => { self.reg.l = self.rl(self.reg.l); 8 }, // RL L
            0x16 => { let res = self.rl(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // RL (HL)
            0x17 => { self.reg.a = self.rl(self.reg.a); 8 }, // RL A
            0x18 => { self.reg.b = self.rr(self.reg.b); 8 }, // RR B
            0x19 => { self.reg.c = self.rr(self.reg.c); 8 }, // RR C
            0x1a => { self.reg.d = self.rr(self.reg.d); 8 }, // RR D
            0x1b => { self.reg.e = self.rr(self.reg.e); 8 }, // RR E
            0x1c => { self.reg.h = self.rr(self.reg.h); 8 }, // RR H
            0x1d => { self.reg.l = self.rr(self.reg.l); 8 }, // RR L
            0x1e => { let res = self.rr(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // RR (HL)
            0x1f => { self.reg.a = self.rr(self.reg.a); 8 }, // RR A
            0x20 => { self.reg.b = self.sla(self.reg.b); 8 }, // SLA B
            0x21 => { self.reg.c = self.sla(self.reg.c); 8 }, // SLA C
            0x22 => { self.reg.d = self.sla(self.reg.d); 8 }, // SLA D
            0x23 => { self.reg.e = self.sla(self.reg.e); 8 }, // SLA E
            0x24 => { self.reg.h = self.sla(self.reg.h); 8 }, // SLA H
            0x25 => { self.reg.l = self.sla(self.reg.l); 8 }, // SLA L
            0x26 => { let res = self.sla(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // SLA (HL)
            0x27 => { self.reg.a = self.sla(self.reg.a); 8 }, // SLA B
            0x28 => { self.reg.b = self.sra(self.reg.b); 8 }, // SRA B
            0x29 => { self.reg.c = self.sra(self.reg.c); 8 }, // SRA C
            0x2a => { self.reg.d = self.sra(self.reg.d); 8 }, // SRA D
            0x2b => { self.reg.e = self.sra(self.reg.e); 8 }, // SRA E
            0x2c => { self.reg.h = self.sra(self.reg.h); 8 }, // SRA H
            0x2d => { self.reg.l = self.sra(self.reg.l); 8 }, // SRA L
            0x2e => { let res = self.sra(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // SRA (HL)
            0x2f => { self.reg.a = self.sra(self.reg.a); 8 }, // SRA B
            0x30 => { self.reg.b = self.swap(self.reg.b); 8 }, // SWAP B
            0x31 => { self.reg.c = self.swap(self.reg.c); 8 }, // SWAP C
            0x32 => { self.reg.d = self.swap(self.reg.d); 8 }, // SWAP D
            0x33 => { self.reg.e = self.swap(self.reg.e); 8 }, // SWAP E
            0x34 => { self.reg.h = self.swap(self.reg.h); 8 }, // SWAP H
            0x35 => { self.reg.l = self.swap(self.reg.l); 8 }, // SWAP L
            0x36 => { let res = self.swap(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // SWAP (HL)
            0x37 => { self.reg.a = self.swap(self.reg.a); 8 }, // SWAP B
            0x38 => { self.reg.b = self.srl(self.reg.b); 8 }, // SRL B
            0x39 => { self.reg.c = self.srl(self.reg.c); 8 }, // SRL C
            0x3a => { self.reg.d = self.srl(self.reg.d); 8 }, // SRL D
            0x3b => { self.reg.e = self.srl(self.reg.e); 8 }, // SRL E
            0x3c => { self.reg.h = self.srl(self.reg.h); 8 }, // SRL H
            0x3d => { self.reg.l = self.srl(self.reg.l); 8 }, // SRL L
            0x3e => { let res = self.srl(mmu.readb(self.reg.hl())); mmu.writeb(self.reg.hl(), res); 16 }, // SRL (HL)
            0x3f => { self.reg.a = self.srl(self.reg.a); 8 }, // SRL A
            0x40 => { self.bit(self.reg.b, 0); 8 }, // BIT 0, B
            0x40..=0x7f => {
                let value = match opcode & 0x0f {
                    0x0 | 0x8 => self.reg.b,
                    0x1 | 0x9 => self.reg.c,
                    0x2 | 0xa => self.reg.d,
                    0x3 | 0xb => self.reg.e,
                    0x4 | 0xc => self.reg.h,
                    0x5 | 0xd => self.reg.l,
                    0x6 | 0xe => mmu.readb(self.reg.hl()),
                    0x7 | 0xf => self.reg.a,
                    _ => panic!("Should not happen!")
                };

                let index = (opcode - 0x40) / 8;
                self.bit(value, index);
                match opcode & 0xf {
                    0x6 | 0xe => 12,
                    _ => 8
                }
            }, // BIT index, r8
            0x80..=0xbf => {
                let index = (opcode - 0x80) / 8;
                match opcode & 0x0f {
                    0x0 | 0x8 => { self.reg.b = self.reg.b.unset_bit(index); 8 },
                    0x1 | 0x9 => { self.reg.c = self.reg.c.unset_bit(index); 8 },
                    0x2 | 0xa => { self.reg.d = self.reg.d.unset_bit(index); 8 },
                    0x3 | 0xb => { self.reg.e = self.reg.e.unset_bit(index); 8 },
                    0x4 | 0xc => { self.reg.h = self.reg.h.unset_bit(index); 8 },
                    0x5 | 0xd => { self.reg.l = self.reg.l.unset_bit(index); 8 },
                    0x6 | 0xe => { let val = mmu.readb(self.reg.hl()); mmu.writeb(self.reg.hl(), val.unset_bit(index)); 16 },
                    0x7 | 0xf => { self.reg.a = self.reg.a.unset_bit(index); 8 },
                    _ => panic!("Should not happen!")
                }
            }, // RES index, r8
            0xc0..=0xff => {
                let index = (opcode - 0xc0) / 8;
                match opcode & 0x0f {
                    0x0 | 0x8 => { self.reg.b = self.reg.b.set_bit(index); 8 },
                    0x1 | 0x9 => { self.reg.c = self.reg.c.set_bit(index); 8 },
                    0x2 | 0xa => { self.reg.d = self.reg.d.set_bit(index); 8 },
                    0x3 | 0xb => { self.reg.e = self.reg.e.set_bit(index); 8 },
                    0x4 | 0xc => { self.reg.h = self.reg.h.set_bit(index); 8 },
                    0x5 | 0xd => { self.reg.l = self.reg.l.set_bit(index); 8 },
                    0x6 | 0xe => { let val = mmu.readb(self.reg.hl()); mmu.writeb(self.reg.hl(), val.set_bit(index)); 16 },
                    0x7 | 0xf => { self.reg.a = self.reg.a.set_bit(index); 8 },
                    _ => panic!("Should not happen!")
                }
            } // SET index, r8
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

    fn add16(&mut self, value: u16) {
        // Result stored in HL

        let sum = self.reg.hl().wrapping_add(value);

        self.reg.set_n(false);
        self.reg.set_h((self.reg.hl() & 0x0fff) + (value & 0x0fff) > 0x0fff);
        self.reg.set_c((self.reg.hl() as u32) + (value as u32) > 0xffff);

        self.reg.set_hl(sum);
    }

    fn add_sp(&mut self, value: u8) -> u16 {

        let value = ((value as i8) as i16) as u16;
        let res = self.reg.sp.wrapping_add(value);

        self.reg.set_c((self.reg.sp & 0xff) + (value & 0xff) > 0xff);
        self.reg.set_h((self.reg.sp & 0xf) + (value & 0xf) > 0xf);
        self.reg.set_z(false);
        self.reg.set_n(false);

        res
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

    fn rlc(&mut self, value: u8) -> u8 {
        // Rotate left
        let res = value.rotate_left(1);
        self.reg.set_z(res == 0);
        self.reg.set_c(value.is_set(7));
        self.reg.set_h(false);
        self.reg.set_n(false);
        res
    }

    fn rl(&mut self, value: u8) -> u8 {
        // Rotate left through carry
        let mut res = value << 1;
        if self.reg.get_c() {
            res |= 1;
        }

        self.reg.set_z(res == 0);
        self.reg.set_c(value.is_set(7));
        self.reg.set_h(false);
        self.reg.set_n(false);
        res
    }

    fn rrc(&mut self, value: u8) -> u8 {
        // Rotate right
        let res = value.rotate_right(1);
        self.reg.set_z(res == 0);
        self.reg.set_c(value.is_set(0));
        self.reg.set_h(false);
        self.reg.set_n(false);
        res
    }

    fn rr(&mut self, value: u8) -> u8 {
        // Rotate right through carry
        let mut res = value >> 1;
        if self.reg.get_c() {
            res = res.set_bit(7);
        }

        self.reg.set_z(res == 0);
        self.reg.set_c(value.is_set(0));
        self.reg.set_h(false);
        self.reg.set_n(false);
        res
    }

    fn srl(&mut self, value: u8) -> u8 {
        // Shift right logically
        self.reg.set_c(value & 1 == 1);
        self.reg.set_h(false);
        self.reg.set_n(false);
        self.reg.set_z((value >> 1) == 0);
        value >> 1
    }

    fn sra(&mut self, value: u8) -> u8 {
        // Shift right arithmetically
        let res = (value >> 1) | (value & 0b1000_0000);

        self.reg.set_c(value.is_set(0));
        self.reg.set_h(false);
        self.reg.set_n(false);
        self.reg.set_z(res == 0);
        res
    }

    fn sla(&mut self, value: u8) -> u8 {
        // Shift left
        let res = (value << 1);

        self.reg.set_c(value.is_set(7));
        self.reg.set_h(false);
        self.reg.set_n(false);
        self.reg.set_z(res == 0);
        res
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

    fn daa(&mut self) {
        if self.reg.get_n() {
            // After a subtraction, only adjust if (half-)carry occurred
            if self.reg.get_c() {
                self.reg.a = self.reg.a.wrapping_sub(0x60);
            }
            if self.reg.get_h() {
                self.reg.a = self.reg.a.wrapping_sub(0x6);
            }
        } else {
            // After an addition, adjust if (half-)carry occurred or if result is out of bounds
            if self.reg.get_c() || self.reg.a > 0x99 {
                self.reg.a = self.reg.a.wrapping_add(0x60);
                self.reg.set_c(true);
            }
            if self.reg.get_h() || (self.reg.a & 0x0f) > 0x09 {
                self.reg.a = self.reg.a.wrapping_add(0x6);
            }
        }

        self.reg.set_z(self.reg.a == 0);
        self.reg.set_h(false);
    }

    fn swap(&mut self, value: u8) -> u8 {
        let msb = value & 0xf0;
        let lsb = value & 0x0f;

        let res = (msb >> 4) | (lsb << 4);
        self.reg.set_z(res == 0);
        self.reg.set_h(false);
        self.reg.set_c(false);
        self.reg.set_n(false);
        res
    }

    fn bit(&mut self, value: u8, index: u8) {
        self.reg.set_n(false);
        self.reg.set_h(true);
        self.reg.set_z(!value.is_set(index));
    }

    fn jr(&mut self, delta: i8) {
        // Add i8 to PC
        self.reg.pc = ((self.reg.pc as u32 as i32) + (delta as i32)) as u16;
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

    pub fn check_interupts(&mut self, mmu: &mut Mmu) {
        if !self.ime && !self.halted {
            return
        }

        let requested = mmu.int_request as u16;
        let enabled = mmu.int_enabled as u16;
        let triggered = requested & enabled;
        if triggered == 0 {
            return
        }

        self.ime = false;
        self.halted = false;

        for interupt in 0..=4 {
            if requested.is_set(interupt) && enabled.is_set(interupt) {
                self.execute_interupt(interupt, mmu);
            }
        }
    }

    fn execute_interupt(&mut self, interupt: u8, mmu: &mut Mmu) {
        // Unset request
        mmu.int_request ^= 1 << interupt;

        match interupt {
            V_BLANK_INTERUPT => self.call(mmu, V_BLANK_ROUTINE),
            LCD_INTERUPT => self.call(mmu, LCD_ROUTINE),
            TIMER_INTERUPT => self.call(mmu, TIMER_ROUTINE),
            JOYPAD_INTERUPT => self.call(mmu, JOYPAD_ROUTINE),
            _ => panic!("Should not happen")
        }
    }
}