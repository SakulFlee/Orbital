wit_bindgen::generate!({
    world: "orbital",
    path: "../../../WIT/"
});

use crate::exports::orbital::core::{commands::CommandBuffer, module::Guest};

pub struct TestModule;

impl Guest for TestModule {
    fn startup() -> Option<CommandBuffer> {
        println!("'Hello World!' from WIT TestModule: Rust");

        None
    }
}
export!(TestModule);
