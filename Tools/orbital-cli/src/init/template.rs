use anyhow::{Context, Result};
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
"#,
        package = config.package_name,
        min_sdk = config.min_sdk,
        target_sdk = config.target_sdk,
    );

    fs::write(project_dir.join("Orbital.toml"), content).context("Failed to write Orbital.toml")?;

    Ok(())
}

fn generate_lib_rs(project_dir: &Path, config: &ProjectConfig) -> Result<()> {
    let template = match config.template.as_str() {
        "skybox" => SKYBOX_TEMPLATE,
        "instancing" => INSTANCING_TEMPLATE,
        "gltf" => GLTF_TEMPLATE,
        _ => MINIMAL_TEMPLATE,
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

const MINIMAL_TEMPLATE: &str = r#"use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{Point3, Rad};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, Position, Rotation,
};
use orbital::logging::{self, error, info};

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
        ecs.attach_component(&camera, Position(Point3::new(0.0, 2.0, 5.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        vec![sys_camera_controller.into_system()]
    }

    fn save_state(&self, ecs: &mut World) {
        let Some(camera) = ecs.get_resource::<ActiveCamera>() else {
            return;
        };
        let position = {
            let Some(store) = ecs.get_component_store::<Position>() else {
                return;
            };
            let Some(position) = store.get_component(camera.0.index) else {
                return;
            };
            *position
        };
        let data = format!("{},{},{}", position.0.x, position.0.y, position.0.z);
        if let Ok(fm) = orbital::file_manager::FileManager::global() {
            let _ = fm.write_bytes(SAVE_PATH, data.as_bytes());
            info!("Saved camera position: {data}");
        }
    }

    fn restore_state(&self, ecs: &mut World) {
        let Ok(fm) = orbital::file_manager::FileManager::global() else {
            return;
        };
        if !fm.storage_path_exists(SAVE_PATH) {
            return;
        }
        let Ok(bytes) = fm.read_bytes(SAVE_PATH) else {
            return;
        };
        let Ok(text) = String::from_utf8(bytes) else {
            return;
        };
        let parts: Vec<f32> = text
            .split(',')
            .filter_map(|s| s.trim().parse().ok())
            .collect();
        if parts.len() != 3 {
            return;
        }
        let Some(camera) = ecs.get_resource::<ActiveCamera>() else {
            return;
        };
        if let Some(mut store) = ecs.get_component_store_mut::<Position>() {
            if let Some(index) = store.sparse.get(camera.0.index).copied().flatten() {
                store.components[index] = Position(Point3::new(parts[0], parts[1], parts[2]));
                info!("Restored camera position: {text}");
            }
        }
    }

    fn clear_state(&self, _ecs: &mut World) {
        if let Ok(fm) = orbital::file_manager::FileManager::global() {
            let _ = fm.remove_file(SAVE_PATH);
            info!("Cleared saved state");
        }
    }
}

const SAVE_PATH: &str = "savegame.bin";
"#;

const SKYBOX_TEMPLATE: &str = r#"use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{Point3, Rad};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, Position, Rotation,
};
use orbital::logging::{self, error, info};

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
        .add_module(SkyboxModule)
        .liftoff(event_loop, app_settings)
    {
        Ok(()) => info!("Cleanly exited!"),
        Err(e) => error!("Runtime failure: {e:?}"),
    }
}

orbital::make_main!(entrypoint);

struct SkyboxModule;

impl Module for SkyboxModule {
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
        ecs.attach_component(&camera, Position(Point3::new(0.0, 2.0, 5.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        vec![sys_camera_controller.into_system()]
    }
}
"#;

const INSTANCING_TEMPLATE: &str = r#"use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{Point3, Rad};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, ImportQueueResource, Position, Rotation,
};
use orbital::importer::{ImportTask, gltf::GltfImport};
use orbital::logging::{self, error, info};

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
        ecs.attach_component(&camera, Position(Point3::new(0.0, 2.0, 5.0)))
            .unwrap();
        ecs.attach_component(&camera, Rotation::identity()).unwrap();
        ecs.insert_resource(ActiveCamera(camera));
        ecs.insert_resource(CursorGrabConfig(true));

        // Import a model
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Assets/Models/DamagedHelmet.glb".into(),
                task: GltfImport::WholeFile,
            });
        }

        vec![sys_camera_controller.into_system()]
    }
}
"#;

const GLTF_TEMPLATE: &str = r#"use orbital::app::{App, AppSettings, Module, sys_camera_controller};
use orbital::cgmath::{Point3, Rad, Vector3};
use orbital::ecs::{IntoSystem, System, World};
use orbital::ecs_bridge::{
    ActiveCamera, CameraDescriptorEcs, CursorGrabConfig, ImportQueueResource, Position, Rotation,
};
use orbital::importer::{ImportTask, gltf::GltfImport};
use orbital::logging::{self, error, info};

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

        // Import a glTF model
        if let Some(mut queue) = ecs.get_resource_mut::<ImportQueueResource>() {
            queue.push(ImportTask::Gltf {
                file_path: "Assets/Models/DamagedHelmet.glb".into(),
                task: GltfImport::WholeFile,
            });
        }

        vec![sys_camera_controller.into_system()]
    }
}
"#;
