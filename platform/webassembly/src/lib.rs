#![allow(clippy::all)]

use wasm_bindgen::prelude::*;

use shared::entrypoint;

#[cfg(test)]
mod tests;

#[wasm_bindgen]
pub fn wasm_entry() {
    entrypoint();
}
