use app_example::entrypoint::entrypoint;
use orbital::winit::event_loop::EventLoop;

fn main() {
    let event_loop = EventLoop::builder().build();

    entrypoint(event_loop);
}
