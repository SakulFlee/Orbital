use wasm_bindgen::prelude::*;
use shared::entrypoint;

#[wasm_bindgen]
pub fn wasm_entry() {
    entrypoint();
}
