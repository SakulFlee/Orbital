use std::sync::Arc;

use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{InnerSpace, Point3, Rad, Vector3};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptor, CursorGrabConfig, EnvironmentDescriptorResource,
    LightDescriptor, LightDirty, ModelDescriptor, ModelDirty, ModelInstances, Position, Rotation,
};
use orbital::logging::{self, error, info};
use orbital::procgeo::scene::{
    EntityDescriptor, SceneBuilder, SceneMaterial, SceneShape, TransformDef,
};
use orbital::resources::{
    GeneratedSkyParameters, SamplingType, ShadowCaster, WorldEnvironmentDescriptor,
};

pub const NAME: &str = env!("CARGO_PKG_NAME");

#[orbital::main]
pub fn entrypoint(
    event_loop_result: Result<
        orbital::winit::event_loop::EventLoop<()>,
        orbital::winit::error::EventLoopError,
    >,
) {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
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
        .add_module(orbital::touch_ui::TouchUiModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

struct GameModule;

impl Module for GameModule {
    fn setup(
        &self,
        ecs: &mut World,
        _device: &orbital::wgpu::Device,
        _queue: &orbital::wgpu::Queue,
    ) -> Vec<Box<dyn System>> {
        // Spawn camera looking at the scene
        let camera = ecs.spawn_entity();
        ecs.attach_component(
            &camera,
            CameraDescriptor {
                label: "Default".into(),
                aspect: 16.0 / 9.0,
                fovy: Rad(std::f32::consts::FRAC_PI_4),
                projection_mode: Default::default(),
                ortho_height: 10.0,
                near: 0.1,
                far: 10000.0,
                global_gamma: 2.2,
            },
        )
        .unwrap();
        ecs.attach_component(&camera, Position(Point3::new(0.0, 3.0, 5.0)))
            .unwrap();
        let mut rot = Rotation::identity();
        rot.rotate_yaw(Rad(std::f32::consts::FRAC_PI_2));
        rot.rotate_pitch(Rad(-0.3));
        ecs.attach_component(&camera, rot).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        // Dynamic procedural sky for ambient lighting
        ecs.insert_resource(EnvironmentDescriptorResource(Some(
            WorldEnvironmentDescriptor::Generated {
                cube_face_size: 256,
                sampling_type: SamplingType::GaussianBlur,
                custom_specular_mip_level_count: Some(3),
                parameters: Some(GeneratedSkyParameters::default()),
                dynamic: true,
            },
        )));

        // Build a minimal scene: a cube sitting on a plane
        let mut scene = SceneBuilder::new();
        scene.add_material(
            "floor",
            SceneMaterial::Color {
                albedo: [0.2, 0.2, 0.25, 1.0],
                metallic: 0.0,
                roughness: 0.95,
            },
        );
        scene.add_material(
            "cube",
            SceneMaterial::Color {
                albedo: [0.9, 0.3, 0.2, 1.0],
                metallic: 0.0,
                roughness: 0.5,
            },
        );
        scene.add_entity(EntityDescriptor {
            label: Some("Floor".into()),
            shape: SceneShape::Plane {
                size: [10.0, 10.0],
                subdivisions: 2,
            },
            material: "floor".into(),
            transform: TransformDef::default(),
        });
        scene.add_entity(EntityDescriptor {
            label: Some("Cube".into()),
            shape: SceneShape::Box {
                size: [1.0, 1.0, 1.0],
            },
            material: "cube".into(),
            transform: TransformDef {
                position: [0.0, 0.5, 0.0],
                ..Default::default()
            },
        });

        // Spawn the generated meshes as renderable entities
        for (mesh, material, transform) in scene.build() {
            let entity = ecs.spawn_entity();
            ecs.attach_component(
                &entity,
                ModelDescriptor {
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

        // A dense ring of unshadowed point lights exercises the tiled light
        // list and the forward PBR light loop without paying for 96 cube maps.
        let palette = [
            Vector3::new(1.0, 0.2, 0.2),
            Vector3::new(1.0, 0.6, 0.1),
            Vector3::new(1.0, 1.0, 0.2),
            Vector3::new(0.2, 1.0, 0.3),
            Vector3::new(0.2, 0.8, 1.0),
            Vector3::new(0.4, 0.3, 1.0),
            Vector3::new(0.9, 0.2, 1.0),
            Vector3::new(1.0, 0.9, 0.8),
        ];
        for index in 0..96u32 {
            let angle = std::f32::consts::TAU * index as f32 / 96.0;
            let ring = (index % 4) as f32;
            let radius = 2.2 + ring * 0.8;
            let height = 0.6 + (index % 8) as f32 * 0.35;
            let light = ecs.spawn_entity();
            ecs.attach_component(
                &light,
                LightDescriptor::new_point(palette[index as usize % palette.len()], 0.25),
            )
            .unwrap();
            ecs.attach_component(
                &light,
                Position(Point3::new(
                    angle.cos() * radius,
                    height,
                    angle.sin() * radius,
                )),
            )
            .unwrap();
            ecs.attach_component(&light, LightDirty(true)).unwrap();
        }

        // Spotlight aimed at the cube
        let spot_pos = Point3::new(3.0, 5.0, 3.0);
        let cube_target = Point3::new(0.0, 0.5, 0.0);
        let dir = (cube_target - spot_pos).normalize();
        let spot_light = ecs.spawn_entity();
        ecs.attach_component(
            &spot_light,
            LightDescriptor::new_spot(
                Vector3::new(1.0, 1.0, 1.0),
                30.0,
                Vector3::new(dir.x, dir.y, dir.z),
                0.1,
                0.44,
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

        vec![sys_camera_controller.into_system()]
    }
}
