//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use std::iter::IntoIterator;

use percival_wasm::compile;

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn basic_compile() {
    assert!(compile("tc(x: 3, y: 4).").js().is_some());
    assert!(compile("tc(x,").err().is_some());
}

#[wasm_bindgen_test]
fn deps_and_results() {
    fn to_js_vec<'a>(arr: impl IntoIterator<Item = &'a str>) -> Vec<JsValue> {
        arr.into_iter().map(JsValue::from_str).collect()
    }

    let result = compile("tc(x, y) :- edge(x, y). tc(x, y) :- hello(y, x). any(x) :- tc(x).");
    assert!(result.is_ok());
    assert_eq!(result.deps(), Some(to_js_vec(["edge", "hello"])));
    assert_eq!(result.results(), Some(to_js_vec(["any", "tc"])));

    let result = compile("tc(x, y) :- edge(x, y). edge(x: 2, y: 3).");
    assert!(result.is_ok());
    assert_eq!(result.deps(), Some(to_js_vec([])));
    assert_eq!(result.results(), Some(to_js_vec(["edge", "tc"])));

    let result = compile("bad");
    assert!(result.is_err());
    assert_eq!(result.deps(), None);
    assert_eq!(result.results(), None);
}
