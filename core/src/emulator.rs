use crate::{Result, Display};
use crate::instruction::Instruction;
use crate::stack::Stack;

pub const PROGRAM_OFFSET: usize = 0x200;
pub const FONT_OFFSET: usize = 0x50;

pub const FONT: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

pub struct Emulator<R: FnMut() -> u8> {
    pub memory: [u8; 0x1000],
    pub display: Display,
    pc: usize,
    i: usize,
    stack: Stack<16>,
    keys: u16,
    dt: u8,
    st: u8,
    v: [u8; 16],
    rng_func: R,
}

impl Emulator<fn() -> u8> {
    pub fn except_rng() -> Self {
        Self::new(|| 0)
    }
}

impl<R: FnMut() -> u8> Emulator<R> {
    pub fn new(rng_func: R) -> Self {
        Self {
            memory: [0; 0x1000],
            display: Display::new(),
            pc: PROGRAM_OFFSET,
            v: [0; 16],
            i: 0,
            stack: Stack::new(),
            dt: 0,
            st: 0,
            rng_func,
            keys: 0,
        }
    }

    pub fn load_program(&mut self, program: &[u8]) {
        for (i, val) in program.iter().enumerate() {
            self.memory[PROGRAM_OFFSET + i] = *val;
        }
    }

    pub fn reset(&mut self) {
        self.pc = PROGRAM_OFFSET;
        self.i = 0;
        self.dt = 0;
        self.st = 0;
        self.memory.fill(0);
        self.v.fill(0);
        self.display.clear();
        self.stack.reset();

        self.load_font();
    }

