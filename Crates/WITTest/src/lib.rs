wit_bindgen::generate!({
    world: "test-world",
});

struct TestImpl;

impl Guest for TestImpl {
    #[allow(async_fn_in_trait)]
    fn init() -> () {
        println!("Init!");
    }

    #[allow(async_fn_in_trait)]
    fn update() -> () {
        println!("Update!");
    }
}

export!(TestImpl);
