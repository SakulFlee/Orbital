#[cfg(target_arch = "wasm32")]
compile_error!(
    "Rayon is not available on WASM without the 'wasm-threads' feature. \
     See NOTE-for-WASM.md for setup instructions."
);
