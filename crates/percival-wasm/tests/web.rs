//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use percival_wasm::compile;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn basic_compile() {
    assert!(compile("tc(x: 3, y: 4).").ok().is_some());
    assert!(compile("tc(x,").err().is_some());
}
