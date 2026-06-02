use chip8_op::{Op, OpFromError};
use std::io::{self, Write};

macro_rules! wprintln {
	($w:expr, $fmt:expr$(, $opt:expr)*) => {
		writeln!($w, $fmt$(, $opt)*).unwrap()
	}
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    let rom = std::fs::read(&args[1]).expect("ROM error");

    eprintln!("rom: {}", rom.len());

    let (chunks, remainder) = if args.get(2).is_some_and(|a| a == "-offset") {
        rom[1..].as_chunks::<2>()
    } else {
        rom.as_chunks::<2>()
    };

    eprintln!("chunks: {}", chunks.len());
    eprintln!("remainder: {}", remainder.len());

    {
        let mut out = io::stdout();

        for (idx, op) in chunks.iter().enumerate() {
            let idx = idx * 2; // byte offset
            let op = u16::from_le_bytes(*op);
            match Op::try_from(op) {
                Ok(op) => wprintln!(out, "{:04X}: {op}", idx),
                Err(OpFromError::UnknownOp(op)) => wprintln!(out, "{:04X}: ???	({op:04X})", idx),
                Err(err) => wprintln!(out, "{:04X}: ???	({err})", idx),
            }
        }

        if !remainder.is_empty() {
            wprintln!(out, "REMAINDER: {:02X}", remainder[0]);
        }

        out.flush().unwrap();
    }
}
