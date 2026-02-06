mod bindings;
use bindings::exports::orbital::wit_test::test_interface::Guest;

pub struct TestImpl;

impl Guest for TestImpl {
    fn test_function() {
        println!("Test!");
    }
}
