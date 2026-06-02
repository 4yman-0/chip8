use crate::lexer::{Token, TokenKind};
use alloc::{string::String, vec::Vec};
use chip8_op::Op;

pub struct Ast {
    pub insts: Vec<Inst>,
}

#[derive(Clone, Debug)]
pub enum Operand {
    Literal(u16),
    Label(String),
}

impl core::fmt::Display for Operand {
    fn fmt(&self, w: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Literal(lit) => write!(w, "0x{lit:04X}"),
            Self::Label(label) => write!(w, "\"{label}\""),
        }
    }
}

#[derive(Clone, Debug)]
pub enum InstKind {
    Op(Op),
    // Special instruction requiring extra processing
    Sys(String),
    Jp(String),
    Call(String),
    Ldi(String),
    // Special instruction requiring extra processing
    Byte(u8),
    Dbyte(Operand),
    DbyteBe(Operand),
    Label(String),
}

impl core::fmt::Display for InstKind {
    fn fmt(&self, w: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Jp(label) => write!(w, "Jp(\"{label}\")"),
            Self::Call(label) => write!(w, "Call(\"{label}\")"),
            Self::Sys(label) => write!(w, "Sys(\"{label}\")"),
            Self::Ldi(label) => write!(w, "Ldi(\"{label}\")"),
            Self::Dbyte(op) => write!(w, "Dbyte({op})"),
            Self::DbyteBe(op) => write!(w, "DbyteBe({op})"),
            Self::Label(label) => write!(w, "Label(\"{label}\")"),
            _ => write!(w, "{self:?}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Inst {
    pub kind: InstKind,
    pub line: usize,
    pub column: usize,
}

impl Inst {
    const fn new(tok: &Token, kind: InstKind) -> Self {
        Self {
            line: tok.line,
            column: tok.column,
            kind,
        }
    }
}

impl core::fmt::Display for Inst {
    fn fmt(&self, w: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(w, "{}:{}: {}", self.line + 1, self.column + 1, self.kind)
    }
}

type TokenIter<'a> = core::iter::Peekable<core::slice::Iter<'a, Token>>;

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseError {
    ExpectedRegister,
    ExpectedNumber,
    ExpectedComma,
    ExpectedAddress,
    ExpectedKeyword,
    ExpectedColon,
    ExpectedIdentifier,
    ExpectedAddressOrIdentifier,
    ExpectedNumberOrIdentifier,
    ExpectedNumberOrRegister,
    NumberTooLarge(u32),
    AddressTooLarge(u32),
    RegisterTooLarge(u32),
    UnmatchedToken(Token),
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, w: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(w, "{self:?}")
    }
}

impl core::error::Error for ParseError {}

fn parse_register(iter: &mut TokenIter) -> ParseResult<u8> {
    match iter.next_if(|t| matches!(t.kind, TokenKind::Reg(_))) {
        Some(Token {
            kind: TokenKind::Reg(r),
            ..
        }) => match u8::try_from(*r) {
            Ok(r) => Ok(r),
            Err(_) => Err(ParseError::RegisterTooLarge(*r)),
        },
        _ => Err(ParseError::ExpectedRegister),
    }
}

fn parse_number(iter: &mut TokenIter) -> ParseResult<u16> {
    match iter.next_if(|t| matches!(t.kind, TokenKind::Num(_))) {
        Some(Token {
            kind: TokenKind::Num(n),
            ..
        }) => match u16::try_from(*n) {
            Ok(n) => Ok(n),
            Err(_) => Err(ParseError::NumberTooLarge(*n)),
        },
        _ => Err(ParseError::ExpectedNumber),
    }
}

fn parse_number_u8(iter: &mut TokenIter) -> ParseResult<u8> {
    let number = parse_number(iter)?;
    if number > u16::from(u8::MAX) {
        return Err(ParseError::NumberTooLarge(number as u32));
    }
    Ok(u8::try_from(number).unwrap())
}

fn parse_address(iter: &mut TokenIter) -> ParseResult<u16> {
    match iter.next_if(|t| matches!(t.kind, TokenKind::Addr(_))) {
        Some(Token {
            kind: TokenKind::Addr(n),
            ..
        }) => match u16::try_from(*n) {
            Ok(n) => Ok(n),
            Err(_) => Err(ParseError::AddressTooLarge(*n)),
        },
        _ => Err(ParseError::ExpectedAddress),
    }
}

fn parse_identifier(iter: &mut TokenIter) -> ParseResult<String> {
    match iter.next_if(|t| matches!(t.kind, TokenKind::Ident(_))) {
        Some(Token {
            kind: TokenKind::Ident(l),
            ..
        }) => Ok(l.clone()),
        _ => Err(ParseError::ExpectedIdentifier),
    }
}

fn expect_colon(iter: &mut TokenIter) -> ParseResult<()> {
    match iter.next_if(|t| matches!(t.kind, TokenKind::Colon)) {
        Some(Token {
            kind: TokenKind::Colon,
            ..
        }) => Ok(()),
        _ => Err(ParseError::ExpectedColon),
    }
}

fn optional_comma(iter: &mut TokenIter) {
    let _ = iter.next_if(|t| matches!(t.kind, TokenKind::Comma));
}

fn parse_token(insts: &mut Vec<Inst>, iter: &mut TokenIter, token: &Token) -> ParseResult<()> {
    match token.kind {
        TokenKind::Ident(ref label) => {
            expect_colon(iter)?;
            insts.push(Inst::new(token, InstKind::Label(label.clone())));
        }
        TokenKind::Dot => match iter.peek() {
            Some(Token {
                kind: TokenKind::Dbyte,
                ..
            }) => {
                let _ = iter.next();
                if let Some(Token {
                    kind: TokenKind::Be,
                    ..
                }) = iter.peek()
                {
                    let _ = iter.next();
                    if let Ok(num) = parse_number(iter) {
                        insts.push(Inst::new(token, InstKind::DbyteBe(Operand::Literal(num))));
                    } else if let Ok(label) = parse_identifier(iter) {
                        insts.push(Inst::new(token, InstKind::DbyteBe(Operand::Label(label))));
                    } else {
                        return Err(ParseError::ExpectedNumberOrIdentifier);
                    }
                } else {
                    let num = parse_number(iter)?;
                    insts.push(Inst::new(token, InstKind::Dbyte(Operand::Literal(num))));
                }
            }
            Some(Token {
                kind: TokenKind::Byte,
                ..
            }) => {
                let _ = iter.next();
                let content = parse_number_u8(iter)?;
                insts.push(Inst::new(token, InstKind::Byte(content)));
            }
            _ => {
                return Err(ParseError::ExpectedKeyword);
            }
        },
        TokenKind::Cls => {
            insts.push(Inst::new(token, InstKind::Op(Op::Cls)));
        }
        TokenKind::Ret => {
            insts.push(Inst::new(token, InstKind::Op(Op::Ret)));
        }
        TokenKind::Scd => {
            let dist = parse_number_u8(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Scd(dist))));
        }
        TokenKind::Scl => {
            insts.push(Inst::new(token, InstKind::Op(Op::Scl)));
        }
        TokenKind::Scr => {
            insts.push(Inst::new(token, InstKind::Op(Op::Scr)));
        }
        TokenKind::Exit => {
            insts.push(Inst::new(token, InstKind::Op(Op::Exit)));
        }
        TokenKind::Low => {
            insts.push(Inst::new(token, InstKind::Op(Op::Low)));
        }
        TokenKind::High => {
            insts.push(Inst::new(token, InstKind::Op(Op::High)));
        }
        TokenKind::Sys => {
            if let Ok(addr) = parse_address(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Sys(addr))));
            } else if let Ok(label) = parse_identifier(iter) {
                insts.push(Inst::new(token, InstKind::Sys(label)));
            } else {
                return Err(ParseError::ExpectedAddressOrIdentifier);
            }
        }
        TokenKind::Jp => {
            if let Ok(addr) = parse_address(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Jp(addr))));
            } else if let Ok(label) = parse_identifier(iter) {
                insts.push(Inst::new(token, InstKind::Jp(label)));
            } else {
                return Err(ParseError::ExpectedAddressOrIdentifier);
            }
        }
        TokenKind::Call => {
            if let Ok(addr) = parse_address(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Call(addr))));
            } else if let Ok(label) = parse_identifier(iter) {
                insts.push(Inst::new(token, InstKind::Call(label)));
            } else {
                return Err(ParseError::ExpectedAddressOrIdentifier);
            }
        }
        TokenKind::Se => {
            let reg = parse_register(iter)?;
            optional_comma(iter);
            let num = parse_number_u8(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Se(reg, num))));
        }
        TokenKind::Sne => {
            let reg = parse_register(iter)?;
            optional_comma(iter);
            let num = parse_number_u8(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Sne(reg, num))));
        }
        TokenKind::Ld => {
            let reg = parse_register(iter)?;
            optional_comma(iter);
            if let Ok(num) = parse_number_u8(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Ldl(reg, num))));
            } else if let Ok(reg2) = parse_register(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Ld(reg, reg2))));
            } else {
                return Err(ParseError::ExpectedNumberOrRegister);
            }
        }
        TokenKind::Ldi => {
            if let Ok(addr) = parse_address(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Ldi(addr))));
            } else if let Ok(label) = parse_identifier(iter) {
                insts.push(Inst::new(token, InstKind::Ldi(label)));
            } else {
                return Err(ParseError::ExpectedAddressOrIdentifier);
            }
        }
        TokenKind::Or => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Or(reg1, reg2))));
        }
        TokenKind::And => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::And(reg1, reg2))));
        }
        TokenKind::Xor => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Xor(reg1, reg2))));
        }
        TokenKind::Add => {
            let reg = parse_register(iter)?;
            optional_comma(iter);
            if let Ok(num) = parse_number_u8(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Addl(reg, num))));
            } else if let Ok(reg2) = parse_register(iter) {
                insts.push(Inst::new(token, InstKind::Op(Op::Add(reg, reg2))));
            } else {
                return Err(ParseError::ExpectedNumberOrRegister);
            }
        }
        TokenKind::Sub => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Sub(reg1, reg2))));
        }
        TokenKind::Subn => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Subn(reg1, reg2))));
        }
        TokenKind::Shr => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Shr(reg1, reg2))));
        }
        TokenKind::Shl => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Shl(reg1, reg2))));
        }
        TokenKind::Jpr => {
            let addr = parse_address(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Jpr(addr))));
        }
        TokenKind::Rnd => {
            let reg = parse_register(iter)?;
            optional_comma(iter);
            let number = parse_number_u8(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Rnd(reg, number))));
        }
        TokenKind::Drw => {
            let reg1 = parse_register(iter)?;
            optional_comma(iter);
            let reg2 = parse_register(iter)?;
            optional_comma(iter);
            let number = parse_number_u8(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Drw(reg1, reg2, number))));
        }
        TokenKind::Skp => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Skp(reg))));
        }
        TokenKind::Sknp => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Sknp(reg))));
        }
        TokenKind::Lddt => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Lddt(reg))));
        }
        TokenKind::Key => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Key(reg))));
        }
        TokenKind::Delay => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Delay(reg))));
        }
        TokenKind::Sound => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Sound(reg))));
        }
        TokenKind::Addi => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Addi(reg))));
        }
        TokenKind::Hex => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Hex(reg))));
        }
        TokenKind::Bcd => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Bcd(reg))));
        }
        TokenKind::Stor => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Stor(reg))));
        }
        TokenKind::Rstr => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Rstr(reg))));
        }
        TokenKind::Hexx => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Hexx(reg))));
        }
        TokenKind::Storx => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Storx(reg))));
        }
        TokenKind::Rstrx => {
            let reg = parse_register(iter)?;
            insts.push(Inst::new(token, InstKind::Op(Op::Rstrx(reg))));
        }
        _ => {
            // If we're still here, report error
            return Err(ParseError::UnmatchedToken(token.clone()));
        }
    }
    Ok(())
}

#[must_use]
pub fn parse(tokens: &[Token]) -> Ast {
    let mut insts = Vec::<Inst>::new();
    let mut iter = tokens.iter().peekable();
    while let Some(token) = iter.next() {
        if let Err(err) = parse_token(&mut insts, &mut iter, token) {
            // Oh no! So anyway...
            log::error!("{err}");
        }
    }
    Ast { insts }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{Token, TokenKind};

    #[test]
    fn parse_test() {
        let tokens = [Token {
            kind: TokenKind::Exit,
            line: 0,
            column: 0,
        }];
        let parsed = parse(&tokens);
        assert!(matches!(parsed.insts[0].kind, InstKind::Op(Op::Exit)));
    }
}
