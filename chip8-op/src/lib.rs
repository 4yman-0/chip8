#![no_std]
//#![warn(clippy::alloc_instead_of_core)]
//extern crate alloc;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum OpIntoError {
    NumberTooLarge(u16),
    RegisterTooLarge(u8),
    ArgumentTooLarge(u16),
}

impl core::fmt::Display for OpIntoError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NumberTooLarge(op) => write!(f, "Number too large: {op:04X}"),
            Self::RegisterTooLarge(op) => write!(f, "Register too large: {op:02X}"),
            Self::ArgumentTooLarge(op) => write!(f, "Argument too large: {op:04X}"),
        }
    }
}

impl core::error::Error for OpIntoError {}

pub type OpIntoResult<T> = Result<T, OpIntoError>;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum OpFromError {
    UnknownOp(u16),
}

impl core::fmt::Display for OpFromError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnknownOp(op) => write!(f, "Unknown op: {op:04X}"),
        }
    }
}

impl core::error::Error for OpFromError {}

pub type OpFromResult<T> = Result<T, OpFromError>;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Cls,
    Ret,
    Scd(u8),
    Scl,
    Scr,
    Exit,
    Low,
    High,
    Sys(u16),
    Jp(u16),
    Call(u16),
    Sel(u8, u8),
    Snel(u8, u8),
    Se(u8, u8),
    Ldl(u8, u8),
    Addl(u8, u8),
    Ld(u8, u8),
    Or(u8, u8),
    And(u8, u8),
    Xor(u8, u8),
    Add(u8, u8),
    Sub(u8, u8),
    Subn(u8, u8),
    Shr(u8, u8),
    Shl(u8, u8),
    Sne(u8, u8),
    Ldi(u16),
    Ldil(u16),
    Jpr(u16),
    Rnd(u8, u8),
    Drw(u8, u8, u8),
    Skp(u8),
    Sknp(u8),
    Lddt(u8),
    Key(u8),
    Delay(u8),
    Sound(u8),
    Addi(u8),
    Hex(u8),
    Bcd(u8),
    Stor(u8),
    Rstr(u8),
    Hexx(u8),
    Storx(u8),
    Rstrx(u8),
}

impl Op {
    fn inst_xy(i: u16, x: u8, y: u8) -> OpIntoResult<u16> {
        if x > 0xF {
            return Err(OpIntoError::RegisterTooLarge(x));
        }
        if y > 0xF {
            return Err(OpIntoError::RegisterTooLarge(y));
        }
        Ok(i | ((x as u16) << 8) | (y as u16) << 4)
    }

    fn inst_xkk(i: u16, x: u8, kk: u8) -> OpIntoResult<u16> {
        if x > 0xF {
            return Err(OpIntoError::RegisterTooLarge(x));
        }
        Ok(i | ((x as u16) << 8) | (kk as u16))
    }

    fn inst_x(i: u16, x: u8) -> OpIntoResult<u16> {
        if x > 0xF {
            return Err(OpIntoError::RegisterTooLarge(x));
        }
        Ok(i | ((x as u16) << 8))
    }

    fn inst_nnn(i: u16, nnn: u16) -> OpIntoResult<u16> {
        if nnn > 0xFFF {
            return Err(OpIntoError::ArgumentTooLarge(nnn));
        }
        Ok(i | nnn)
    }

    fn inst_n(i: u16, n: u8) -> OpIntoResult<u16> {
        if n > 0xF {
            return Err(OpIntoError::ArgumentTooLarge(n as u16));
        }
        Ok(i | n as u16)
    }

