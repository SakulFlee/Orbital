#![allow(clippy::all)]

use shared::entrypoint;

#[cfg(test)]
mod tests;

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    entrypoint();
}
