use orbital::app::{App, AppSettings, Module};
use orbital::ecs::{System, World};
use orbital::logging::{error, info};
use orbital_iced::{IcedBridgeModule, IcedState, IcedUiState};

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(target_os = "android"))]
    orbital::logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let app_settings = AppSettings {
        vsync_enabled: true,
        name: "Iced Demo".to_string(),
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(IcedDemoModule)
        .add_module(IcedBridgeModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

struct IcedDemoModule;

impl Module for IcedDemoModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        let state = IcedState::new()
            .with_title("Orbital + Iced")
            .with_button_label("Click Me!");

        ecs.insert_resource(IcedUiState(state));

        info!("Iced demo module registered");
        vec![]
    }
}