    pub fn into_u16_ne(&self) -> OpIntoResult<u16> {
        Ok(match *self {
            Self::Cls => 0x00E0,
            Self::Ret => 0x00EE,
            Self::Scd(a) => Self::inst_n(0x00C0, a)?,
            Self::Scl => 0x00FB,
            Self::Scr => 0x00FC,
            Self::Exit => 0x00FD,
            Self::Low => 0x00FE,
            Self::High => 0x00FF,
            Self::Sys(addr) => Self::inst_nnn(0x0000, addr)?,
            Self::Jp(addr) => Self::inst_nnn(0x1000, addr)?,
            Self::Call(addr) => Self::inst_nnn(0x2000, addr)?,
            Self::Sel(a1, a2) => Self::inst_xkk(0x3000, a1, a2)?,
            Self::Snel(a1, a2) => Self::inst_xkk(0x4000, a1, a2)?,
            Self::Se(a1, a2) => Self::inst_xy(0x5000, a1, a2)?,
            Self::Ldl(a1, a2) => Self::inst_xkk(0x6000, a1, a2)?,
            Self::Addl(a1, a2) => Self::inst_xkk(0x7000, a1, a2)?,
            Self::Ld(a1, a2) => Self::inst_xy(0x8000, a1, a2)?,
            Self::Or(a1, a2) => Self::inst_xy(0x8001, a1, a2)?,
            Self::And(a1, a2) => Self::inst_xy(0x8002, a1, a2)?,
            Self::Xor(a1, a2) => Self::inst_xy(0x8003, a1, a2)?,
            Self::Add(a1, a2) => Self::inst_xy(0x8004, a1, a2)?,
            Self::Sub(a1, a2) => Self::inst_xy(0x8005, a1, a2)?,
            Self::Subn(a1, a2) => Self::inst_xy(0x8006, a1, a2)?,
            Self::Shr(a1, a2) => Self::inst_xy(0x8007, a1, a2)?,
            Self::Shl(a1, a2) => Self::inst_xy(0x800E, a1, a2)?,
            Self::Sne(a1, a2) => Self::inst_xy(0x9000, a1, a2)?,
            Self::Ldi(addr) => Self::inst_nnn(0xa000, addr)?,
            Self::Ldil(a) => Self::inst_nnn(0xa000, a)?,
            Self::Jpr(a) => Self::inst_nnn(0xb000, a)?,
            Self::Rnd(a1, a2) => Self::inst_xkk(0xc000, a1, a2)?,
            Self::Drw(a1, a2, a3) => {
                0xd000 | (u16::from(a1) << 8) | (u16::from(a2) << 4) | u16::from(a3)
            }
            Self::Skp(a) => Self::inst_x(0xe09e, a)?,
            Self::Sknp(a) => Self::inst_x(0xe0a1, a)?,
            Self::Lddt(a) => Self::inst_x(0xf007, a)?,
            Self::Key(a) => Self::inst_x(0xf00a, a)?,
            Self::Delay(a) => Self::inst_x(0xf015, a)?,
            Self::Sound(a) => Self::inst_x(0xf018, a)?,
            Self::Addi(a) => Self::inst_x(0xf01e, a)?,
            Self::Hex(a) => Self::inst_x(0xf029, a)?,
            Self::Bcd(a) => Self::inst_x(0xf033, a)?,
            Self::Stor(a) => Self::inst_x(0xf055, a)?,
            Self::Rstr(a) => Self::inst_x(0xf065, a)?,
            Self::Hexx(a) => Self::inst_x(0xf030, a)?,
            Self::Storx(a) => Self::inst_x(0xf075, a)?,
            Self::Rstrx(a) => Self::inst_x(0xf085, a)?,
        }
        .to_be())
    }

    /// Translates a raw u16 integer into a CHIP8 instruction
    /// # Errors
    /// Returns error if the instruction is unknown
    pub fn from_u16_ne(op: u16) -> OpFromResult<Op> {
        // CHIP8 is big endian
        let op = op.to_be();

        let o = (op & 0xF000) >> 12;
        let x = ((op & 0x0F00) >> 8) as u8;
        let y = ((op & 0x00F0) >> 4) as u8;
        let n = (op & 0x000F) as u8;
        let kk = (op & 0x00FF) as u8;
        let nnn = op & 0x0FFF;

        Ok(match o {
            0x0 => match op {
                0x00E0 => Op::Cls,
                0x00EE => Op::Ret,
                0x00FB => Op::Scr,
                0x00FC => Op::Scl,
                0x00FD => Op::Exit,
                0x00FE => Op::Low,
                0x00FF => Op::High,
                _ => {
                    if op & 0x00F0 == 0x00C0 {
                        Op::Scd(n)
                    } else {
                        Op::Sys(nnn)
                    }
                }
            },
            0x1 => Op::Jp(nnn),
            0x2 => Op::Call(nnn),
            0x3 => Op::Sel(x, kk),
            0x4 => Op::Snel(x, kk),
            0x5 => Op::Se(x, y),
            0x6 => Op::Ldl(x, kk),
            0x7 => Op::Addl(x, kk),
            0x8 => match n {
                0x0 => Op::Ld(x, y),
                0x1 => Op::Or(x, y),
                0x2 => Op::And(x, y),
                0x3 => Op::Xor(x, y),
                0x4 => Op::Add(x, y),
                0x5 => Op::Sub(x, y),
                0x6 => Op::Subn(x, y),
                0x7 => Op::Shr(x, y),
                0xE => Op::Shl(x, y),
                _ => return Err(OpFromError::UnknownOp(op)),
            },
            0x9 => Op::Sne(x, y),
            0xa => {
                if nnn < 0x200 {
                    Op::Ldi(nnn)
                } else {
                    Op::Ldil(nnn)
                }
            }
            0xb => Op::Jpr(nnn),
            0xc => Op::Rnd(x, kk),
            0xd => Op::Drw(x, y, n),
            0xe => match kk {
                0x9e => Op::Skp(x),
                0xa1 => Op::Sknp(x),
                _ => return Err(OpFromError::UnknownOp(op)),
            },
            0xf => match kk {
                0x07 => Op::Lddt(x),
                0x0a => Op::Key(x),
                0x15 => Op::Delay(x),
                0x18 => Op::Sound(x),
                0x1e => Op::Addi(x),
                0x29 => Op::Hex(x),
                0x33 => Op::Bcd(x),
                0x55 => Op::Stor(x),
                0x65 => Op::Rstr(x),
                0x30 => Op::Hexx(x),
                0x75 => Op::Storx(x),
                0x85 => Op::Rstrx(x),
                _ => return Err(OpFromError::UnknownOp(op)),
            },
            _ => return Err(OpFromError::UnknownOp(op)),
        })
    }
}

