use alloc::{string::String, vec::Vec};

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum TokenKind {
    // single character
    Colon,
    Comma,
    Dot,
    // literals
    Ident(String),
    Reg(u32),
    Num(u32),
    Addr(u32),
    // keywords (mnemonics)
    Cls,
    Ret,
    Scd,
    Scl,
    Scr,
    Exit,
    Low,
    High,
    Sys,
    Jp,
    Call,
    Se,
    Sne,
    Ld,
    Add,
    Or,
    And,
    Xor,
    Sub,
    Subn,
    Shr,
    Shl,
    Ldi,
    Jpr,
    Rnd,
    Drw,
    Skp,
    Sknp,
    Lddt,
    Key,
    Delay,
    Sound,
    Addi,
    Hex,
    Bcd,
    Stor,
    Rstr,
    Hexx,
    Storx,
    Rstrx,
    // more keywords
    Byte,
    Dbyte,
    Be,
}

impl core::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match *self {
            Self::Ident(ref i) => write!(f, "Ident(\"{i}\")"),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl TokenKind {
    fn keyword_from_str(string: &str) -> Option<Self> {
        Some(match string {
            "cls" => Self::Cls,
            "ret" => Self::Ret,
            "scd" => Self::Scd,
            "scl" => Self::Scl,
            "scr" => Self::Scr,
            "exit" => Self::Exit,
            "low" => Self::Low,
            "high" => Self::High,
            "sys" => Self::Sys,
            "jp" => Self::Jp,
            "call" => Self::Call,
            "se" => Self::Se,
            "sne" => Self::Sne,
            "ld" => Self::Ld,
            "add" => Self::Add,
            "or" => Self::Or,
            "and" => Self::And,
            "xor" => Self::Xor,
            "sub" => Self::Sub,
            "subn" => Self::Subn,
            "shr" => Self::Shr,
            "shl" => Self::Shl,
            "ldi" => Self::Ldi,
            "jpr" => Self::Jpr,
            "rnd" => Self::Rnd,
            "drw" => Self::Drw,
            "skp" => Self::Skp,
            "sknp" => Self::Sknp,
            "lddt" => Self::Lddt,
            "key" => Self::Key,
            "delay" => Self::Delay,
            "sound" => Self::Sound,
            "addi" => Self::Addi,
            "hex" => Self::Hex,
            "bcd" => Self::Bcd,
            "stor" => Self::Stor,
            "rstr" => Self::Rstr,
            "hexx" => Self::Hexx,
            "storx" => Self::Storx,
            "rstrx" => Self::Rstrx,
            "byte" => Self::Byte,
            "dbyte" => Self::Dbyte,
            "be" => Self::Be,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

impl core::fmt::Display for Token {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}: {}", self.line + 1, self.column + 1, self.kind)
    }
}

type LexChars<'a> = core::iter::Peekable<core::iter::Enumerate<core::str::Chars<'a>>>;

#[derive(Clone, Copy, Default)]
enum NumberKind {
    #[default]
    Hex,
    Decimal,
}

impl NumberKind {
    const fn from_char(from: char) -> Option<Self> {
        match from {
            'D' | 'd' => Some(Self::Decimal),
            'X' | 'x' => Some(Self::Hex),
            _ => None,
        }
    }
}

fn lex_number_kind(chars: &mut LexChars<'_>) -> NumberKind {
    if let Some((_, from)) = chars.next_if(|(_, from)| NumberKind::from_char(*from).is_some()) {
        NumberKind::from_char(from).unwrap()
    } else {
        NumberKind::Hex
    }
}

fn lex_hex_digit(chars: &mut LexChars<'_>) -> Option<u32> {
    if let Some((_, digit)) = chars.next_if(|(_, digit)| digit.is_ascii_hexdigit()) {
        Some(match digit {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            'A' | 'a' => 0xA,
            'B' | 'b' => 0xB,
            'C' | 'c' => 0xC,
            'D' | 'd' => 0xD,
            'E' | 'e' => 0xE,
            'F' | 'f' => 0xF,
            _ => unreachable!(),
        })
    } else {
        None
    }
}

fn lex_hex_number(chars: &mut LexChars<'_>) -> Option<u32> {
    let mut number = lex_hex_digit(chars)?;
    while let Some(digit) = lex_hex_digit(chars) {
        number = (number << 4) + digit;
    }
    Some(number)
}

fn lex_dec_digit(chars: &mut LexChars<'_>) -> Option<u32> {
    if let Some((_, digit)) = chars.next_if(|(_, digit)| digit.is_ascii_digit()) {
        Some(match digit {
            '0' => 0,
            '1' => 1,
            '2' => 2,
            '3' => 3,
            '4' => 4,
            '5' => 5,
            '6' => 6,
            '7' => 7,
            '8' => 8,
            '9' => 9,
            _ => unreachable!(),
        })
    } else {
        None
    }
}

fn lex_dec_number(chars: &mut LexChars<'_>) -> Option<u32> {
    let mut number = lex_dec_digit(chars)?;
    while let Some(digit) = lex_dec_digit(chars) {
        number = (number * 10) + digit;
    }
    Some(number)
}

/*fn lex_digit(chars: &mut LexChars<'_>) -> Option<u32> {
    match lex_number_kind(chars) {
        NumberKind::Decimal => lex_dec_digit(chars),
        NumberKind::Hex => lex_hex_digit(chars),
    }
}*/

fn lex_number(chars: &mut LexChars<'_>) -> Option<u32> {
    match lex_number_kind(chars) {
        NumberKind::Decimal => lex_dec_number(chars),
        NumberKind::Hex => lex_hex_number(chars),
    }
}

enum Find<T> {
    Found(T),
    Continue,
    Break,
}

fn lex_root(chars: &mut LexChars<'_>, lineno: usize) -> Find<Token> {
    let (charno, character) = match chars.next() {
        Some(s) => s,
        None => return Find::Break,
    };
    if character.is_alphabetic() || character == '_' {
        let mut ident = String::from(character);
        while let Some((_, character)) =
            chars.next_if(|(_, character)| character.is_alphanumeric() || *character == '_')
        {
            ident.push(character);
        }
        if let Some(kind) = TokenKind::keyword_from_str(&ident) {
            return Find::Found(Token {
                line: lineno,
                column: charno,
                kind,
            });
        } else {
            return Find::Found(Token {
                line: lineno,
                column: charno,
                kind: TokenKind::Ident(ident),
            });
        }
    }
    if character.is_ascii_whitespace() {
        return Find::Continue; // no need to handle whitespace
    }
    let kind = match character {
        ';' => {
            // no need to handle comment
            return Find::Break;
        }
        ':' => Find::Found(TokenKind::Colon),
        ',' => Find::Found(TokenKind::Comma),
        '.' => Find::Found(TokenKind::Dot),
        '$' => {
            if let Some(number) = lex_hex_number(chars) {
                Find::Found(TokenKind::Reg(number))
            } else {
                log::error!(
                    "Expected hex number after '$' at {}:{}",
                    lineno + 1,
                    charno + 1,
                );
                Find::Continue
            }
        }
        '#' => {
            if let Some(number) = lex_number(chars) {
                Find::Found(TokenKind::Num(number))
            } else {
                log::error!("Expected number after '#' at {}:{}", lineno + 1, charno + 1);
                Find::Continue
            }
        }
        '&' => {
            if let Some(number) = lex_number(chars) {
                Find::Found(TokenKind::Addr(number))
            } else {
                log::error!("Expected number after '&' at {}:{}", lineno + 1, charno + 1);
                Find::Continue
            }
        }
        _ => {
            log::error!(
                "Unknown character '{character}' at {}:{}",
                lineno + 1,
                charno + 1,
            );
            Find::Continue
        }
    };
    match kind {
        Find::Found(kind) => Find::Found(Token {
            kind,
            line: lineno,
            column: charno,
        }),
        Find::Continue => Find::Continue,
        Find::Break => Find::Break,
    }
}

fn lex_line(tokens: &mut Vec<Token>, line: &str, lineno: usize) {
    let mut chars = line.chars().enumerate().peekable();
    loop {
        let find = lex_root(&mut chars, lineno);
        match find {
            Find::Found(f) => tokens.push(f),
            Find::Continue => {}
            Find::Break => break,
        }
    }
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut tokens = Vec::<Token>::new();
    let mut lines = input.lines().enumerate();
    for (lineno, line) in &mut lines {
        lex_line(&mut tokens, line, lineno);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_test() {
        const INPUT: &str = "exit\n";
        let tokens = lex(INPUT);
        assert!(matches!(tokens[0].kind, TokenKind::Exit));
    }
}
