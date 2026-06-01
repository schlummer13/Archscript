// wasm/src/lib.rs
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_archscript(source: &str) -> String {
    let result = archscript_core::compile(source);
    serde_json::to_string_pretty(&result).unwrap()
}