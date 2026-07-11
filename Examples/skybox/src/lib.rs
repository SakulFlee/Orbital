use std::sync::Arc;

use orbital::cgmath::{Point3, Quaternion, Rad};
use orbital::app::{AppSettings, Module, ModuleRuntime};
use orbital::ecs::{IntoSystem, Res, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CameraDirty, CameraRealization, DeltaTime, EngineEvent,
    EngineEvents, EnvironmentDescriptorResource, Position, Rotation,
};
use orbital::logging::{self, error, info};
use orbital::resources::{Camera, WorldEnvironmentDescriptor};

pub const NAME: &str = "Orbital-Demo-Project: SkyBox";

pub fn entrypoint(
    event_loop_result: Result<orbital::winit::event_loop::EventLoop<()>, orbital::winit::error::EventLoopError>,
) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match ModuleRuntime::liftoff(event_loop, app_settings, SkyboxModule) {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_desktop_main!(entrypoint);

struct SkyboxModule;

impl Module for SkyboxModule {
    fn setup(
        &self,
        ecs: &mut World,
        device: &orbital::wgpu::Device,
        queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn camera entity
        let camera = ecs.spawn_entity();
        ecs.attach_component(&camera, CameraDescriptorEcs {
            label: "Default".into(),
            aspect: 16.0 / 9.0,
            fovy: Rad(std::f32::consts::FRAC_PI_4),
            near: 0.1,
            far: 10000.0,
            global_gamma: 2.2,
        }).unwrap();
        ecs.attach_component(&camera, Position(Point3::new(0.0, 0.0, 3.0))).unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();

        let gpu_camera = Camera::new(
            Point3::new(0.0, 0.0, 3.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
            45.0, 16.0 / 9.0, 0.1, 10000.0, 2.2,
            device, queue,
        );
        ecs.attach_component(&camera, CameraRealization(Arc::new(std::sync::RwLock::new(gpu_camera)))).unwrap();
        ecs.attach_component(&camera, CameraDirty(false)).unwrap();
        ecs.insert_resource(ActiveCamera(camera));

        // Set initial environment
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size: 2048,
                path: "Assets/WorldEnvironments/Kloppenheim.hdr".to_string(),
                sampling_type: WorldEnvironmentDescriptor::DEFAULT_SAMPLING_TYPE,
                custom_specular_mip_level_count: None,
            },
        )));

        // Grab cursor
        if let Some(mut events) = ecs.get_resource_mut::<EngineEvents>() {
            events.push(EngineEvent::CursorGrabbed(true));
            events.push(EngineEvent::CursorVisible(false));
        }

        vec![
            (|rot: &mut Rotation, dt: Res<DeltaTime>| {
                let roll_speed = 0.3_f32;
                rot.rotate_roll(Rad(roll_speed * dt.0 as f32));
            }).into_system(),
        ]
    }
}
