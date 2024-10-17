use std::{path::PathBuf, process::Command};

fn path_to_models_dir() -> String {
    const MODELS_DIR: &str = "../Assets/Models/";
    let current_dir = std::env::current_dir().unwrap();
    let current_dir_str = current_dir.to_str().unwrap();

    let concat = format!("{}/{}", current_dir_str, MODELS_DIR);
    let canonicalize = std::fs::canonicalize(&concat).unwrap();

    let canonicalize_str = canonicalize.to_str().unwrap().to_string();
    return canonicalize_str;
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    blender_pbr_spheres();
    blender_model_files();
}

fn blender_pbr_spheres() {
    println!(
        "cargo::rerun-if-changed={}/pbr_sphere_gen.py",
        path_to_models_dir()
    );
    println!(
        "cargo::rerun-if-changed={}/PBR_Spheres.glb",
        path_to_models_dir()
    );

    let mut handle = Command::new("blender")
        .arg("--background")
        .arg("--python")
        .arg(format!("{}/pbr_sphere_gen.py", path_to_models_dir()))
        .current_dir(path_to_models_dir())
        .spawn()
        .expect("Failed to run Blender command!");

    let exit_code = handle.wait().expect("Failed to wait for process to finish");
    if !exit_code.success() {
        panic!(
            "Failed generating PBR Spheres and export to glTF! Blender exited with code: {}",
            exit_code
        );
    }
}

fn blender_model_files() {
    println!(
        "cargo::rerun-if-changed={}/blender_gltf_export.py",
        path_to_models_dir()
    );

    for entry in glob::glob(&format!("{}/*.blend", path_to_models_dir())).unwrap() {
        let path = entry.unwrap();

        blender_convert_to_gltf(path.into_os_string().into_string().unwrap().as_str());
    }
}

fn blender_convert_to_gltf(filepath: &str) {
    println!(
        "cargo::rerun-if-changed={}/{}",
        path_to_models_dir(),
        filepath
    );

    let mut handle = Command::new("blender")
        .arg("--background")
        .arg(filepath)
        .arg("--python")
        .arg(format!("{}/blender_gltf_export.py", path_to_models_dir()))
        .current_dir(path_to_models_dir())
        .spawn()
        .expect("Failed to run Blender command!");

    let exit_code = handle.wait().expect("Failed to wait for process to finish");
    if !exit_code.success() {
        panic!(
            "Failed converting Blender file to glTF! Blender exited with code: {}",
            exit_code
        );
    }
}
