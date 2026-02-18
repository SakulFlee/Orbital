use crate::Module;

#[test]
fn test_module() {
    println!(
        "NOTE: If the following test FAILS, make sure you have (re-)build the 'wit_test_module' crate for WASM-WASI-P2:"
    );
    println!("cargo build --target wasm32-wasip2 --package wit_test_module");

    let mut module = Module::new(include_bytes!(
        "../../../target/wasm32-wasip2/debug/wit_test_module.wasm"
    ))
    .expect("Failed to initialize module");
    module
        .call_startup_function()
        .expect("Failed to call startup function!");
}
