//! Datalog compiler for Percival, shared with the client through WebAssembly.

#![warn(missing_docs)]

use chumsky::prelude::*;
use wasm_bindgen::prelude::*;
use yansi::Paint;

use percival::{ast::Program, codegen, errors::format_errors, parser::parser};

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

/// Compile a Percival program and return the result.
#[wasm_bindgen]
pub fn compile(src: &str) -> CompilerResult {
    thread_local! {
        static PARSER: BoxedParser<'static, char, Program, Simple<char>> = parser();
    }

    let mut src = String::from(src);
    if !src.ends_with('\n') {
        src += "\n";
    }
    CompilerResult(PARSER.with(|parser| {
        parser
            .parse(&src[..])
            .map_err(|err| format_errors(&src[..], err))
            .and_then(|prog| {
                let js = codegen::compile(&prog)
                    .map_err(|err| format!("{} {}", Paint::red("Error:"), err))?;
                Ok((prog, js))
            })
    }))
}

/// The result of a compilation.
#[wasm_bindgen]
pub struct CompilerResult(Result<(Program, String), String>);

#[wasm_bindgen]
impl CompilerResult {
    /// Returns the compiled JavaScript program.
    pub fn js(&self) -> Option<String> {
        self.0.as_ref().ok().map(|(_, js)| js.clone())
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

    /// Returns the names of relations produced by this program, including imports.
    pub fn results(&self) -> Option<Vec<JsValue>> {
        self.0.as_ref().ok().map(|(prog, _)| {
            prog.results()
                .into_iter()
                .chain(prog.imports().into_iter())
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
