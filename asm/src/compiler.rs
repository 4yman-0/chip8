// I'm sorry
extern crate std;
use std::collections::HashMap;

use crate::parser::{Ast, Inst, InstKind, Operand};
use alloc::vec::Vec;
use chip8_op::{Op, OpIntoError};

pub type CompileResult<T> = Result<T, CompileError>;

#[derive(Debug, Clone)]
pub enum CompileErrorKind {
    UndefinedLabel,
    DuplicateLabel,
    OpInto(OpIntoError),
}

impl From<OpIntoError> for CompileErrorKind {
    fn from(from: OpIntoError) -> Self {
        Self::OpInto(from)
    }
}

use CompileErrorKind as CompErrKind;

#[derive(Debug, Clone)]
pub struct CompileError {
    kind: CompileErrorKind,
    line: usize,
    column: usize,
}

use CompileError as CompErr;

impl CompileError {
    fn new(inst: &Inst, kind: CompErrKind) -> Self {
        Self {
            kind,
            line: inst.line,
            column: inst.column,
        }
    }
}

impl core::fmt::Display for CompErr {
    fn fmt(&self, w: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(w, "{}:{}: {:?}", self.line, self.column, self.kind)
    }
}

impl core::error::Error for CompErr {}

fn resolve_label(inst: &Inst, name: &str, labels: &HashMap<&str, u16>) -> CompileResult<u16> {
    labels
        .get(name)
        .copied()
        .ok_or(CompErr::new(inst, CompErrKind::UndefinedLabel))
}

fn resolve_operand(inst: &Inst, op: &Operand, labels: &HashMap<&str, u16>) -> CompileResult<u16> {
    match op {
        Operand::Literal(v) => Ok(*v),
        Operand::Label(name) => labels
            .get(name.as_str())
            .copied()
            .ok_or(CompErr::new(inst, CompErrKind::UndefinedLabel)),
    }
}

fn labels_from_ast(ast: &Ast) -> HashMap<&str, u16> {
    let mut labels = HashMap::<&str, u16>::new();
    let mut pc: u16 = 0x200;

    for inst in &ast.insts {
        match &inst.kind {
            InstKind::Label(name) => {
                if labels.insert(name, pc).is_some() {
                    log::error!(
                        "{}: {inst}",
                        CompErr::new(inst, CompErrKind::DuplicateLabel)
                    );
                }
            }

            InstKind::Byte(_) => pc += 1,

            _ => pc += 2,
        }
    }

    labels
}

fn emit_inst_code(
    inst: &Inst,
    output: &mut Vec<u8>,
    labels: &HashMap<&str, u16>,
) -> Result<(), CompErr> {
    match &inst.kind {
        InstKind::Label(_) => {}
        InstKind::Byte(content) => {
            output.push(*content);
        }
        InstKind::Dbyte(operand) => {
            let operand = resolve_operand(inst, operand, labels)?;
            output.extend_from_slice(&operand.to_be_bytes());
        }
        InstKind::DbyteBe(operand) => {
            let operand = resolve_operand(inst, operand, labels)?;
            output.extend_from_slice(&operand.to_ne_bytes());
        }
        InstKind::Sys(addr) => {
            let addr = resolve_label(inst, addr, labels)?;
            let opcode: u16 = Op::Sys(addr)
                .try_into()
                .map_err(|e: OpIntoError| CompErr::new(inst, e.into()))?;
            output.extend_from_slice(&opcode.to_ne_bytes());
        }
        InstKind::Jp(addr) => {
            let addr = resolve_label(inst, addr, labels)?;
            let opcode: u16 = Op::Jp(addr)
                .try_into()
                .map_err(|e: OpIntoError| CompErr::new(inst, e.into()))?;
            output.extend_from_slice(&opcode.to_ne_bytes());
        }
        InstKind::Call(addr) => {
            let addr = resolve_label(inst, addr, labels)?;
            let opcode: u16 = Op::Call(addr)
                .try_into()
                .map_err(|e: OpIntoError| CompErr::new(inst, e.into()))?;
            output.extend_from_slice(&opcode.to_ne_bytes());
        }
        InstKind::Ldi(addr) => {
            let addr = resolve_label(inst, addr, labels)?;
            let opcode: u16 = Op::Ldi(addr)
                .try_into()
                .map_err(|e: OpIntoError| CompErr::new(inst, e.into()))?;
            output.extend_from_slice(&opcode.to_ne_bytes());
        }
        InstKind::Op(op) => {
            let opcode: u16 = (*op)
                .try_into()
                .map_err(|e: OpIntoError| CompErr::new(inst, e.into()))?;
            output.extend_from_slice(&opcode.to_ne_bytes());
        }
    }
    Ok(())
}

#[must_use]
pub fn compile(ast: &Ast) -> Vec<u8> {
    // Pass 1: build label hashmap
    let labels = labels_from_ast(ast);

    // Pass 2: emit code
    let mut output = Vec::<u8>::new();

    for inst in &ast.insts {
        if let Err(err) = emit_inst_code(inst, &mut output, &labels) {
            log::error!("{err}");
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Ast, Inst, InstKind};

    #[test]
    fn compile_test() {
        let ast = Ast {
            insts: alloc::vec![Inst {
                kind: InstKind::Op(Op::Exit),
                line: 0,
                column: 0,
            },],
        };
        let compiled = compile(&ast);
        assert_eq!(compiled, [0x00, 0xFD]);
    }
}
