use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{Point3, Rad, Vector3};
use orbital::debug_render::DebugModule;
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, ImportQueueResource, LightDescriptorEcs,
    LightDirty, Position, Rotation,
};
use orbital::importer::{ImportTask, gltf::GltfImport};
use orbital::file_manager::FileManager;
use orbital::logging::{self, error, info};
use orbital::resources::ShadowCaster;
use winit::keyboard::KeyCode;

pub const NAME: &str = "Orbital-Demo-Project: DamagedHelmet";

pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    logging::init();

    let event_loop = event_loop_result.expect("Event Loop failure");

    #[cfg(target_os = "android")]
    {
        use orbital::winit::platform::android::EventLoopExtAndroid;
        let app = event_loop.android_app();
        FileManager::init_android_global(
            app.asset_manager(),
            app.internal_data_path(),
        )
        .expect("Failed to initialize FileManager for Android");
    }

    let mut app_settings = AppSettings::default();
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match App::new()
        .add_module(DamagedHelmetModule)
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

struct DamagedHelmetModule;

impl Module for DamagedHelmetModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn camera
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
        ecs.attach_component(&camera, Position(Point3::new(-10.0, 0.0, 0.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        // Spot light from behind the camera, casting shadows
        let spot = ecs.spawn_entity();
        ecs.attach_component(
            &spot,
            LightDescriptorEcs::new_spot(
                Vector3::new(1.0, 0.9, 0.7), // warm white
                8.0,
                Vector3::new(0.0, -1.0, 1.0), // down + forward toward helmet
                0.3,                          // inner cone (~17°)
                0.5,                          // outer cone (~29°)
            ),
        )
        .unwrap();
        ecs.attach_component(&spot, Position(Point3::new(0.0, 4.0, -4.0)))
            .unwrap();
        ecs.attach_component(&spot, LightDirty(true)).unwrap();
        ecs.attach_component(
            &spot,
            ShadowCaster {
                cascade_count: 0, // ignored for spot lights (single perspective map)
                ..Default::default()
            },
        )
        .unwrap();

        // Import model
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Models/DamagedHelmet.glb".into(),
                task: GltfImport::WholeFile,
            });
        }

        // Spawn directional light with shadows
        let sun = ecs.spawn_entity();
        ecs.attach_component(
            &sun,
            LightDescriptorEcs::new_directional(
                Vector3::new(-1.0, -1.0, -1.0),
                Vector3::new(1.0, 1.0, 1.0),
                1.5,
            ),
        )
        .unwrap();
        ecs.attach_component(&sun, Position(Point3::new(0.0, 0.0, 0.0)))
            .unwrap();
        ecs.attach_component(&sun, LightDirty(true)).unwrap();
        ecs.attach_component(&sun, ShadowCaster::default()).unwrap();

        // Rainbow ring of 10 point lights around the helmet
        let colors = [
            [1.0, 0.0, 0.0], // red
            [1.0, 0.5, 0.0], // orange
            [1.0, 1.0, 0.0], // yellow
            [0.5, 1.0, 0.0], // lime
            [0.0, 1.0, 0.0], // green
            [0.0, 1.0, 1.0], // cyan
            [0.0, 0.5, 1.0], // blue
            [0.5, 0.0, 1.0], // purple
            [1.0, 0.0, 1.0], // magenta
            [1.0, 0.2, 0.5], // pink
        ];
        let count = colors.len();
        for (i, &rgb) in colors.iter().enumerate() {
            let angle = i as f32 * std::f32::consts::TAU / count as f32;
            let (s, c) = angle.sin_cos();
            let entity = ecs.spawn_entity();
            ecs.attach_component(
                &entity,
                LightDescriptorEcs::new_point(Vector3::new(rgb[0], rgb[1], rgb[2]), 3.0),
            )
            .unwrap();
            ecs.attach_component(&entity, Position(Point3::new(c * 3.0, 2.0, s * 3.0)))
                .unwrap();
            ecs.attach_component(&entity, LightDirty(true)).unwrap();
            // First red light casts cube shadows
            if i == 0 {
                ecs.attach_component(
                    &entity,
                    ShadowCaster {
                        cascade_count: 0,
                        ..Default::default()
                    },
                )
                .unwrap();
            }
        }

        vec![sys_camera_controller.into_system()]
    }
}
