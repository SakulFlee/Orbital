use std::sync::Arc;

use orbital::cgmath::{Point3, Quaternion, Rad, Vector3};
use orbital::app::{AppSettings, Module, ModuleRuntime};
use orbital::ecs::{IntoSystem, Res, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CameraDirty, CameraRealization, DeltaTime, EcsCameraStore,
    EngineEvent, EngineEvents, EnvironmentDescriptorResource, LightDescriptorEcs, LightDirty,
    Position, Rotation,
};
use orbital::logging::{self, error, info};
use orbital::resources::{Camera, WorldEnvironmentDescriptor};

pub const NAME: &str = "Orbital-Demo-Project: MultiModule";

pub fn entrypoint(
    event_loop_result: Result<orbital::winit::event_loop::EventLoop<()>, orbital::winit::error::EventLoopError>,
) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match ModuleRuntime::liftoff(event_loop, app_settings, CombinedModule) {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_desktop_main!(entrypoint);

// ---------------------------------------------------------------------------
// Combined Module — merges camera + light sub-modules
// ---------------------------------------------------------------------------

struct CombinedModule;

impl Module for CombinedModule {
    fn setup(
        &self,
        ecs: &mut World,
        device: &orbital::wgpu::Device,
        queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        let mut systems: Vec<Box<dyn System>> = Vec::new();

        // Sub-module A: Camera + Environment
        systems.extend(CameraModule.setup(ecs, device, queue));
        info!("CameraModule contributed {} systems", systems.len());

        // Sub-module B: Light
        let light_systems = LightModule.setup(ecs, device, queue);
        info!("LightModule contributed {} systems", light_systems.len());
        systems.extend(light_systems);

        info!("Total systems: {}", systems.len());
        systems
    }
}

// ---------------------------------------------------------------------------
// Sub-module A — Camera + Environment
// ---------------------------------------------------------------------------

struct CameraModule;

impl Module for CameraModule {
    fn setup(
        &self,
        ecs: &mut World,
        device: &orbital::wgpu::Device,
        queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn camera
        let camera = ecs.spawn_entity();
        ecs.attach_component(&camera, CameraDescriptorEcs {
            label: "Default".into(),
            aspect: 16.0 / 9.0,
            fovy: Rad(std::f32::consts::FRAC_PI_4),
            near: 0.1,
            far: 10000.0,
            global_gamma: 2.2,
        }).unwrap();
        ecs.attach_component(&camera, Position(Point3::new(0.0, 2.0, 5.0))).unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();

        let gpu_camera = Camera::new(
            Point3::new(0.0, 2.0, 5.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
            45.0, 16.0 / 9.0, 0.1, 10000.0, 2.2,
            device, queue,
        );
        if let Some(mut store) = ecs.get_resource_mut::<orbital::ecs_bridge::EcsCameraStore>() {
            store.insert(camera.index, std::sync::Arc::new(std::sync::RwLock::new(gpu_camera)));
        }
        ecs.attach_component(&camera, CameraRealization).unwrap();
        ecs.attach_component(&camera, CameraDirty(false)).unwrap();
        ecs.insert_resource(ActiveCamera(camera));

        // Set environment
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size: 2048,
                path: "Assets/WorldEnvironments/PhotoStudio.hdr".to_string(),
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
            sys_roll.into_system(),
        ]
    }
}

// ---------------------------------------------------------------------------
// Sub-module B — Light
// ---------------------------------------------------------------------------

struct LightModule;

impl Module for LightModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn a directional light
        let light = ecs.spawn_entity();
        ecs.attach_component(&light, LightDescriptorEcs::new_directional(
            Vector3::new(-1.0, -1.0, -1.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
        )).unwrap();
        ecs.attach_component(&light, Position(Point3::new(0.0, 0.0, 0.0))).unwrap();
        ecs.attach_component(&light, LightDirty(true)).unwrap();

        // Spawn a point light
        let light2 = ecs.spawn_entity();
        ecs.attach_component(&light2, LightDescriptorEcs::new_point(
            Vector3::new(1.0, 1.0, 1.0), 5.0,
        )).unwrap();
        ecs.attach_component(&light2, Position(Point3::new(3.0, 3.0, 3.0))).unwrap();
        ecs.attach_component(&light2, LightDirty(true)).unwrap();

        vec![] // Light has no per-frame systems
    }
}

// ---------------------------------------------------------------------------
// Shared system
// ---------------------------------------------------------------------------

fn sys_roll(rot: &mut Rotation, dt: Res<DeltaTime>) {
    rot.rotate_roll(Rad(0.3 * dt.0 as f32));
}