    pub fn time_step(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            self.st -= 1;
        }
    }

    pub fn cycle(&mut self, times: u32) -> Result<()> {
        for _ in 0..times {
            let opcode = self.fetch();
            self.pc += 2;

            let instruction = Instruction::try_from(opcode)?;
            self.execute(instruction)?;
        }

        Ok(())
    }

    pub fn keydown(&mut self, key: u8) {
        self.keys |= 1 << key;
    }

    pub fn keyup(&mut self, key: u8) {
        self.keys &= !(1 << key);
    }

    pub fn sound_timer(&self) -> u8 {
        self.st
    }

    fn fetch(&self) -> u16 {
        let hi = self.memory[self.pc] as u16;
        let lo = self.memory[self.pc + 1] as u16;
        (hi << 8) | lo
    }

    fn execute(&mut self, instruction: Instruction) -> Result<()> {
        match instruction {
            Instruction::SYS(_) => {}
            Instruction::CLS => self.display.clear(),
            Instruction::RET => self.pc = self.stack.pop()?,
            Instruction::JP(addr) => self.pc = addr,
            Instruction::CALL(addr) => {
                self.stack.push(self.pc)?;
                self.pc = addr;
            }
            Instruction::SEVxKK(x, kk) => {
                if self.v[x] == kk {
                    self.pc += 2;
                }
            }
            Instruction::SNEVxKK(x, kk) => {
                if self.v[x] != kk {
                    self.pc += 2;
                }
            }
            Instruction::SEVxVy(x, y) => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            Instruction::LDVxKK(x, kk) => self.v[x] = kk,
            Instruction::ADDVxKK(x, kk) => self.v[x] = self.v[x].wrapping_add(kk),
            Instruction::LDVxVy(x, y) => self.v[x] = self.v[y],
            Instruction::OR(x, y) => self.v[x] |= self.v[y],
            Instruction::AND(x, y) => self.v[x] &= self.v[y],
            Instruction::XOR(x, y) => self.v[x] ^= self.v[y],
            Instruction::ADD(x, y) => {
                let (sum, overflow) = self.v[x].overflowing_add(self.v[y]);
                self.v[0xF] = overflow.into();
                self.v[x] = sum;
            }
            Instruction::SUB(x, y) => {
                self.v[0xF] = (self.v[x] >= self.v[y]).into();
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
            }
            Instruction::SHR(x, _) => {
                self.v[0xF] = self.v[x] & 1;
                self.v[x] >>= 1;
            }
            Instruction::SUBN(x, y) => {
                self.v[0xF] = (self.v[y] >= self.v[x]).into();
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            }
            Instruction::SHL(x, _) => {
                self.v[0xF] = (self.v[x] & 0b1000_0000) >> 7;
                self.v[x] <<= 1;
            }
            Instruction::SNE(x, y) => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            Instruction::LDI(addr) => self.i = addr,
            Instruction::JPV0(addr) => self.pc = addr + self.v[0] as usize,
            Instruction::RND(x, kk) => self.v[x] = (self.rng_func)() & kk,
            Instruction::DRW(x, y, len) => {
                let pos_x = self.v[x] % 64;
                let pos_y = self.v[y] % 32;

                let bytes = &self.memory[self.i..(self.i + len as usize)];

                self.v[0xF] = 0;

                for (i, byte) in bytes.iter().enumerate() {
                    let y = pos_y + i as u8;

                    for bit in 0..8 {
                        let x = pos_x + bit;
                        let mask: u8 = 0b1000_0000 >> bit;

                        if byte & mask != 0 {
                            let result = self.display.toggle(x, y);
                            self.v[0xF] |= !result as u8;
                        }
                    }
                }
            }
            Instruction::SKP(x) => {
                if self.keys & (1 << self.v[x]) != 0 {
                    self.pc += 2;
                }
            }
            Instruction::SKNP(x) => {
                if self.keys & (1 << self.v[x]) == 0 {
                    self.pc += 2;
                }
            }
            Instruction::LDVxDT(x) => self.v[x] = self.dt,
            Instruction::LDVxK(x) => {
                self.pc -= 2;

                for key in 0..16 {
                    let mask = 1 << key;
                    if self.keys & mask != 0 {
                        self.keyup(key);
                        self.v[x] = key;
                        self.pc += 2;
                        break;
                    }
                }
            }
            Instruction::LDDTVx(x) => self.dt = self.v[x],
            Instruction::LDSTVx(x) => self.st = self.v[x],
            Instruction::ADDIVx(x) => self.i += self.v[x] as usize,
            Instruction::LDFVx(x) => {
                let char = (self.v[x] & 0xF) as usize;
                self.i = FONT_OFFSET + char * 5;
            }
            Instruction::LDBVx(x) => {
                self.memory[self.i + 0] = self.v[x] / 100;
                self.memory[self.i + 1] = (self.v[x] / 10) % 10;
                self.memory[self.i + 2] = self.v[x] % 10;
            }
            Instruction::LDIVx(x) => {
                for i in 0..=x {
                    self.memory[self.i + i] = self.v[i];
                }
            }
            Instruction::LDVxI(x) => {
                for i in 0..=x {
                    self.v[i] = self.memory[self.i + i];
                }
            }
        };

        Ok(())
    }

    fn load_font(&mut self) {
        for (i, val) in FONT.iter().enumerate() {
            self.memory[FONT_OFFSET + i] = *val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cls() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.display.toggle(0, 0);
        emulator.display.toggle(6, 1);

        emulator.execute(Instruction::CLS)?;
        assert!(emulator.display.pixel_rows().iter().all(|r| *r == 0));
        Ok(())
    }

    #[test]
    fn test_ret() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.stack.push(0x230)?;
        emulator.execute(Instruction::RET)?;

        assert_eq!(emulator.pc, 0x230);
        Ok(())
    }

    #[test]
    fn test_jp() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.execute(Instruction::JP(0x320))?;

        assert_eq!(emulator.pc, 0x320);
        Ok(())
    }

    #[test]
    fn test_call() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.pc = 0x240;
        emulator.execute(Instruction::CALL(0x350))?;

        assert_eq!(emulator.pc, 0x350);
        assert_eq!(emulator.stack.pop()?, 0x240);
        Ok(())
    }

    #[test]
    fn test_sevxkk_skip() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.pc = 0;
        emulator.v[0x6] = 0xA5;
        emulator.execute(Instruction::SEVxKK(0x6, 0xA5))?;

        assert_eq!(emulator.pc, 2);
        Ok(())
    }

    #[test]
    fn test_sevxkk_no_skip() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.pc = 0;
        emulator.v[0x6] = 0xA5;
        emulator.execute(Instruction::SEVxKK(0x6, 0x02))?;

        assert_eq!(emulator.pc, 0);
        Ok(())
    }

    #[test]
    fn test_snevxkk_skip() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.pc = 0;
        emulator.v[0x6] = 0xF3;
        emulator.execute(Instruction::SNEVxKK(0x6, 0x09))?;

        assert_eq!(emulator.pc, 2);
        Ok(())
    }

    #[test]
    fn test_snevxkk_no_skip() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.pc = 0;
        emulator.v[0x6] = 0xC9;
        emulator.execute(Instruction::SNEVxKK(0x6, 0xC9))?;

        assert_eq!(emulator.pc, 0);
        Ok(())
    }

    #[test]
    fn test_sevxvy_skip() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.pc = 0;
        emulator.v[0xB] = 0x56;
        emulator.v[0x5] = 0x56;
        emulator.execute(Instruction::SEVxVy(0xB, 0x5))?;

        assert_eq!(emulator.pc, 2);
        Ok(())
    }

    #[test]
    fn test_sevxvy_no_skip() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.pc = 0;
        emulator.v[0x3] = 0x26;
        emulator.v[0x9] = 0xBF;
        emulator.execute(Instruction::SEVxVy(0x3, 0x9))?;

        assert_eq!(emulator.pc, 0);
        Ok(())
    }

    #[test]
    fn test_ldvxkk() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.execute(Instruction::LDVxKK(0x4, 0x12))?;

        assert_eq!(emulator.v[0x4], 0x12);
        Ok(())
    }

    #[test]
    fn test_addvxkk() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.v[0x2] = 0x15;
        emulator.execute(Instruction::ADDVxKK(0x2, 0x06))?;

        assert_eq!(emulator.v[0x2], 0x1B);
        Ok(())
    }

    #[test]
    fn test_ldvxvy() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.v[0xC] = 0x24;
        emulator.execute(Instruction::LDVxVy(0x0, 0xC))?;

        assert_eq!(emulator.v[0x0], 0x24);
        Ok(())
    }

    #[test]
    fn test_or() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.v[0x3] = 0b0110_1000;
        emulator.v[0x7] = 0b1000_1101;
        emulator.execute(Instruction::OR(0x3, 0x7))?;

        assert_eq!(emulator.v[0x3], 0b1110_1101);
        Ok(())
    }

    #[test]
    fn test_and() -> Result<()> {
        let mut emulator = Emulator::except_rng();
        emulator.v[0xB] = 0b0111_1010;
        emulator.v[0x1] = 0b1010_1111;
        emulator.execute(Instruction::AND(0xB, 0x1))?;

        assert_eq!(emulator.v[0xB], 0b0010_1010);
        Ok(())
    }
}
