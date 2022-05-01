//! JSON AST emitter (is this really needed??)

use serde_json::to_string_pretty;

use crate::ast::Program;

/// Generates a JSON representation of the program AST.
pub fn compile(prog: &Program) -> Result<String, serde_json::Error> {
    to_string_pretty(prog)
}
