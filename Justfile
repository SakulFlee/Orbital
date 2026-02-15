# Settings
WASI_PATH := '/opt/wasi-sdk/bin/clang'

default:
  just --list

# WIT build tasks
build-wit-modules: build-wit-test-module-rust build-wit-test-module-c

build-wit-test-module-rust:
  cargo build --target wasm32-wasip2 --package wit-test-module-rust

build-wit-test-module-c: generate-wit-bindings-for-c
  {{ WASI_PATH }} \
    --target=wasm32-wasi \
    -Wl,--no-entry \
    -Wl,--export=exports_orbital_core_module_startup \
    -mexec-model=reactor \
    -Oz \
    -o wasi-modules/test-module-c.wasm \
    wit-bindings/c/test-module/src/lib.c

# WIT generation tasks
generate-wit-bindings-for-c:
  wit-bindgen c --world orbital --out-dir wit-bindings/c/bindings/ WIT/

