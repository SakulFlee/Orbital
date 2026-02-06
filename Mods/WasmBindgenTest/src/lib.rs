use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    let s = format!("Hello, {name}");

    return s;
}