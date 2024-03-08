use app::App;
use winit::event_loop::ControlFlow;

mod app;

pub fn entrypoint() {
    hello_world();

    let app = App::new(ControlFlow::Poll);
    app.run();
}

pub fn hello_world() {
    println!("Hello, World!");
}
