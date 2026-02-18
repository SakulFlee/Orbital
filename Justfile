# Settings
WASI_SDK_PATH := '/opt/wasi-sdk/'

default:
  just --list

# WIT build tasks
build-wit-modules: build-wit-test-module-rust build-wit-test-module-c build-wit-test-module-cpp build-wit-test-module-csharp build-wit-test-module-go
  ls -l wasi-modules/
  du -sh wasi-modules/* | sort -h

build-wit-test-module-rust:
  cargo build --release --target wasm32-wasip2 --package wit-test-module-rust
  cp target/wasm32-wasip2/release/wit_test_module_rust.wasm wasi-modules/test-module-rust.wasm

build-wit-test-module-c: download-wasi-p1-to-p2-reactor generate-wit-bindings-for-c  
  mkdir -p wit-bindings/c/test-module/out/

  {{ WASI_SDK_PATH }}/bin/clang \
    --target=wasm32-wasi \
    -Wl,--no-entry \
    -Wl,--export=exports_orbital_core_module_startup \
    -mexec-model=reactor \
    -Oz \
    -o wit-bindings/c/test-module/out/test-module-raw.wasm \
    wit-bindings/c/test-module/src/module.c

  wasm-tools component embed \
    -w orbital \
    WIT/ \
    wit-bindings/c/test-module/out/test-module-raw.wasm \
    -o wit-bindings/c/test-module/out/test-module-wit.wasm
  wasm-tools component new \
    --adapt target/wasi_snapshot_preview1.reactor \
    wit-bindings/c/test-module/out/test-module.wasm \
    -o wit-bindings/c/test-module/out/test-module-component.wasm

  cp wit-bindings/c/test-module/out/test-module-component.wasm \
    wasi-modules/test-module-c.wasm

build-wit-test-module-cpp: download-wasi-p1-to-p2-reactor generate-wit-bindings-for-cpp
  mkdir -p wit-bindings/cpp/test-module/out/

  {{ WASI_SDK_PATH }}/bin/clang++ \
    -std=c++20 \
    --target=wasm32-wasi \
    -fno-exceptions \
    -Wl,--no-entry \
    -mexec-model=reactor \
    -Oz \
    -o wit-bindings/cpp/test-module/out/test-module-raw.wasm \
    wit-bindings/cpp/test-module/src/module.cpp \
    wit-bindings/cpp/bindings/orbital.cpp \
    wit-bindings/cpp/bindings/orbital_component_type.o

  wasm-tools component embed \
    -w orbital \
    WIT/ \
    wit-bindings/cpp/test-module/out/test-module-raw.wasm \
    -o wit-bindings/cpp/test-module/out/test-module-wit.wasm
  wasm-tools component new \
    --adapt target/wasi_snapshot_preview1.reactor \
    wit-bindings/cpp/test-module/out/test-module-raw.wasm \
    -o wit-bindings/cpp/test-module/out/test-module-component.wasm

  cp wit-bindings/cpp/test-module/out/test-module-component.wasm \
    wasi-modules/test-module-cpp.wasm

[working-directory: 'wit-bindings/csharp/test-module/']
build-wit-test-module-csharp: generate-wit-bindings-for-csharp  
  dotnet build --configuration Release
  cp bin/Release/net10.0/wasi-wasm/publish/test-module.wasm ../../../wasi-modules/test-module-csharp.wasm

[working-directory: 'wit-bindings/go/test-module/']
build-wit-test-module-go: download-wasi-p1-to-p2-reactor download-wasi-p1-to-p2-reactor
  mkdir -p out

  # Build WASI-P1 module
  GOARCH=wasm GOOS=wasip1 go build \
    -o out/test-module-raw.wasm \
    -buildmode=c-shared \
    -ldflags="-checklinkname=0 -s -w"

  # Turn into WASI-P2 + WIT
  wasm-tools component embed -w orbital ../../../WIT/ out/test-module-raw.wasm -o out/test-module-wit.wasm
  wasm-tools component new --adapt ../../../target/wasi_snapshot_preview1.reactor out/test-module-wit.wasm -o out/test-module-component.wasm

  # Copy
  cp out/test-module-component.wasm ../../../wasi-modules/test-module-go.wasm

download-wasi-p1-to-p2-reactor:
  mkdir -p target/

  # Download Go WASI Reactor if not present
  {{ if path_exists("target/wasi_snapshot_preview1.reactor") == "true" { "" } else { \
    "curl -L -o target/wasi_snapshot_preview1.reactor https://github.com/bytecodealliance/wasmtime/releases/download/v39.0.1/wasi_snapshot_preview1.reactor.wasm" \
  }}}

# WIT generation tasks
generate-wit-bindings-for-c:
  wit-bindgen c \
    --world orbital \
    --out-dir wit-bindings/c/bindings/ \
    WIT/

generate-wit-bindings-for-cpp:
  wit-bindgen cpp \
    --world orbital \
    --out-dir wit-bindings/cpp/bindings/ \
    WIT/

generate-wit-bindings-for-csharp:
   wit-bindgen csharp \
    --runtime native-aot \
    --world orbital \
    --out-dir wit-bindings/csharp/bindings/ \
    WIT/ 

