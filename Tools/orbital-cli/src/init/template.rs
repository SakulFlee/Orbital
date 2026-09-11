use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

use super::prompt::ProjectConfig;

pub fn generate_project(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    match config.template.as_str() {
        "minimal" => generate_project_minimal(project_dir, config),
        "procgeo_scene" => generate_project_procgeo_scene(project_dir, config),
        other => bail!("Unknown template '{other}'. Available templates: minimal, procgeo_scene"),
    }
}

fn generate_project_minimal(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
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

    // Create the Assets/ directory. Assets referenced by engine shaders
    // (e.g. "Shaders/pbr.wgsl") resolve against this directory on desktop
    // (<cwd>/Assets) and are bundled into the APK on Android.
    fs::create_dir_all(project_dir.join("Assets"))?;
    fs::write(project_dir.join("Assets").join(".gitkeep"), "")
        .context("Failed to write Assets/.gitkeep")?;

    Ok(())
}

fn generate_project_procgeo_scene(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let lib_name = config.project_name.replace('-', "_").to_lowercase();

    // Get the template directory path (relative to the executable)
    let template_dir = std::env::current_exe()
        .context("Failed to get executable path")?
        .parent()
        .context("Failed to get executable parent")?
        .join("templates")
        .join("procgeo_scene");

    // If the template directory doesn't exist next to the executable,
    // fall back to looking in the source tree
    let template_dir = if template_dir.exists() {
        template_dir
    } else {
        // Try to find it relative to the source file
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("template")
            .join("procgeo_scene")
    };

    if !template_dir.exists() {
        bail!(
            "Template directory not found: {}",
            template_dir.display()
        );
    }

    // Copy the entire template directory
    copy_dir_all(&template_dir, project_dir)
        .context("Failed to copy template directory")?;

    // Generate Orbital.toml
    generate_orbital_toml(project_dir, config)?;

    // Replace placeholders in Cargo.toml
    let cargo_toml_path = project_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        let content = fs::read_to_string(&cargo_toml_path)
            .context("Failed to read Cargo.toml")?;
        let content = content
            .replace("{name}", &config.project_name)
            .replace("{lib_name}", &lib_name)
            .replace("{repo}", &config.engine_repo)
            .replace("{branch}", &config.engine_branch);
        fs::write(&cargo_toml_path, content)
            .context("Failed to write Cargo.toml")?;
    }

    // Replace placeholders in lib.rs
    let lib_rs_path = project_dir.join("src").join("lib.rs");
    if lib_rs_path.exists() {
        let content = fs::read_to_string(&lib_rs_path)
            .context("Failed to read lib.rs")?;
        let content = content.replace("{{PROJECT_NAME}}", &config.project_name);
        fs::write(&lib_rs_path, content)
            .context("Failed to write lib.rs")?;
    }

    // Replace placeholders in main.rs
    let main_rs_path = project_dir.join("src").join("main.rs");
    if main_rs_path.exists() {
        let content = fs::read_to_string(&main_rs_path)
            .context("Failed to read main.rs")?;
        let content = content.replace("{lib_name}", &lib_name);
        fs::write(&main_rs_path, content)
            .context("Failed to write main.rs")?;
    }

    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src).context("Failed to read template directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let ty = entry.file_type().context("Failed to get file type")?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))
                .context(format!("Failed to copy {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn generate_cargo_toml(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let lib_name = config.project_name.replace('-', "_").to_lowercase();

    let content = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "{name}_desktop"
path = "src/main.rs"

[lib]
name = "{lib_name}"
crate-type = ["cdylib", "lib"]

[dependencies]
orbital = {{ git = "{repo}", branch = "{branch}" }}
winit = "0.30.0"
"#,
        name = config.project_name,
        lib_name = lib_name,
        repo = config.engine_repo,
        branch = config.engine_branch,
    );

    fs::write(project_dir.join("Cargo.toml"), content).context("Failed to write Cargo.toml")?;

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
screen_orientation = "unspecified"
"#,
        package = config.package_name,
        min_sdk = config.min_sdk,
        target_sdk = config.target_sdk,
    );

    fs::write(project_dir.join("Orbital.toml"), content).context("Failed to write Orbital.toml")?;

    Ok(())
}

fn generate_lib_rs(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    // Only the minimal template exists for now. To add a new template:
    //   1. Add a `const FOO_TEMPLATE: &str = r#"..."#;` below.
    //   2. Add a `"foo" => FOO_TEMPLATE` arm to this match.
    //   3. Add "foo" to the prompt list in `init/prompt.rs`.
    let template = match config.template.as_str() {
        "minimal" => MINIMAL_TEMPLATE,
        "procgeo_scene" => {
            // procgeo_scene is handled by generate_project_procgeo_scene
            // This function should not be called for procgeo_scene template
            bail!("procgeo_scene template should be handled by generate_project_procgeo_scene");
        }
        other => bail!("Unknown template '{other}'. Available templates: minimal, procgeo_scene"),
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
    app_settings.back_presses_to_exit = 3;

    match App::new()
        .add_module(GameModule)
        .add_module(orbital::touch_ui::TouchUiModule)
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

const SAVE_PATH: &str = "savegame.bin";
"#;
