use orbital::app::sys_camera_controller;
use orbital::app::{App, AppSettings, Module};
use orbital::cgmath::{Point3, Rad};
use orbital::ecs::{IntoSystem, Res, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, DeltaTime, EnvironmentDescriptorResource,
    Position, Rotation,
};
use orbital::logging::{error, info};
use orbital::resources::WorldEnvironmentDescriptor;

pub const NAME: &str = "Orbital-Demo-Project: RollCamera";

#[orbital::entrypoint]
pub fn entrypoint(event_loop: orbital::winit::event_loop::EventLoop<()>) {
    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match App::new()
        .add_module(RollCameraModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

struct RollCameraModule;

impl Module for RollCameraModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn camera entity
        let camera = ecs.spawn_entity();
        ecs.attach_component(
            &camera,
            CameraDescriptorEcs {
                label: "Default".into(),
                aspect: 16.0 / 9.0,
                fovy: Rad(std::f32::consts::FRAC_PI_4),
                near: 0.1,
                far: 10000.0,
                global_gamma: 2.2,
            },
        )
        .unwrap();
        ecs.attach_component(&camera, Position(Point3::new(0.0, 0.0, 3.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        // Set environment
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size: 2048,
                path: "Assets/WorldEnvironments/PhotoStudio.hdr".to_string(),
                sampling_type: WorldEnvironmentDescriptor::DEFAULT_SAMPLING_TYPE,
                custom_specular_mip_level_count: None,
            },
        )));

        // Return systems
        vec![
            sys_camera_controller.into_system(),
            sys_roll_camera.into_system(),
        ]
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn sys_roll_camera(rot: &mut Rotation, dt: Res<DeltaTime>) {
    let roll_speed = 2.5_f32;
    let delta_roll = roll_speed * dt.0 as f32;
    rot.rotate_roll(Rad(delta_roll));
}
