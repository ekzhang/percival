//! Datalog compiler for Percival, shared with the client through WebAssembly.

#![warn(missing_docs)]

use wasm_bindgen::prelude::*;

pub mod utils;

/// Computes the sum of two integers.
#[wasm_bindgen]
pub fn add(x: i32, y: i32) -> i32 {
    x + y
}
