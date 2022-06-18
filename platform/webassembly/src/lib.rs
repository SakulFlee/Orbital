use wasm_bindgen::prelude::*;
use shared::entrypoint;

#[wasm_bindgen]
pub fn main() {
    entrypoint();
}
