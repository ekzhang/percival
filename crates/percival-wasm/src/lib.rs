//! Datalog compiler for Percival, shared with the client through WebAssembly.

#![warn(missing_docs)]

use chumsky::prelude::*;
use wasm_bindgen::prelude::*;

use percival::{
    codegen,
    parser::{format_errors, parser},
};

/// Set a panic listener to display better error messages.
#[wasm_bindgen(start)]
pub fn start() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Computes the sum of two integers.
#[wasm_bindgen]
pub fn compile(src: &str) -> CompilerResult {
    CompilerResult({
        parser()
            .parse(src)
            .map_err(|err| format_errors(src, err))
            .and_then(|prog| {
                codegen::compile(&prog).map_err(|err| format!("Compiler error: {}", err))
            })
    })
}

/// The result of a compilation.
#[wasm_bindgen]
pub struct CompilerResult(Result<String, String>);

#[wasm_bindgen]
impl CompilerResult {
    /// Returns a string representation of any errors during compilation.
    pub fn err(&self) -> Option<String> {
        self.0.as_ref().err().cloned()
    }

    /// Returns the compiled JavaScript program.
    pub fn ok(&self) -> Option<String> {
        self.0.as_ref().ok().cloned()
    }
}
