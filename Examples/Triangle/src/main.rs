use akimo_runtime::winit::event_loop::EventLoopBuilder;
use app_example_triangle::entrypoint::entrypoint;

fn main() {
    let event_loop = EventLoopBuilder::new().build();

    entrypoint(event_loop);
}
