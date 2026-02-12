use crate::bindings::exports::orbital::wit_test::test_interface::Guest;

wit_bindgen::generate!({
    path: "wit/world.wit",
});

pub struct TestImpl;

impl Guest for TestImpl {
    fn test_function() {
        println!("Test!");
    }
}

export!(TestImpl);
