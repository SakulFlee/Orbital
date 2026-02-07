use wasmtime::{Engine, component::Component};

fn run() {
    let engine = Engine::default();
    let component_bytes = include_bytes!("../../../target/wasm32-wasip2/debug/wit_test.wasm");
    let _component = Component::new(&engine, component_bytes).expect("Component import failed");
}

#[cfg(test)]
mod tests {
    use crate::run;

    #[test]
    fn test() {
        run();
    }
}
