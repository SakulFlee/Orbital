use orbital::ecs::{Commands, System, World};
use orbital::ecs_bridge::DeltaTime;
use orbital::logging::{self, error, info};
use orbital::app::{App, AppSettings, Module};
use orbital::twod::{Camera2D, ShapeDescriptor};

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
        name: "2D Scene".to_string(),
        back_presses_to_exit: 3,
        ..AppSettings::default()
    };

    match App::new()
        .add_module(Scene2DModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

struct Scene2DModule;

impl Module for Scene2DModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Create a 2D camera
        let camera = Camera2D::orthographic(1.0);
        ecs.insert_resource(camera);

        // Create some 2D shapes
        let square = ShapeDescriptor::solid_quad([0.9, 0.3, 0.2, 1.0]);
        let circle = ShapeDescriptor::solid_circle([0.2, 0.8, 0.3, 1.0]);
        let triangle = ShapeDescriptor::solid_triangle([0.3, 0.4, 0.9, 1.0]);

        info!("2D Scene initialized with camera, square, circle, and triangle");

        vec![]
    }
}
