# Crate/Library: Shared

[![Rust](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/shared.yml/badge.svg)](https://github.com/rust-multiplatform/Base-Project-Template/actions/workflows/shared.yml)

This is our **one** crate that holds all the shared code for our Cross/Multi-platform apps.  
This crate will be compiled as a library by each platform in their target architecture and called from each platform.

## How does this work?

Rust is a cross-platform language that supports many different architectures and platforms.  
Building for Desktop Platforms (like Windows, Linux, macOS) is natively supported as long as the required tools are installed.
For Android, iOS and WebAssembly (WASM) we _"require"_ some additional tools to build.
In essence, we could do it without those tools but it would be much harder to do so.
Said tools commonly package our apps into .APK, .APP and .JS/WASM, link them against our library and make sure the entrypoint is called.

Inside `src/lib.rs` we have a public entrypoint defined:

```rust
pub fn entrypoint() {
    // Our cross-platform code here
}
```

Each platform calls this, while each platform itself has an entrypoint.  

Ideally, we **only** have our shared code.  
However, sometimes we need some very platform specific things that can only be done on said platform.
In this case we have _some_ non-shared/platform-specific code but overall this should only be true for very special cases as the Rust standard library works mostly the same on each platform/target.

## What does this crate do after being called?

This is the most basic of examples: A simple `"Hello World"` or better yet: `"Hello from Rust!"` to the standard output.  
On Desktop platforms this will open a Terminal/Console printing out the text and exiting.  
For Android, iOS and WebAssembly this will be logged to LogCat, Device Logs and the Browser Console respectively.  

**Independent of the platform the app will _exit_ right after being done. Depending on the platform the window (console _or_ app) may close before you even see it.**  
On Desktop you can call the executable from a terminal either by using the executable in `target/(debug|release|<arch>)/platform_*(.exe)` or using the `cargo run --package platform_<platform>` command.
For Android, iOS and WebAssembly enable persistent logging and reopen the app/website.

## Building

Requirements:  

- Rust and Cargo is installed, preferably via [Rustup](https://rustup.rs/). Use the stable channel ideally.

Debug/Development Build:

```shell
cargo build --package shared
```

Release/Production Build:

```shell
cargo build --package shared --release
```

> Optionally add `--verbose` to see what is happening.  
> Floods your console with message though possibly, use when there are build errors.

## Testing

Requirements:  

- Rust and Cargo is installed, preferably via [Rustup](https://rustup.rs/). Use the stable channel ideally.

Debug/Development Testing:

```shell
cargo test --package shared
```

Release/Production Testing:

```shell
cargo test --package shared --release
```

> Optionally add `--verbose` to see what is happening.  
> Floods your console with message though possibly, use when there are build errors.
