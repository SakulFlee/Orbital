use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use orbital::app::{sys_camera_controller, App, AppSettings, Module};
use orbital::cgmath::{InnerSpace, Point3, Quaternion, Rad, Vector3};
use orbital::debug_render::DebugModule;
use orbital::ecs::{
    Commands, ComponentAccess, IntoSystem, System, World,
};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, EnvironmentDescriptorResource,
    ImportQueueResource, LightDescriptorEcs, LightDirty, ModelDescriptorEcs, ModelDirty,
    ModelInstances, Position, Rotation,
};
use orbital::importer::{gltf::GltfImport, ImportTask};
use orbital::logging::{self, error, info};
use orbital::procgeo::scene::SceneBuilder;
use orbital::resources::{ShadowCaster, WorldEnvironmentDescriptor};
use winit::keyboard::KeyCode;

pub const NAME: &str = "Orbital-Demo-Project: ProcGeo Scene";

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match App::new()
        .add_module(ProcgeoSceneModule)
        .add_module(
            DebugModule::new()
                .with_toggle_key(KeyCode::F3)
                .with_freeze_key(KeyCode::F4),
        )
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_desktop_main!(entrypoint);

struct HelmetAdjuster {
    adjusted: AtomicBool,
    access: ComponentAccess,
}

impl HelmetAdjuster {
    fn new() -> Self {
        Self {
            adjusted: AtomicBool::new(false),
            access: ComponentAccess::new()
                .reads::<ModelDescriptorEcs>()
                .writes::<Position>()
                .writes::<Rotation>(),
        }
    }
}

impl System for HelmetAdjuster {
    fn name(&self) -> &str {
        "helmet_adjuster"
    }

    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, commands: &mut Commands) {
        if self.adjusted.load(Ordering::Relaxed) {
            return;
        }

        let descs = match world.get_component_store::<ModelDescriptorEcs>() {
            Some(s) => s,
            None => return,
        };

        for &eid in descs.dense.as_slice() {
            let idx = match descs.sparse.get(eid).and_then(|x| *x) {
                Some(i) => i,
                None => continue,
            };
            if !descs.components[idx].label.to_lowercase().contains("helmet") {
                continue;
            }

            let generation = world.generation(eid);
            let entity = orbital::ecs::Entity::new(eid, generation);

            commands.detach_component::<Position>(&entity);
            commands.attach_component(
                &entity,
                Position(Point3::new(0.0, 0.35, 0.0)),
            );

            commands.detach_component::<Rotation>(&entity);
            commands.attach_component(
                &entity,
                Rotation(Quaternion::new(0.7071, 0.0, 0.7071, 0.0)),
            );

            self.adjusted.store(true, Ordering::Relaxed);
            info!("Adjusted helmet: raised 0.35, rotated +90° Y to face +X");
            break;
        }
    }
}

fn spawn_light(
    ecs: &mut World,
    desc: LightDescriptorEcs,
    pos: Point3<f32>,
) {
    let entity = ecs.spawn_entity();
    ecs.attach_component(&entity, desc).unwrap();
    ecs.attach_component(&entity, Position(pos)).unwrap();
    ecs.attach_component(&entity, LightDirty(true)).unwrap();
    ecs.attach_component(&entity, ShadowCaster::default()).unwrap();
}

struct ProcgeoSceneModule;

impl Module for ProcgeoSceneModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn camera at the front, looking toward the rooms
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
        // Rotate +90° around Y so forward (+X) faces -Z (toward rooms)
        let rot = Quaternion::new(0.7071, 0.0, 0.7071, 0.0);
        ecs.attach_component(&camera, Position(Point3::new(0.0, 4.0, 10.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation(rot)).unwrap();
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

        // Load scene from RON file
        let scene = match SceneBuilder::load("Assets/Scenes/procgeo_demo.ron") {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to load scene: {}", e);
                return vec![sys_camera_controller.into_system()];
            }
        };

        // Build and spawn RON entities
        for (mesh, material, transform) in scene.build() {
            let entity = ecs.spawn_entity();
            ecs.attach_component(
                &entity,
                ModelDescriptorEcs {
                    label: "procgeo".into(),
                    mesh: Arc::new(mesh),
                    materials: vec![material],
                },
            )
            .unwrap();

            let mut instances = ModelInstances::new();
            instances.add_instance(transform);
            ecs.attach_component(&entity, instances).unwrap();
            ecs.attach_component(&entity, ModelDirty(true)).unwrap();
        }

        info!("Scene loaded with {} entities", scene.entities.len());

        // Import the DamagedHelmet into Room 2
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Assets/Models/DamagedHelmet.glb".into(),
                task: GltfImport::WholeFile,
            });
            info!("Queued DamagedHelmet import");
        }

        // ── Room Lights ──
        // Spot from above, pointing down toward the room center
        let spot_dir = (Point3::new(0.0, 0.0, 0.0) - Point3::new(0.0, 5.0, 0.0)).normalize();
        spawn_light(ecs,
            LightDescriptorEcs::new_spot(
                Vector3::new(1.0, 0.95, 0.85), 8.0,
                Vector3::new(spot_dir.x, spot_dir.y, spot_dir.z),
                0.3, 0.5),
            Point3::new(0.0, 5.0, 0.0));

        // Directional fill from above
        spawn_light(ecs,
            LightDescriptorEcs::new_directional(
                Vector3::new(0.0, -1.0, 0.0),
                Vector3::new(1.0, 1.0, 1.0), 1.5),
            Point3::new(0.0, 6.0, 0.0));

        // Warm point lights at front corners
        spawn_light(ecs,
            LightDescriptorEcs::new_point(Vector3::new(1.0, 0.85, 0.65), 3.0),
            Point3::new(-3.5, 3.0, 4.0));
        spawn_light(ecs,
            LightDescriptorEcs::new_point(Vector3::new(1.0, 0.85, 0.65), 3.0),
            Point3::new(3.5, 3.0, 4.0));

        // Cool point lights at back corners
        spawn_light(ecs,
            LightDescriptorEcs::new_point(Vector3::new(0.65, 0.8, 1.0), 3.0),
            Point3::new(-3.5, 3.0, -3.5));
        spawn_light(ecs,
            LightDescriptorEcs::new_point(Vector3::new(0.65, 0.8, 1.0), 3.0),
            Point3::new(3.5, 3.0, -3.5));

        info!("Spawned 6 shadow-casting lights");

        vec![
            sys_camera_controller.into_system(),
            Box::new(HelmetAdjuster::new()),
        ]
    }
}
