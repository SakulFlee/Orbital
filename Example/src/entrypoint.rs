use orbital::{
    app::{App, AppChange, AppRuntime, AppSettings},
    log::debug,
    logging,
    winit::{error::EventLoopError, event_loop::EventLoop},
};

pub fn entrypoint(event_loop_result: Result<EventLoop<()>, EventLoopError>) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    AppRuntime::liftoff::<X>(event_loop, AppSettings::default()).expect("Runtime failure");
}

pub struct X {}

impl App for X {
    fn initialize() -> Self
    where
        Self: Sized,
    {
        X {}
    }

    async fn on_resume(
        &mut self,
        _config: &orbital::wgpu::SurfaceConfiguration,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) where
        Self: Sized,
    {
        debug!("Resumed");
    }

    async fn on_update(&mut self) -> Option<Vec<AppChange>>
    where
        Self: Sized,
    {
        debug!("Update");

        None
    }
}
