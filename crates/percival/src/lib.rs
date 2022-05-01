//! Core compiler code for the Percival language.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod ast;
pub mod codegen_js;
pub mod codegen_json;
pub mod errors;
pub mod parser;
