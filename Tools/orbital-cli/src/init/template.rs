use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

use super::prompt::ProjectConfig;

pub fn generate_project(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    // Create directory structure
    fs::create_dir_all(project_dir.join("src"))?;

    // Generate Cargo.toml
    generate_cargo_toml(project_dir, config)?;

    // Generate Orbital.toml
    generate_orbital_toml(project_dir, config)?;

    // Generate src/lib.rs from template
    generate_lib_rs(project_dir, config)?;

    // Generate src/main.rs for desktop
    generate_main_rs(project_dir, config)?;

    Ok(())
}

fn generate_cargo_toml(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let lib_name = config.project_name.replace('-', "_").to_lowercase();

    let content = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
name = "{lib_name}"
crate-type = ["cdylib", "lib"]

[[bin]]
name = "{name}_desktop"
path = "src/main.rs"

[dependencies]
orbital = {{ git = "{repo}", branch = "{branch}" }}
winit = "0.30.0"
"#,
        name = config.project_name,
        lib_name = lib_name,
        repo = config.engine_repo,
        branch = config.engine_branch,
    );

    fs::write(project_dir.join("Cargo.toml"), content)
        .context("Failed to write Cargo.toml")?;

    Ok(())
}

fn generate_orbital_toml(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let content = format!(
        r#"[orbital]

[android]
package = "{package}"
min_sdk = {min_sdk}
target_sdk = {target_sdk}
targets = ["arm64-v8a", "armeabi-v7a", "x86_64", "x86"]
apk_mode = "multiarch"
ndk_version = "26.2.11394342"
"#,
        package = config.package_name,
        min_sdk = config.min_sdk,
        target_sdk = config.target_sdk,
    );

    fs::write(project_dir.join("Orbital.toml"), content)
        .context("Failed to write Orbital.toml")?;

    Ok(())
}

fn generate_lib_rs(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    // Only the minimal template exists for now. To add a new template:
    //   1. Add a `const FOO_TEMPLATE: &str = r#"..."#;` below.
    //   2. Add a `"foo" => FOO_TEMPLATE` arm to this match.
    //   3. Add "foo" to the prompt list in `init/prompt.rs`.
    let template = match config.template.as_str() {
        "minimal" => MINIMAL_TEMPLATE,
        other => bail!(
            "Unknown template '{other}'. Available templates: minimal"
        ),
    };

    let content = template.replace("{{PROJECT_NAME}}", &config.project_name);

    fs::write(project_dir.join("src").join("lib.rs"), content)
        .context("Failed to write src/lib.rs")?;

    Ok(())
}

fn generate_main_rs(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let lib_name = config.project_name.replace('-', "_").to_lowercase();

    let content = format!(
        r#"fn main() {{
    // Desktop entry point - calls into lib.rs
    {lib_name}::entrypoint(Ok(
        orbital::winit::event_loop::EventLoop::builder()
            .build()
            .expect("Failed to create event loop")
    ));
}}
"#,
        lib_name = lib_name,
    );

    fs::write(project_dir.join("src").join("main.rs"), content)
        .context("Failed to write src/main.rs")?;

    Ok(())
}

const MINIMAL_TEMPLATE: &str = r#"use std::sync::Arc;

use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{InnerSpace, Point3, Rad, Vector3};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, EnvironmentDescriptorResource,
    LightDescriptorEcs, LightDirty, ModelDescriptorEcs, ModelDirty, ModelInstances, Position,
    Rotation,
};
use orbital::logging::{self, error, info};
use orbital::procgeo::scene::{
    EntityDescriptor, SceneBuilder, SceneMaterial, SceneShape, TransformDef,
};
use orbital::resources::{
    GeneratedSkyParameters, SamplingType, ShadowCaster, WorldEnvironmentDescriptor,
};

pub const NAME: &str = "{{PROJECT_NAME}}";

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
    app_settings.vsync_enabled = true;
    app_settings.name = NAME.to_string();

    match App::new()
        .add_module(GameModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

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
        ecs.attach_component(&camera, Position(Point3::new(0.0, 2.0, 5.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();
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

        // Spotlight aimed at the cube
        let spot_pos = Point3::new(3.0, 5.0, 3.0);
        let cube_target = Point3::new(0.0, 0.5, 0.0);
        let dir = (cube_target - spot_pos).normalize();
        let spot_light = ecs.spawn_entity();
        ecs.attach_component(
            &spot_light,
            LightDescriptorEcs::new_spot(
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
"#;