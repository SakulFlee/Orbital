use orbital::app::{App, AppSettings, Module, RenderOverlayResource};
use orbital::ecs::{System, World};
use orbital::logging::{error, info};

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
        let state = orbital_iced::IcedState::new()
            .with_title("Orbital + Iced")
            .with_button_label("Click Me!");

        let overlay = orbital_iced::IcedLayerRenderer::new(state);

        if ecs.get_resource::<RenderOverlayResource>().is_none() {
            ecs.insert_resource(RenderOverlayResource::new());
        }
        if let Some(res) = ecs.get_resource_mut::<RenderOverlayResource>() {
            res.add_layer_renderer(Box::new(overlay));
        }

        info!("Iced demo module registered");
        vec![]
    }
}
