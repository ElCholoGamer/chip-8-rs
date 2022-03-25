use crate::Error;

pub enum Instruction {
    SYS(usize),
    CLS,
    RET,
    JP(usize),
    CALL(usize),
    SEVxKK(usize, u8),
    SNEVxKK(usize, u8),
    SEVxVy(usize, usize),
    LDVxKK(usize, u8),
    ADDVxKK(usize, u8),
    LDVxVy(usize, usize),
    OR(usize, usize),
    AND(usize, usize),
    XOR(usize, usize),
    ADD(usize, usize),
    SUB(usize, usize),
    SHR(usize, usize),
    SUBN(usize, usize),
    SHL(usize, usize),
    SNE(usize, usize),
    LDI(usize),
    JPV0(usize),
    RND(usize, u8),
    DRW(usize, usize, u8),
    SKP(usize),
    SKNP(usize),
    LDVxDT(usize),
    LDVxK(usize),
    LDDTVx(usize),
    LDSTVx(usize),
    ADDIVx(usize),
    LDFVx(usize),
    LDBVx(usize),
    LDIVx(usize),
    LDVxI(usize),
}

impl TryFrom<u16> for Instruction {
    type Error = Error;

    fn try_from(opcode: u16) -> Result<Self, Self::Error> {
        let kind = ((opcode & 0xF000) >> 12) as u8;
        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;
        let kk = (opcode & 0x00FF) as u8;
        let nnn = (opcode & 0x0FFF) as usize;

        Ok(match kind {
            0x0 => match nnn {
                0x0E0 => Self::CLS,
                0x0EE => Self::RET,
                _ => Self::SYS(nnn),
            },
            0x1 => Self::JP(nnn),
            0x2 => Self::CALL(nnn),
            0x3 => Self::SEVxKK(x, kk),
            0x4 => Self::SNEVxKK(x, kk),
            0x5 => Self::SEVxVy(x, y),
            0x6 => Self::LDVxKK(x, kk),
            0x7 => Self::ADDVxKK(x, kk),
            0x8 => match n {
                0x0 => Self::LDVxVy(x, y),
                0x1 => Self::OR(x, y),
                0x2 => Self::AND(x, y),
                0x3 => Self::XOR(x, y),
                0x4 => Self::ADD(x, y),
                0x5 => Self::SUB(x, y),
                0x6 => Self::SHR(x, y),
                0x7 => Self::SUBN(x, y),
                0xE => Self::SHL(x, y),
                _ => return Err(Error::IllegalOpcode { opcode }),
            },
            0x9 => Self::SNE(x, y),
            0xA => Self::LDI(nnn),
            0xB => Self::JPV0(nnn),
            0xC => Self::RND(x, kk),
            0xD => Self::DRW(x, y, n),
            0xE => match kk {
                0x9E => Self::SKP(x),
                0xA1 => Self::SKNP(x),
                _ => return Err(Error::IllegalOpcode { opcode }),
            },
            0xF => match kk {
                0x07 => Self::LDVxDT(x),
                0x0A => Self::LDVxK(x),
                0x15 => Self::LDDTVx(x),
                0x18 => Self::LDSTVx(x),
                0x1E => Self::ADDIVx(x),
                0x29 => Self::LDFVx(x),
                0x33 => Self::LDBVx(x),
                0x55 => Self::LDIVx(x),
                0x65 => Self::LDVxI(x),
                _ => return Err(Error::IllegalOpcode { opcode }),
            }
            _ => return Err(Error::IllegalOpcode { opcode }),
        })
    }
}