//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use percival_wasm::add;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn add_two() {
    assert_eq!(add(2, 3), 5);
}
