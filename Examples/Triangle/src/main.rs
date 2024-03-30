use triangle::entrypoint;
use winit::event_loop::EventLoopBuilder;

fn main() {
    let event_loop = EventLoopBuilder::new().build();
    entrypoint(event_loop);
}
