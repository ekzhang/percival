//! Datalog compiler for Percival, shared with the client through WebAssembly.

#![warn(missing_docs)]

use chumsky::prelude::*;
use wasm_bindgen::prelude::*;

use percival::{
    ast::Program,
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
                let src =
                    codegen::compile(&prog).map_err(|err| format!("Compiler error: {}", err))?;
                Ok((prog, src))
            })
    })
}

/// The result of a compilation.
#[wasm_bindgen]
pub struct CompilerResult(Result<(Program, String), String>);

#[wasm_bindgen]
impl CompilerResult {
    /// Returns the compiled JavaScript program.
    pub fn src(&self) -> Option<String> {
        self.0.as_ref().ok().map(|(_, src)| src.clone())
    }

    /// Returns the names of relations that are dependencies of this program.
    pub fn deps(&self) -> Option<Vec<JsValue>> {
        self.0.as_ref().ok().map(|(prog, _)| {
            prog.deps()
                .into_iter()
                .map(|s| JsValue::from_str(&s))
                .collect()
        })
    }

    /// Returns the names of relations that are produced by this program.
    pub fn results(&self) -> Option<Vec<JsValue>> {
        self.0.as_ref().ok().map(|(prog, _)| {
            prog.results()
                .into_iter()
                .map(|s| JsValue::from_str(&s))
                .collect()
        })
    }

    /// Returns a string representation of any errors during compilation.
    pub fn err(&self) -> Option<String> {
        self.0.as_ref().err().cloned()
    }

    /// Returns `true` if the result is `Ok`.
    pub fn is_ok(&self) -> bool {
        self.0.is_ok()
    }

    /// Returns `true` if the result is `Err`.
    pub fn is_err(&self) -> bool {
        self.0.is_err()
    }
}