impl core::fmt::Display for Op {
    fn fmt(&self, w: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Cls => write!(w, "cls"),
            Self::Ret => write!(w, "ret"),
            Self::Scd(a) => write!(w, "scd	#{a:01X}"),
            Self::Scl => write!(w, "scl"),
            Self::Scr => write!(w, "scr"),
            Self::Exit => write!(w, "exit"),
            Self::Low => write!(w, "low"),
            Self::High => write!(w, "high"),
            Self::Sys(a) => write!(w, "sys	&{a:03X}"),
            Self::Jp(a) => write!(w, "jp	&{a:03X}"),
            Self::Call(a) => write!(w, "call	&{a:03X}"),
            Self::Sel(a1, a2) => write!(w, "se	${a1:01X}, #{a2:02X}"),
            Self::Snel(a1, a2) => write!(w, "sne	${a1:01X}, #{a2:02X}"),
            Self::Se(a1, a2) => write!(w, "se	${a1:01X}, ${a2:01X}"),
            Self::Ldl(a1, a2) => write!(w, "ld	${a1:01X}, #{a2:02X}"),
            Self::Addl(a1, a2) => write!(w, "addl	${a1:01X}, #{a2:02X}"),
            Self::Ld(a1, a2) => write!(w, "ld	${a1:01X}, ${a2:01X}"),
            Self::Or(a1, a2) => write!(w, "or	${a1:01X}, ${a2:01X}"),
            Self::And(a1, a2) => write!(w, "and	${a1:01X}, ${a2:01X}"),
            Self::Xor(a1, a2) => write!(w, "xor	${a1:01X}, ${a2:01X}"),
            Self::Add(a1, a2) => write!(w, "add	${a1:01X}, ${a2:01X}"),
            Self::Sub(a1, a2) => write!(w, "sub	${a1:01X}, ${a2:01X}"),
            Self::Subn(a1, a2) => write!(w, "subn	${a1:01X}, ${a2:01X}"),
            Self::Shr(a1, a2) => write!(w, "shr	${a1:01X}, ${a2:01X}"),
            Self::Shl(a1, a2) => write!(w, "shl	${a1:01X}, ${a2:01X}"),
            Self::Sne(a1, a2) => write!(w, "sne	${a1:01X}, ${a2:01X}"),
            Self::Ldi(a) => write!(w, "ldi	&{a:03X}"),
            Self::Ldil(a) => write!(w, "ldi	#{a:03X}"),
            Self::Jpr(a) => write!(w, "jp	V0, &{a:03X}"),
            Self::Rnd(a1, a2) => write!(w, "rnd	${a1:01X}, #{a2:02X}"),
            Self::Drw(a1, a2, a3) => write!(w, "drw	${a1:01X}, ${a2:01X}, #{a3:01X}"),
            Self::Skp(a) => write!(w, "skp	${a:01X}"),
            Self::Sknp(a) => write!(w, "sknp	${a:01X}"),
            Self::Lddt(a) => write!(w, "lddt	${a:01X}"),
            Self::Key(a) => write!(w, "key	${a:01X}"),
            Self::Delay(a) => write!(w, "delay	${a:01X}"),
            Self::Sound(a) => write!(w, "sound	${a:01X}"),
            Self::Addi(a) => write!(w, "addi	${a:01X}"),
            Self::Hex(a) => write!(w, "hex	${a:01X}"),
            Self::Bcd(a) => write!(w, "bcd	${a:01X}"),
            Self::Stor(a) => write!(w, "stor	${a:01X}"),
            Self::Rstr(a) => write!(w, "rstr	${a:01X}"),
            Self::Hexx(a) => write!(w, "hexx	${a:01X}"),
            Self::Storx(a) => write!(w, "storx	${a:01X}"),
            Self::Rstrx(a) => write!(w, "rstrx	${a:01X}"),
        }
    }
}

impl core::convert::TryFrom<u16> for Op {
    type Error = OpFromError;
    fn try_from(from: u16) -> OpFromResult<Self> {
        Self::from_u16_ne(from)
    }
}

impl core::convert::TryFrom<Op> for u16 {
    type Error = OpIntoError;
    fn try_from(from: Op) -> OpIntoResult<Self> {
        from.into_u16_ne()
    }
}
