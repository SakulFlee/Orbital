use orbital::cgmath::{Point3, Quaternion, Rad, Vector3};
use orbital::app::{AppSettings, Module, App, sys_camera_controller};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CameraRealization,
    EngineEvent, EngineEvents, EnvironmentDescriptorResource, ImportQueueResource, LightDescriptorEcs,
    LightDirty, Position, Rotation,
};
use orbital::importer::{ImportTask, gltf::GltfImport};
use orbital::logging::{self, error, info};
use orbital::resources::{Camera, WorldEnvironmentDescriptor};

pub const NAME: &str = "Orbital-Demo-Project: DamagedHelmet";

pub fn entrypoint(
    event_loop_result: Result<orbital::winit::event_loop::EventLoop<()>, orbital::winit::error::EventLoopError>,
) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match App::new().add_module(DamagedHelmetModule).liftoff(event_loop, app_settings) {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_desktop_main!(entrypoint);

struct DamagedHelmetModule;

impl Module for DamagedHelmetModule {
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
        ecs.attach_component(&camera, Position(Point3::new(-10.0, 0.0, 0.0))).unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();

        let gpu_camera = Camera::new(
            Point3::new(-10.0, 0.0, 0.0),
            Quaternion::new(1.0, 0.0, 0.0, 0.0),
            45.0, 16.0 / 9.0, 0.1, 10000.0, 2.2,
            device, queue,
        );
        if let Some(mut store) = ecs.get_resource_mut::<orbital::ecs_bridge::EcsCameraStore>() {
            store.insert(camera.index, std::sync::Arc::new(std::sync::RwLock::new(gpu_camera)));
        }
        ecs.attach_component(&camera, CameraRealization).unwrap();
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

        // Import model
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Assets/Models/DamagedHelmet.glb".into(),
                task: GltfImport::WholeFile,
            });
        }

        // Spawn lights
        let light1 = ecs.spawn_entity();
        ecs.attach_component(&light1, LightDescriptorEcs::new_point(
            Vector3::new(1.0, 1.0, 1.0), 10.0,
        )).unwrap();
        ecs.attach_component(&light1, Position(Point3::new(5.0, 5.0, 5.0))).unwrap();
        ecs.attach_component(&light1, LightDirty(true)).unwrap();

        let light2 = ecs.spawn_entity();
        ecs.attach_component(&light2, LightDescriptorEcs::new_directional(
            Vector3::new(-1.0, -1.0, -1.0),
            Vector3::new(1.0, 1.0, 1.0),
            1.0,
        )).unwrap();
        ecs.attach_component(&light2, Position(Point3::new(0.0, 0.0, 0.0))).unwrap();
        ecs.attach_component(&light2, LightDirty(true)).unwrap();

        // Grab cursor
        if let Some(mut events) = ecs.get_resource_mut::<EngineEvents>() {
            events.push(EngineEvent::CursorGrabbed(true));
            events.push(EngineEvent::CursorVisible(false));
        }

        vec![
            sys_camera_controller.into_system(),
        ]
    }
}
