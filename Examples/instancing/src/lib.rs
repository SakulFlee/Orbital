use std::sync::Arc;

use orbital::cgmath::{Point3, Quaternion, Rad};
use orbital::app::{AppSettings, Module, ModuleRuntime, sys_camera_controller};
use orbital::ecs::{IntoSystem, Res, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CameraDirty, CameraRealization, DeltaTime, EcsCameraStore,
    EngineEvent, EngineEvents, EnvironmentDescriptorResource, ImportQueueResource, Position, Rotation,
};
use orbital::importer::{ImportTask, gltf::GltfImport};
use orbital::logging::{self, error, info};
use orbital::resources::{Camera, WorldEnvironmentDescriptor};

pub const NAME: &str = "Orbital-Demo-Project: Instancing Test";

pub fn entrypoint(
    event_loop_result: Result<orbital::winit::event_loop::EventLoop<()>, orbital::winit::error::EventLoopError>,
) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match ModuleRuntime::liftoff(event_loop, app_settings, InstancingModule) {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_desktop_main!(entrypoint);

struct InstancingModule;

impl Module for InstancingModule {
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

        // Environment
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::FromFile {
                cube_face_size: 2048,
                path: "Assets/WorldEnvironments/PhotoStudio.hdr".to_string(),
                sampling_type: WorldEnvironmentDescriptor::DEFAULT_SAMPLING_TYPE,
                custom_specular_mip_level_count: None,
            },
        )));

        // Import instancing test model
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Assets/Models/InstancingTest.glb".into(),
                task: GltfImport::WholeFile,
            });
        }

        // Grab cursor
        if let Some(mut events) = ecs.get_resource_mut::<EngineEvents>() {
            events.push(EngineEvent::CursorGrabbed(true));
            events.push(EngineEvent::CursorVisible(false));
        }

        vec![
            sys_camera_controller.into_system(),
            (|rot: &mut Rotation, dt: Res<DeltaTime>| {
                rot.rotate_roll(Rad(0.5 * dt.0 as f32));
            }).into_system(),
        ]
    }
}
