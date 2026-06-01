use archscript_core::compile;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile_archscript(source: &str) -> String {
    let result = compile(source);

    serde_json::to_string(&result)
        .unwrap_or_else(|_| {
            r#"{
                "success": false,
                "diagnostics": [{
                    "message": "Failed to serialize compile result"
                }]
            }"#
            .to_string()
        })
}