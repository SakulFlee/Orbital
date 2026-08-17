use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{InnerSpace, Point3, Quaternion, Rad, Vector3};
use orbital::debug_render::DebugModule;
use orbital::ecs::{Commands, ComponentAccess, IntoSystem, Res, ResMut, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, DeltaTime, EnvironmentDescriptorResource,
    ImportQueueResource, LightDescriptorEcs, LightDirty, ModelDescriptorEcs, ModelDirty,
    ModelInstances, Position, Rotation,
};
use orbital::importer::{ImportTask, gltf::GltfImport};
use orbital::logging::{self, error, info};
use orbital::procgeo::scene::SceneBuilder;
use orbital::resources::WorldEnvironmentDescriptor;
use orbital::resources::{
    GeneratedSkyParameters, SamplingType, ShadowCaster, SunPosition, Transform,
};
use winit::keyboard::KeyCode;

pub const NAME: &str = "Orbital-Demo-Project: ProcGeo Scene";

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(target_os = "android"))]
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = false;
    app_settings.name = NAME.to_string();
    app_settings.back_presses_to_exit = 3;

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

orbital::make_main!(entrypoint);

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
                .reads::<ModelInstances>()
                .writes::<ModelInstances>()
                .writes::<ModelDirty>(),
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
            if !descs.components[idx]
                .label
                .to_lowercase()
                .contains("helmet")
            {
                continue;
            }

            let generation = world.generation(eid);
            let entity = orbital::ecs::Entity::new(eid, generation);

            // Read original transform from imported glTF, only modify position
            let original_transform = world
                .get_component_store::<ModelInstances>()
                .and_then(|store| {
                    store.sparse.get(eid).and_then(|x| *x).map(|idx| {
                        let map = &store.components[idx].0;
                        map.values().next().copied().unwrap_or(Transform::new(
                            Vector3::new(0.0, 0.0, 0.0),
                            Quaternion::new(1.0, 0.0, 0.0, 0.0),
                            Vector3::new(1.0, 1.0, 1.0),
                        ))
                    })
                })
                .unwrap_or(Transform::new(
                    Vector3::new(10.0, 1.85, 0.0),
                    Quaternion::new(0.7071, 0.0, -0.7071, 0.0),
                    Vector3::new(1.0, 1.0, 1.0),
                ));

            // Rotate 90° Y so the helmet faces +Z (toward camera)
            let correction = Quaternion::new(0.7071, 0.0, 0.7071, 0.0);
            let final_rot = correction * original_transform.rotation;

            let mut new_instances = ModelInstances::new();
            new_instances.add_instance(Transform::new(
                Vector3::new(10.0, 2.2, 0.0),
                final_rot,
                original_transform.scale,
            ));
            new_instances.add_instance(Transform::new(
                Vector3::new(20.0, 2.2, 0.0),
                final_rot,
                original_transform.scale,
            ));
            commands.detach_component::<ModelInstances>(&entity);
            commands.attach_component(&entity, new_instances);
            commands.detach_component::<ModelDirty>(&entity);
            commands.attach_component(&entity, ModelDirty(true));

            self.adjusted.store(true, Ordering::Relaxed);
            info!("Adjusted helmet: pos (10, 2.0, 0), rotated +90° Y toward camera");
            break;
        }
    }
}

struct ProcgeoSceneModule;

// ── TEMPORARY: light animation for shadow diagnosis ──────────────────

struct LightAnimator {
    t: f32,
    access: ComponentAccess,
    entity: orbital::ecs::Entity,
    started: bool,
}

impl LightAnimator {
    fn new(entity: orbital::ecs::Entity) -> Self {
        Self {
            t: 0.0,
            entity,
            started: false,
            access: ComponentAccess::new()
                .reads::<Position>()
                .writes::<Position>()
                .writes::<LightDirty>(),
        }
    }
}

impl System for LightAnimator {
    fn name(&self) -> &str {
        "light_animator"
    }
    fn access(&self) -> &ComponentAccess {
        &self.access
    }

