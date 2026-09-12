use orbital::ecs::{Commands, System, World};
use orbital::ecs_bridge::DeltaTime;
use orbital::logging::{self, error, info};
use orbital::app::{App, AppSettings, Module};
use orbital::twod::{Camera2D, ShapeDescriptor, Vertex2D, Batch2D};
use orbital::text::FontData;
use orbital::ui::{UiButton, UiText, UiLayout, UiEntity, UiElement, UiBackground, UiFocus};
use orbital::ui::widgets;
use orbital::ecs::Events;
use orbital::ui::ButtonPressed;

pub const NAME: &str = "{{PROJECT_NAME}}";

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(target_os = "android"))]
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let app_settings = AppSettings {
        vsync_enabled: true,
        name: NAME.to_string(),
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(GameModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

struct GameModule;

impl Module for GameModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Create a 2D camera
        let camera = Camera2D::new(800.0, 600.0);
        ecs.insert_resource(camera);

        // Create some 2D shapes
        let square = ShapeDescriptor::rectangle(100.0, 100.0);
        let circle = ShapeDescriptor::circle(50.0);

        info!("2D scene initialized with camera and shapes");

        vec![]
    }
}
