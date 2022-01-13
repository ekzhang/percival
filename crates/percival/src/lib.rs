//! Core compiler code for the Percival language.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod codegen;
pub mod errors;
pub mod parser;