    fn run(&mut self, world: &World, commands: &mut Commands) {
        let dt = world
            .get_resource::<orbital::ecs_bridge::DeltaTime>()
            .map(|d| d.0)
            .unwrap_or(0.016);
        self.t += dt as f32;
        let y = (self.t * 1.5).sin() * 3.0 + 8.0; // oscillate ±3 around y=8
        let new_pos = Position(Point3::new(0.0, y, 6.0));
        commands.detach_component::<Position>(&self.entity);
        commands.attach_component(&self.entity, new_pos);

        commands.detach_component::<LightDirty>(&self.entity);
        commands.attach_component(&self.entity, LightDirty(true));
    }
}

// ────────────────────────────────────────────────────────────────────

// Dynamic time-of-day sky. Uses the cheap in-place update path
// (`WorldEnvironment::update_sky_parameters`), so the descriptor is rewritten
// and realized every frame.
const DYNAMIC_SKY_CUBE_SIZE: u32 = 256;
const DYNAMIC_SKY_MIP_LEVELS: u32 = 3;

fn sys_animate_dynamic_sky(
    initial_hours: f32,
) -> impl FnMut(Res<DeltaTime>, ResMut<EnvironmentDescriptorResource>) {
    let mut clock = initial_hours;

    move |dt: Res<DeltaTime>, mut descriptor: ResMut<EnvironmentDescriptorResource>| {
        // Full day/night cycle every ~2 minutes of real time.
        clock = (clock + dt.0 as f32 / 5.0).rem_euclid(24.0);

        descriptor.0 = Some(WorldEnvironmentDescriptor::Generated {
            cube_face_size: DYNAMIC_SKY_CUBE_SIZE,
            sampling_type: SamplingType::GaussianBlur,
            custom_specular_mip_level_count: Some(DYNAMIC_SKY_MIP_LEVELS),
            parameters: Some(GeneratedSkyParameters {
                sun_position: SunPosition::TimeOfDay { hours: clock },
                ..GeneratedSkyParameters::default()
            }),
            dynamic: true,
        });
    }
}

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
        ecs.attach_component(&camera, Position(Point3::new(0.0, 7.0, 14.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation(rot)).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        // Dynamic procedural sky (in-place updates, cheap per frame).
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::Generated {
                cube_face_size: DYNAMIC_SKY_CUBE_SIZE,
                sampling_type: SamplingType::GaussianBlur,
                custom_specular_mip_level_count: Some(DYNAMIC_SKY_MIP_LEVELS),
                parameters: Some(GeneratedSkyParameters::default()),
                dynamic: true,
            },
        )));

        // Load scene from RON file
        let scene = match SceneBuilder::load("Scenes/procgeo_demo.ron") {
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
                file_path: "Models/DamagedHelmet.glb".into(),
                task: GltfImport::WholeFile,
            });
            info!("Queued DamagedHelmet import");
        }

        // ════════════════════════════════════════════════════════════
        // Spot light in Room 1 (Shadow Test)
        // Front of room (camera side), pointing backward toward center.
        // Objects cast shadows onto the floor and back wall.
        // ════════════════════════════════════════════════════════════
        let spot_light = ecs.spawn_entity();
        let spot_pos = Point3::new(0.0, 8.0, 6.0);
        let target = Point3::new(0.0, 0.0, -2.0);
        let dir = (target - spot_pos).normalize();
        ecs.attach_component(
            &spot_light,
            LightDescriptorEcs::new_spot(
                Vector3::new(1.0, 1.0, 1.0),
                50.0,
                Vector3::new(dir.x, dir.y, dir.z),
                0.1,
                0.44, // inner/outer ~6°/~25°
            ),
        )
        .unwrap();
        ecs.attach_component(&spot_light, Position(spot_pos))
            .unwrap();
        ecs.attach_component(&spot_light, LightDirty(true)).unwrap();
        ecs.attach_component(
            &spot_light,
            ShadowCaster {
                cascade_count: 0,
                bias: 0.0002,
                ..Default::default()
            },
        )
        .unwrap();

        // ── Point lights for Rooms 1 & 2 (x=-20, x=-10) ───────────
        for room_x in [-20.0, -10.0f32] {
            let pl = ecs.spawn_entity();
            ecs.attach_component(
                &pl,
                LightDescriptorEcs::new_point(Vector3::new(1.0, 1.0, 1.0), 50.0),
            )
            .unwrap();
            ecs.attach_component(&pl, Position(Point3::new(room_x, 5.0, 0.0)))
                .unwrap();
            ecs.attach_component(&pl, LightDirty(true)).unwrap();
            ecs.attach_component(&pl, ShadowCaster::default()).unwrap();
        }

        // ── Spot ring for Room 4 helmet (x=10) ──────────────────────
        let helmet_10 = Point3::new(10.0, 2.2, 0.0);
        for angle in [0.0, 2.1, 4.2f32] {
            let (s, c) = angle.sin_cos();
            let lpos = Point3::new(helmet_10.x + c * 3.0, 4.0, helmet_10.z + s * 3.0);
            let d = (helmet_10 - lpos).normalize();
            let hl = ecs.spawn_entity();
            ecs.attach_component(
                &hl,
                LightDescriptorEcs::new_spot(
                    Vector3::new(1.0, 0.9, 0.8),
                    30.0,
                    Vector3::new(d.x, d.y, d.z),
                    0.1,
                    0.44,
                ),
            )
            .unwrap();
            ecs.attach_component(&hl, Position(lpos)).unwrap();
            ecs.attach_component(&hl, LightDirty(true)).unwrap();
            ecs.attach_component(
                &hl,
                ShadowCaster {
                    cascade_count: 0,
                    bias: 0.0002,
                    ..Default::default()
                },
            )
            .unwrap();
        }

        // ── Colorful point-light ring for Room 5 helmet (x=20) ───────
        let helmet_20 = Point3::new(20.0, 2.2, 0.0);
        let colors: [Vector3<_>; 8] = [
            Vector3::new(1.0, 0.2, 0.2),
            Vector3::new(1.0, 0.6, 0.1),
            Vector3::new(1.0, 1.0, 0.2),
            Vector3::new(0.2, 1.0, 0.2),
            Vector3::new(0.2, 0.6, 1.0),
            Vector3::new(0.4, 0.2, 1.0),
            Vector3::new(1.0, 0.2, 1.0),
            Vector3::new(1.0, 0.9, 0.8),
        ];
        for i in 0u32..8u32 {
            let angle = std::f32::consts::TAU * i as f32 / 8.0;
            let (s, c) = angle.sin_cos();
            let lpos = Point3::new(helmet_20.x + c * 3.0, 4.0, helmet_20.z + s * 3.0);
            let pl = ecs.spawn_entity();
            ecs.attach_component(&pl, LightDescriptorEcs::new_point(colors[i as usize], 30.0))
                .unwrap();
            ecs.attach_component(&pl, Position(lpos)).unwrap();
            ecs.attach_component(&pl, LightDirty(true)).unwrap();
        }
        let animator = LightAnimator::new(spot_light);

        // Low-intensity directional fill light for scene-wide diffuse lighting
        let fill = ecs.spawn_entity();
        ecs.attach_component(
            &fill,
            LightDescriptorEcs::new_directional(
                Vector3::new(0.0, -0.6, -0.8), // emission direction (shader negates -> light from above)
                Vector3::new(1.0, 0.88, 0.65), // subtle warm white
                0.25,
            ),
        )
        .unwrap();
        ecs.attach_component(&fill, Position(Point3::new(0.0, 0.0, 0.0)))
            .unwrap();
        ecs.attach_component(&fill, LightDirty(true)).unwrap();

        vec![
            sys_camera_controller.into_system(),
            Box::new(HelmetAdjuster::new()),
            Box::new(animator),
            sys_animate_dynamic_sky(14.0).into_system(),
        ]
    }
}
