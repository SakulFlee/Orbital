default:
  just --list

# WIT build tasks
build-wit-modules: build-wit-test-module-rust

build-wit-test-module-rust:
  cargo build --target wasm32-wasip2 --package wit-test-module-rust

