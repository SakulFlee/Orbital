use crate::module::exports::orbital::core::{
    commands::Command,
    module::{CommandBuffer, Guest},
};

wit_bindgen::generate!({
    path: "../../WIT/",
    world: "orbital",
});

struct ExampleModule;
export!(ExampleModule);

impl Guest for ExampleModule {
    #[allow(async_fn_in_trait)]
    fn startup() -> CommandBuffer {
        println!("ExampleModule startup called!");

        CommandBuffer {
            commands: vec![Command::RegisterSystem("Test".to_string())],
        }
    }
}
