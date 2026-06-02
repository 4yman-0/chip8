#![no_std]
#![warn(clippy::alloc_instead_of_core)]
extern crate alloc;
pub mod compiler;
pub mod lexer;
pub mod parser;
