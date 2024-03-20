use akimo_project::{error::ConnectorError, gpu_connector::GPUConnector, logging::*};
use winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder};

fn main() -> Result<(), ConnectorError> {
    log_init();

    info!("Akimo-Project: Engine");
    info!("(C) SakulFlee 2024");

    let size = PhysicalSize::new(1280, 720);

    let event_loop = EventLoop::new().expect("EventLoop boot failed");
    let window = WindowBuilder::new()
        .with_inner_size(size)
        .build(&event_loop)
        .expect("Winit window/canvas creation failed");

    let _connector = GPUConnector::new(Some(&window))?;

    let _ = event_loop.run(|_e, _w| {});

    Ok(())
}
