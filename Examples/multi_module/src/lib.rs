use orbital::app::{App, AppSettings};
use orbital::logging::{error, info};

mod modules;
use modules::camera_module::CameraModule;
use modules::light_module::LightModule;
use modules::model_module::ModelModule;

pub const NAME: &str = "Orbital-Demo-Project: MultiModule";

#[orbital::entrypoint]
pub fn entrypoint(event_loop: orbital::winit::event_loop::EventLoop<()>) {
    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match App::new()
        .add_module(CameraModule)
        .add_module(ModelModule)
        .add_module(LightModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}
