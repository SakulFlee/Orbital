#![allow(clippy::all)]

use shared::entrypoint;

#[cfg(test)]
mod tests;

#[no_mangle]
pub extern "C" fn main_rs() {
    entrypoint();
}
