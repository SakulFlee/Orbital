use crate::module::exports::orbital::module::module_impl::Guest;

wit_bindgen::generate!({
    path: "../../WIT/module.wit",
});

struct ExampleModule;
export!(ExampleModule);

impl Guest for ExampleModule {
    fn startup() {
        println!("ExampleModule startup called!");
    }
}
